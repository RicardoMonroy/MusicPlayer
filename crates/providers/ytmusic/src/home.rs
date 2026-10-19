//! YouTube Music's Home feed ("Vuelve a escucharlo", "Mixes personalizados"...).
//! rustypipe has no Home query, so this calls the same Innertube `browse`
//! endpoint the web app uses, signed with the session cookies when there are some.

use std::path::Path;

use mp_core::Track;
use serde::Serialize;
use serde_json::{Value, json};

use crate::{Collection, Kind, watch_url};

const ORIGIN: &str = "https://music.youtube.com";
/// Used when rustypipe's cache has no client version yet.
const FALLBACK_VERSION: &str = "1.20261013.15.00";
/// Each continuation page brings 3-4 shelves; ~15 is what the web app shows.
const MAX_PAGES: usize = 6;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "item", rename_all = "lowercase")]
pub enum HomeItem {
    Track(Track),
    Collection(Collection),
}

#[derive(Debug, Clone, Serialize)]
pub struct Shelf {
    pub title: String,
    pub items: Vec<HomeItem>,
}

pub(crate) async fn fetch(dir: &Path, cookies_file: Option<&Path>) -> Result<Vec<Shelf>, String> {
    let cookies = cookies_file
        .and_then(|f| std::fs::read_to_string(f).ok())
        .map(|txt| parse_cookies(&txt))
        .unwrap_or_default();
    let version = client_version(dir).unwrap_or_else(|| FALLBACK_VERSION.into());
    let http = reqwest::Client::new();

    let mut page = browse(
        &http,
        &cookies,
        &version,
        json!({ "browseId": "FEmusic_home" }),
        "",
    )
    .await?;
    let list = &page["contents"]["singleColumnBrowseResultsRenderer"]["tabs"][0]["tabRenderer"]["content"]
        ["sectionListRenderer"];
    let mut shelves = parse_shelves(&list["contents"]);
    let mut next = continuation(list);
    for _ in 0..MAX_PAGES {
        let Some(token) = next else { break };
        let query = format!("&ctoken={token}&continuation={token}&type=next");
        page = browse(&http, &cookies, &version, json!({}), &query).await?;
        let list = &page["continuationContents"]["sectionListContinuation"];
        shelves.extend(parse_shelves(&list["contents"]));
        next = continuation(list);
    }
    Ok(shelves)
}

async fn browse(
    http: &reqwest::Client,
    cookies: &[(String, String)],
    version: &str,
    extra: Value,
    query: &str,
) -> Result<Value, String> {
    let mut body = json!({ "context": { "client": {
        "clientName": "WEB_REMIX", "clientVersion": version, "hl": "es", "gl": "MX"
    }}});
    if let (Some(b), Some(e)) = (body.as_object_mut(), extra.as_object()) {
        b.extend(e.clone());
    }
    let mut req = http
        .post(format!("{ORIGIN}/youtubei/v1/browse?prettyPrint=false{query}"))
        .header("Origin", ORIGIN)
        .header("X-Origin", ORIGIN)
        .header("Content-Type", "application/json")
        .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0 Safari/537.36")
        .body(body.to_string());
    if let Some(sapisid) = cookie(cookies, "SAPISID").or(cookie(cookies, "__Secure-3PAPISID")) {
        let cookie_header = cookies
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("; ");
        req = req
            .header("Authorization", sapisid_hash(sapisid))
            .header("X-Goog-AuthUser", "0")
            .header("Cookie", cookie_header);
    }
    let res = req.send().await.map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("YouTube Music respondió {}", res.status()));
    }
    res.json().await.map_err(|e| e.to_string())
}

/// The web app's auth header: SHA-1 of "<unix time> <SAPISID> <origin>".
fn sapisid_hash(sapisid: &str) -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    let digest = sha1_smol::Sha1::from(format!("{ts} {sapisid} {ORIGIN}"))
        .digest()
        .to_string();
    format!("SAPISIDHASH {ts}_{digest}")
}

fn cookie<'a>(cookies: &'a [(String, String)], name: &str) -> Option<&'a str> {
    cookies
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.as_str())
}

/// youtube.com cookies from a Netscape cookies.txt.
fn parse_cookies(txt: &str) -> Vec<(String, String)> {
    txt.lines()
        .filter(|l| !l.starts_with('#') || l.starts_with("#HttpOnly_"))
        .filter_map(|l| {
            let f: Vec<_> = l.split('\t').collect();
            (f.len() >= 7 && f[0].contains("youtube.com")).then(|| (f[5].into(), f[6].into()))
        })
        .collect()
}

fn client_version(dir: &Path) -> Option<String> {
    let cache: Value =
        serde_json::from_slice(&std::fs::read(dir.join("rustypipe_cache.json")).ok()?).ok()?;
    cache["clients"]["desktop_music"]["data"]["version"]
        .as_str()
        .map(String::from)
}

fn continuation(list: &Value) -> Option<String> {
    list["continuations"][0]["nextContinuationData"]["continuation"]
        .as_str()
        .map(String::from)
}

fn text(v: &Value) -> String {
    v["runs"]
        .as_array()
        .map(|r| r.iter().filter_map(|r| r["text"].as_str()).collect())
        .unwrap_or_default()
}

fn thumb(v: &Value) -> Option<String> {
    let thumbs = v["musicThumbnailRenderer"]["thumbnail"]["thumbnails"].as_array()?;
    crate::pick_thumb(
        thumbs
            .iter()
            .filter_map(|t| Some((t["width"].as_u64()? as u32, t["url"].as_str()?))),
    )
}

pub(crate) fn parse_shelves(contents: &Value) -> Vec<Shelf> {
    contents
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|s| {
            let shelf = &s["musicCarouselShelfRenderer"];
            let title = text(&shelf["header"]["musicCarouselShelfBasicHeaderRenderer"]["title"]);
            let items: Vec<_> = shelf["contents"]
                .as_array()?
                .iter()
                .filter_map(item)
                .collect();
            (!title.is_empty() && !items.is_empty()).then_some(Shelf { title, items })
        })
        .collect()
}

fn item(v: &Value) -> Option<HomeItem> {
    if let Some(r) = v.get("musicTwoRowItemRenderer") {
        let title = text(&r["title"]);
        let subtitle = text(&r["subtitle"]);
        let cover = thumb(&r["thumbnailRenderer"]);
        let nav = &r["navigationEndpoint"];
        if let Some(id) = nav["watchEndpoint"]["videoId"].as_str() {
            return Some(HomeItem::Track(Track {
                title,
                artist: strip_kind(&subtitle),
                uri: watch_url(id),
                cover,
                ..Default::default()
            }));
        }
        let id = nav["browseEndpoint"]["browseId"].as_str()?;
        let (kind, id) = if id.starts_with("MPRE") {
            (Kind::Album, id)
        } else if let Some(pl) = id.strip_prefix("VL") {
            (Kind::Playlist, pl)
        } else if id.starts_with("UC") {
            (Kind::Artist, id)
        } else {
            return None;
        };
        return Some(HomeItem::Collection(Collection {
            kind,
            id: id.into(),
            title,
            subtitle: strip_kind(&subtitle),
            cover,
        }));
    }
    let r = v.get("musicResponsiveListItemRenderer")?;
    let column =
        |i: usize| &r["flexColumns"][i]["musicResponsiveListItemFlexColumnRenderer"]["text"];
    let id = r["playlistItemData"]["videoId"]
        .as_str()
        .or(column(0)["runs"][0]["navigationEndpoint"]["watchEndpoint"]["videoId"].as_str())?;
    Some(HomeItem::Track(Track {
        title: text(column(0)),
        artist: column(1)["runs"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .into(),
        uri: watch_url(id),
        cover: thumb(&r["thumbnail"]),
        ..Default::default()
    }))
}

/// "Canción • Opeth" → "Opeth", "Álbum • Moonspell" → "Moonspell": the icon
/// already shows the kind.
fn strip_kind(subtitle: &str) -> String {
    match subtitle.split_once(" • ") {
        Some((kind, rest))
            if [
                "Canción", "Song", "Video", "Vídeo", "Álbum", "Album", "EP", "Sencillo", "Single",
                "Playlist", "Lista",
            ]
            .contains(&kind) =>
        {
            rest.into()
        }
        _ => subtitle.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_shelves_of_every_item_kind() {
        let two_row = |nav: Value, title: &str, sub: &str| {
            json!({ "musicTwoRowItemRenderer": {
                "title": { "runs": [{ "text": title }] },
                "subtitle": { "runs": [{ "text": sub }] },
                "thumbnailRenderer": { "musicThumbnailRenderer": { "thumbnail": { "thumbnails": [
                    { "url": "small", "width": 60 }, { "url": "big", "width": 226 } ] } } },
                "navigationEndpoint": nav
            }})
        };
        let contents = json!([
            { "musicCarouselShelfRenderer": {
                "header": { "musicCarouselShelfBasicHeaderRenderer": { "title": { "runs": [{ "text": "Mixes" }] } } },
                "contents": [
                    two_row(json!({ "watchEndpoint": { "videoId": "v1" } }), "Song", "Canción • Opeth"),
                    two_row(json!({ "browseEndpoint": { "browseId": "MPREb_x" } }), "Album", "Álbum • 2001"),
                    two_row(json!({ "browseEndpoint": { "browseId": "VLPLabc" } }), "Mix", "Playlist"),
                    two_row(json!({ "browseEndpoint": { "browseId": "UCzz" } }), "Opeth", "Artista"),
                    two_row(json!({ "browseEndpoint": { "browseId": "FEmusic_x" } }), "skip", ""),
                    { "musicResponsiveListItemRenderer": {
                        "playlistItemData": { "videoId": "v2" },
                        "flexColumns": [
                            { "musicResponsiveListItemFlexColumnRenderer": { "text": { "runs": [{ "text": "Harvest" }] } } },
                            { "musicResponsiveListItemFlexColumnRenderer": { "text": { "runs": [{ "text": "Opeth" }, { "text": " • x" }] } } }
                        ] } }
                ] } },
            { "musicCarouselShelfRenderer": { "header": {}, "contents": [] } }
        ]);
        let shelves = parse_shelves(&contents);
        assert_eq!(shelves.len(), 1, "empty shelves are dropped");
        let items = &shelves[0].items;
        assert_eq!(items.len(), 5, "unknown browse ids are skipped");
        assert!(
            matches!(&items[0], HomeItem::Track(t) if t.artist == "Opeth" && t.uri == watch_url("v1") && t.cover.as_deref() == Some("big"))
        );
        assert!(
            matches!(&items[1], HomeItem::Collection(c) if c.kind == Kind::Album && c.id == "MPREb_x")
        );
        assert!(
            matches!(&items[2], HomeItem::Collection(c) if c.kind == Kind::Playlist && c.id == "PLabc")
        );
        assert!(matches!(&items[3], HomeItem::Collection(c) if c.kind == Kind::Artist));
        assert!(
            matches!(&items[4], HomeItem::Track(t) if t.title == "Harvest" && t.artist == "Opeth")
        );
    }

    #[test]
    fn signs_with_sapisid_and_reads_cookies() {
        let txt = "# Netscape\n.youtube.com\tTRUE\t/\tTRUE\t0\tSAPISID\tabc\n.google.com\tTRUE\t/\tTRUE\t0\tSID\tx\n";
        let c = parse_cookies(txt);
        assert_eq!(c, vec![("SAPISID".into(), "abc".into())]);
        let h = sapisid_hash("abc");
        assert!(h.starts_with("SAPISIDHASH ") && h.len() > 50);
    }
}
