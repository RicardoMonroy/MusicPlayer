//! YouTube Music through rustypipe (Innertube). Playback needs no stream
//! extraction here: mpv resolves `music.youtube.com/watch` URLs with yt-dlp.

use std::path::{Path, PathBuf};

use mp_core::Track;
use rustypipe::{
    client::RustyPipe,
    model::{AlbumItem, ArtistId, ArtistItem, MusicPlaylistItem, Thumbnail, TrackItem},
};
use serde::{Deserialize, Serialize};

pub use rustypipe::error::Error;

mod home;
pub use home::{HomeItem, Shelf};
pub mod translate;

const COOKIES: &str = "cookies.txt";

fn write_private(path: &Path, contents: &str) -> std::io::Result<()> {
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut opts, 0o600);
    std::io::Write::write_all(&mut opts.open(path)?, contents.as_bytes())
}

/// Upper bound when expanding long playlists, to keep a request burst sane.
const MAX_TRACKS: usize = 1000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Album,
    Playlist,
    Artist,
}

/// An album, playlist or artist shown as a card; open it to get its tracks.
#[derive(Debug, Clone, Serialize)]
pub struct Collection {
    pub kind: Kind,
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub cover: Option<String>,
}

/// The best match, shown big like YouTube Music's "Top result".
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "item", rename_all = "lowercase")]
pub enum Top {
    Artist(Collection),
    Track(Track),
}

#[derive(Debug, Default, Serialize)]
pub struct SearchResults {
    pub top: Option<Top>,
    pub tracks: Vec<Track>,
    pub videos: Vec<Track>,
    pub artists: Vec<Collection>,
    pub albums: Vec<Collection>,
    pub playlists: Vec<Collection>,
}

/// An artist's page: popular tracks and discography.
#[derive(Debug, Serialize)]
pub struct ArtistPage {
    pub name: String,
    pub cover: Option<String>,
    pub tracks: Vec<Track>,
    pub albums: Vec<Collection>,
    pub playlists: Vec<Collection>,
}

#[derive(Debug, Default, Serialize)]
pub struct Library {
    pub playlists: Vec<Collection>,
    pub albums: Vec<Collection>,
}

pub struct YtMusic {
    rp: RustyPipe,
    dir: PathBuf,
}

impl YtMusic {
    /// `storage_dir` keeps rustypipe's cache, including login cookies: keep it private.
    pub fn new(storage_dir: &Path) -> Result<Self, Error> {
        Ok(Self {
            rp: RustyPipe::builder().storage_dir(storage_dir).build()?,
            dir: storage_dir.to_path_buf(),
        })
    }

    pub async fn search(&self, query: &str) -> Result<SearchResults, Error> {
        let q = self.rp.query();
        let (tracks, videos, artists, albums, featured, community) = tokio::join!(
            q.music_search_tracks(query),
            q.music_search_videos(query),
            q.music_search_artists(query),
            q.music_search_albums(query),
            q.music_search_playlists(query, false),
            q.music_search_playlists(query, true),
        );
        let mut r = SearchResults {
            top: None,
            tracks: tracks?.items.items.into_iter().map(track).collect(),
            // Videos and artists are extras: a failure there should not hide songs.
            videos: videos
                .map(|v| v.items.items.into_iter().map(track).collect())
                .unwrap_or_default(),
            artists: artists
                .map(|a| a.items.items.into_iter().map(artist).collect())
                .unwrap_or_default(),
            albums: albums?.items.items.into_iter().map(album).collect(),
            playlists: featured?
                .items
                .items
                .into_iter()
                .chain(community?.items.items)
                .map(playlist)
                .collect(),
        };
        r.top = top_result(query, &r);
        Ok(r)
    }

    pub async fn artist(&self, id: &str) -> Result<ArtistPage, Error> {
        // Full discography: people open an artist to play whole albums (~50 ms more).
        let mut a = self.rp.query().music_artist(id, true).await?;
        // Albums first, then EPs and singles; YouTube's order (newest first) within each.
        a.albums
            .sort_by_key(|x| x.album_type != rustypipe::model::AlbumType::Album);
        Ok(ArtistPage {
            cover: best_thumb(&a.header_image),
            name: a.name,
            tracks: a.tracks.into_iter().map(track).collect(),
            albums: a.albums.into_iter().map(album).collect(),
            playlists: a.playlists.into_iter().map(playlist).collect(),
        })
    }

    pub async fn open(&self, kind: &Kind, id: &str) -> Result<Vec<Track>, Error> {
        let q = self.rp.query();
        match kind {
            Kind::Album => {
                let a = q.music_album(id).await?;
                let cover = best_thumb(&a.cover);
                Ok(a.tracks
                    .into_iter()
                    .map(|t| {
                        let mut t = track(t);
                        t.album.clone_from(&a.name);
                        t.cover = t.cover.or_else(|| cover.clone());
                        t.album_id = Some(id.to_string());
                        t.artist_id = t.artist_id.or_else(|| a.artist_id.clone());
                        t
                    })
                    .collect())
            }
            Kind::Artist => Ok(self.artist(id).await?.tracks),
            Kind::Playlist => match self.playlist_tracks(id).await {
                Ok(tracks) => Ok(tracks),
                // rustypipe rejects some valid playlists (old 13-char IDs, mixed
                // "album" playlists); yt-dlp lists them fine.
                Err(e) => self.playlist_via_ytdlp(id).await.ok_or(e),
            },
        }
    }

    /// Private playlists only resolve with the session, so retry signed in.
    async fn playlist_tracks(&self, id: &str) -> Result<Vec<Track>, Error> {
        let q = self.rp.query();
        let q = match q.music_playlist(id).await {
            Ok(p) => return self.expand(&q, p).await,
            Err(_) if self.logged_in().await => self.rp.query().authenticated(),
            Err(e) => return Err(e),
        };
        let p = q.music_playlist(id).await?;
        self.expand(&q, p).await
    }

    async fn expand(
        &self,
        q: &rustypipe::client::RustyPipeQuery,
        mut p: rustypipe::model::MusicPlaylist,
    ) -> Result<Vec<Track>, Error> {
        p.tracks.extend_limit(q, MAX_TRACKS).await?;
        Ok(p.tracks.items.into_iter().map(track).collect())
    }

    /// Shareable album link, as the OLAK5uy playlist that yt-dlp based
    /// downloaders treat as an album.
    pub async fn album_url(&self, album_id: &str) -> Result<String, Error> {
        let id = if album_id.starts_with("OLAK5uy") {
            album_id.to_string()
        } else {
            let a = self.rp.query().music_album(album_id).await?;
            match a.playlist_id {
                Some(p) => p,
                None => return Ok(format!("https://music.youtube.com/browse/{album_id}")),
            }
        };
        Ok(format!("https://music.youtube.com/playlist?list={id}"))
    }

    async fn playlist_via_ytdlp(&self, id: &str) -> Option<Vec<Track>> {
        let mut cmd = tokio::process::Command::new("yt-dlp");
        cmd.args(["--flat-playlist", "-J", "--no-warnings"]);
        if let Some(c) = self.cookies_file() {
            cmd.arg("--cookies").arg(c);
        }
        let out = cmd
            .arg(format!("https://www.youtube.com/playlist?list={id}"))
            .output()
            .await
            .ok()?;
        ytdlp_tracks(&serde_json::from_slice(&out.stdout).ok()?)
    }

    /// The Home feed, personalized when logged in (generic otherwise).
    pub async fn home(&self) -> Result<Vec<Shelf>, Error> {
        home::fetch(&self.dir, self.cookies_file().as_deref())
            .await
            .map_err(|e| Error::Other(e.into()))
    }

    pub async fn logged_in(&self) -> bool {
        self.rp.user_auth_check_cookie().await.is_ok()
    }

    /// Log in with a Netscape `cookies.txt` exported from a music.youtube.com session.
    /// A private copy is kept for yt-dlp: some tracks only stream when signed in.
    pub async fn login(&self, cookies_txt: &str) -> Result<(), Error> {
        self.rp.user_auth_set_cookie_txt(cookies_txt).await?;
        write_private(&self.dir.join(COOKIES), cookies_txt)
            .map_err(|e| Error::Other(format!("saving cookies: {e}").into()))
    }

    pub async fn logout(&self) -> Result<(), Error> {
        let _ = std::fs::remove_file(self.dir.join(COOKIES));
        self.rp.user_auth_remove_cookie().await
    }

    /// cookies.txt for yt-dlp, when logged in.
    pub fn cookies_file(&self) -> Option<PathBuf> {
        Some(self.dir.join(COOKIES)).filter(|p| p.exists())
    }

    pub async fn library(&self) -> Result<Library, Error> {
        let q = self.rp.query();
        let (playlists, albums) = tokio::join!(q.music_saved_playlists(), q.music_saved_albums());
        Ok(Library {
            // "LM" is Liked Music, which callers show through `liked()`;
            // "SE" (Episodes for Later) and podcasts are not music.
            playlists: playlists?
                .items
                .into_iter()
                .filter(|p| p.id != "LM" && p.id != "SE" && !p.is_podcast)
                .map(playlist)
                .collect(),
            albums: albums?.items.into_iter().map(album).collect(),
        })
    }

    pub async fn liked(&self) -> Result<Vec<Track>, Error> {
        let q = self.rp.query();
        let mut p = q.music_liked_tracks().await?;
        p.tracks.extend_limit(&q, MAX_TRACKS).await?;
        Ok(p.tracks.items.into_iter().map(track).collect())
    }

    pub async fn lyrics(&self, uri: &str) -> Result<Option<String>, Error> {
        let Some(id) = video_id(uri) else {
            return Ok(None);
        };
        let q = self.rp.query();
        let details = q.music_details(id).await?;
        match details.lyrics_id {
            Some(id) => Ok(Some(q.music_lyrics(id).await?.body)),
            None => Ok(None),
        }
    }
}

fn video_id(uri: &str) -> Option<&str> {
    uri.strip_prefix("https://music.youtube.com/watch?v=")
        .map(|query| query.split(['&', '#']).next().unwrap_or_default())
        .filter(|id| !id.is_empty())
}

pub fn watch_url(video_id: &str) -> String {
    format!("https://music.youtube.com/watch?v={video_id}")
}

/// Largest thumbnail up to 544px (YouTube Music's own card size), else the smallest.
fn best_thumb(thumbs: &[Thumbnail]) -> Option<String> {
    pick_thumb(thumbs.iter().map(|t| (t.width, t.url.as_str())))
}

pub(crate) fn pick_thumb<'a>(
    thumbs: impl Iterator<Item = (u32, &'a str)> + Clone,
) -> Option<String> {
    thumbs
        .clone()
        .filter(|(w, _)| *w <= 544)
        .max_by_key(|(w, _)| *w)
        .or_else(|| thumbs.min_by_key(|(w, _)| *w))
        .map(|(_, url)| upscale(url))
}

/// googleusercontent art takes its size from the URL (`=w120-h120-...`); ask for 544px.
fn upscale(url: &str) -> String {
    match url.rfind("=w") {
        Some(i) if url.contains("googleusercontent.com") => {
            let rest = &url[i..];
            let opts = rest
                .find('-')
                .and_then(|a| rest[a + 1..].find('-').map(|b| &rest[a + 1 + b..]));
            format!("{}=w544-h544{}", &url[..i], opts.unwrap_or(""))
        }
        _ => url.to_string(),
    }
}

fn names(artists: &[ArtistId]) -> String {
    artists
        .iter()
        .map(|a| a.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn track(t: TrackItem) -> Track {
    Track {
        title: t.name,
        artist: names(&t.artists),
        album_id: t.album.as_ref().map(|a| a.id.clone()),
        artist_id: t.artists.iter().find_map(|a| a.id.clone()),
        album: t.album.map(|a| a.name).unwrap_or_default(),
        uri: watch_url(&t.id),
        track_no: t.track_nr.map(u32::from),
        duration: t.duration.map(f64::from),
        cover: best_thumb(&t.cover),
        ..Default::default()
    }
}

fn album(a: AlbumItem) -> Collection {
    use rustypipe::model::AlbumType;
    let kind = match a.album_type {
        AlbumType::Ep => "EP · ",
        AlbumType::Single => "Sencillo · ",
        _ => "",
    };
    Collection {
        kind: Kind::Album,
        subtitle: match a.year {
            Some(y) => format!("{kind}{} · {y}", names(&a.artists)),
            None => format!("{kind}{}", names(&a.artists)),
        },
        id: a.id,
        title: a.name,
        cover: best_thumb(&a.cover),
    }
}

/// Tracks from `yt-dlp --flat-playlist -J` output.
fn ytdlp_tracks(json: &serde_json::Value) -> Option<Vec<Track>> {
    let entries = json["entries"].as_array()?;
    Some(
        entries
            .iter()
            .filter_map(|e| {
                let artist = e["channel"]
                    .as_str()
                    .or(e["uploader"].as_str())
                    .unwrap_or("");
                Some(Track {
                    title: e["title"].as_str()?.to_string(),
                    artist: artist.trim_end_matches(" - Topic").to_string(),
                    album: String::new(),
                    uri: watch_url(e["id"].as_str()?),
                    track_no: None,
                    duration: e["duration"].as_f64(),
                    cover: e["thumbnails"]
                        .as_array()
                        .and_then(|t| t.last())
                        .and_then(|t| t["url"].as_str())
                        .map(String::from),
                    ..Default::default()
                })
            })
            .collect(),
    )
}

/// An artist whose name matches the query wins, like YouTube Music; else the first song.
fn top_result(query: &str, r: &SearchResults) -> Option<Top> {
    let q = query.trim().to_lowercase();
    r.artists
        .iter()
        .find(|a| a.title.to_lowercase() == q)
        .map(|a| Top::Artist(a.clone()))
        .or_else(|| r.tracks.first().map(|t| Top::Track(t.clone())))
}

fn artist(a: ArtistItem) -> Collection {
    Collection {
        kind: Kind::Artist,
        subtitle: match a.subscriber_count {
            Some(n) => format!("Artista · {} suscriptores", compact_count(n)),
            None => "Artista".into(),
        },
        cover: best_thumb(&a.avatar),
        id: a.id,
        title: a.name,
    }
}

/// 1234567 -> "1.2 M", like YouTube's own counters.
fn compact_count(n: u64) -> String {
    match n {
        0..1_000 => n.to_string(),
        1_000..1_000_000 => format!("{:.1} K", n as f64 / 1e3),
        _ => format!("{:.1} M", n as f64 / 1e6),
    }
    .replace(".0 ", " ")
}

fn playlist(p: MusicPlaylistItem) -> Collection {
    Collection {
        kind: Kind::Playlist,
        subtitle: p.channel.map(|c| c.name).unwrap_or_default(),
        id: p.id,
        title: p.name,
        cover: best_thumb(&p.thumbnail),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_video_id() {
        assert_eq!(
            video_id("https://music.youtube.com/watch?v=abc123"),
            Some("abc123")
        );
        assert_eq!(
            video_id("https://music.youtube.com/watch?v=abc123&list=playlist&index=2"),
            Some("abc123")
        );
        assert_eq!(
            video_id("https://music.youtube.com/watch?v=abc123#lyrics"),
            Some("abc123")
        );
        assert_eq!(video_id("https://example.com/watch?v=abc123"), None);
        assert_eq!(video_id("/home/user/Music/song.flac"), None);
        assert_eq!(video_id("https://music.youtube.com/watch?v="), None);
        assert_eq!(
            video_id("https://music.youtube.com/watch?v=&list=playlist"),
            None
        );
    }

    /// Hits the network; run with `cargo test -p mp-ytmusic -- --ignored`.
    #[tokio::test]
    #[ignore]
    async fn fetches_lyrics() {
        let dir = std::env::temp_dir().join("mp-ytmusic-lyrics-test");
        std::fs::create_dir_all(&dir).unwrap();
        let yt = YtMusic::new(&dir).unwrap();
        let lyrics = yt.lyrics(&watch_url("fJ9rUzIMcZQ")).await.unwrap().unwrap();
        assert!(!lyrics.is_empty());
        assert!(lyrics.contains('\n'));
    }

    #[test]
    fn picks_card_sized_thumbnail() {
        let pick = |ws: &[u32]| {
            let urls: Vec<String> = ws.iter().map(|w| w.to_string()).collect();
            pick_thumb(ws.iter().copied().zip(urls.iter().map(String::as_str)))
        };
        assert_eq!(pick(&[60, 544, 1200]).unwrap(), "544");
        assert_eq!(pick(&[800, 1200]).unwrap(), "800");
        assert_eq!(pick(&[]), None);
    }

    #[test]
    fn top_result_prefers_exact_artist() {
        let c = |t: &str| Collection {
            kind: Kind::Artist,
            id: t.into(),
            title: t.into(),
            subtitle: String::new(),
            cover: None,
        };
        let t = Track {
            title: "Song".into(),
            artist: "X".into(),
            album: String::new(),
            uri: watch_url("a"),
            track_no: None,
            duration: None,
            cover: None,
            ..Default::default()
        };
        let mut r = SearchResults {
            tracks: vec![t],
            artists: vec![c("Opeth Tribute"), c("Opeth")],
            ..Default::default()
        };
        assert!(matches!(top_result(" opeth ", &r), Some(Top::Artist(a)) if a.title == "Opeth"));
        r.artists.pop();
        assert!(matches!(top_result("opeth", &r), Some(Top::Track(_))));
        assert_eq!(compact_count(950), "950");
        assert_eq!(compact_count(1_000_000), "1 M");
        assert_eq!(compact_count(2_340_000), "2.3 M");
    }

    #[test]
    fn parses_ytdlp_flat_playlist() {
        let json = serde_json::json!({ "entries": [
            { "id": "a1", "title": "Sanctus", "duration": 854.0, "channel": "Lacrimosa - Topic",
              "thumbnails": [{ "url": "small" }, { "url": "big" }] },
            { "id": "b2", "title": "No channel" },
            { "title": "missing id is skipped" }
        ]});
        let t = ytdlp_tracks(&json).unwrap();
        assert_eq!(t.len(), 2);
        assert_eq!(
            (t[0].artist.as_str(), t[0].duration, t[0].cover.as_deref()),
            ("Lacrimosa", Some(854.0), Some("big"))
        );
        assert_eq!(t[0].uri, watch_url("a1"));
        assert_eq!(t[1].artist, "");
    }

    #[test]
    fn upscales_google_art_only() {
        assert_eq!(
            upscale("https://lh3.googleusercontent.com/abc=w120-h120-l90-rj"),
            "https://lh3.googleusercontent.com/abc=w544-h544-l90-rj"
        );
        assert_eq!(
            upscale("https://lh3.googleusercontent.com/abc=w60-h60"),
            "https://lh3.googleusercontent.com/abc=w544-h544"
        );
        let yt = "https://i.ytimg.com/vi/x/hqdefault.jpg?w=1";
        assert_eq!(upscale(yt), yt);
    }

    /// Hits the network; run with `cargo test -p mp-ytmusic -- --ignored`.
    #[tokio::test]
    #[ignore]
    async fn anonymous_home_has_shelves() {
        let dir = std::env::temp_dir().join("mp-ytmusic-home-test");
        std::fs::create_dir_all(&dir).unwrap();
        let shelves = YtMusic::new(&dir).unwrap().home().await.unwrap();
        assert!(!shelves.is_empty() && shelves.iter().all(|s| !s.items.is_empty()));
        let tracks: Vec<_> = shelves
            .iter()
            .flat_map(|s| &s.items)
            .filter_map(|i| match i {
                HomeItem::Track(t) => Some(t),
                _ => None,
            })
            .collect();
        let linked = tracks.iter().filter(|t| t.artist_id.is_some()).count();
        eprintln!("home tracks: {}, with artist id: {linked}", tracks.len());
        assert!(
            tracks.is_empty() || linked > 0,
            "home tracks lost their artist links"
        );
    }

    /// Hits the network; run with `cargo test -p mp-ytmusic -- --ignored`.
    #[tokio::test]
    #[ignore]
    async fn search_and_open_album() {
        let dir = std::env::temp_dir().join("mp-ytmusic-test");
        std::fs::create_dir_all(&dir).unwrap();
        let yt = YtMusic::new(&dir).unwrap();
        let r = yt.search("daft punk discovery").await.unwrap();
        assert!(!r.tracks.is_empty() && !r.albums.is_empty());
        assert!(
            r.tracks[0]
                .uri
                .starts_with("https://music.youtube.com/watch?v=")
        );
        let tracks = yt.open(&Kind::Album, &r.albums[0].id).await.unwrap();
        assert!(tracks.len() > 3);
        assert!(
            tracks
                .iter()
                .all(|t| t.cover.is_some() && !t.album.is_empty())
        );
        let album = tracks[0].album_id.as_deref().unwrap();
        assert_eq!(album, r.albums[0].id);
        assert!(tracks[0].artist_id.is_some());
        // Search tracks carry the album browse id too; it resolves to OLAK5uy.
        let browse = r.tracks[0].album_id.as_deref().unwrap();
        let url = yt.album_url(browse).await.unwrap();
        assert!(
            url.starts_with("https://music.youtube.com/playlist?list=OLAK5uy"),
            "{url}"
        );
        assert!(!yt.logged_in().await);
    }
}
