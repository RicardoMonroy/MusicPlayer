//! YouTube Music through rustypipe (Innertube). Playback needs no stream
//! extraction here: mpv resolves `music.youtube.com/watch` URLs with yt-dlp.

use std::path::{Path, PathBuf};

use mp_core::Track;
use rustypipe::{
    client::RustyPipe,
    model::{AlbumItem, ArtistId, MusicPlaylistItem, Thumbnail, TrackItem},
};
use serde::{Deserialize, Serialize};

pub use rustypipe::error::Error;

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
}

/// An album or playlist shown as a card; open it to get its tracks.
#[derive(Debug, Clone, Serialize)]
pub struct Collection {
    pub kind: Kind,
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub cover: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct SearchResults {
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
        let (tracks, albums, featured, community) = tokio::join!(
            q.music_search_tracks(query),
            q.music_search_albums(query),
            q.music_search_playlists(query, false),
            q.music_search_playlists(query, true),
        );
        Ok(SearchResults {
            tracks: tracks?.items.items.into_iter().map(track).collect(),
            albums: albums?.items.items.into_iter().map(album).collect(),
            playlists: featured?
                .items
                .items
                .into_iter()
                .chain(community?.items.items)
                .map(playlist)
                .collect(),
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
                        t
                    })
                    .collect())
            }
            Kind::Playlist => {
                let mut p = q.music_playlist(id).await?;
                p.tracks.extend_limit(&q, MAX_TRACKS).await?;
                Ok(p.tracks.items.into_iter().map(track).collect())
            }
        }
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
            playlists: playlists?.items.into_iter().map(playlist).collect(),
            albums: albums?.items.into_iter().map(album).collect(),
        })
    }

    pub async fn liked(&self) -> Result<Vec<Track>, Error> {
        let q = self.rp.query();
        let mut p = q.music_liked_tracks().await?;
        p.tracks.extend_limit(&q, MAX_TRACKS).await?;
        Ok(p.tracks.items.into_iter().map(track).collect())
    }
}

pub fn watch_url(video_id: &str) -> String {
    format!("https://music.youtube.com/watch?v={video_id}")
}

/// Largest thumbnail up to 544px (YouTube Music's own card size), else the smallest.
fn best_thumb(thumbs: &[Thumbnail]) -> Option<String> {
    pick_thumb(thumbs.iter().map(|t| (t.width, t.url.as_str())))
}

fn pick_thumb<'a>(thumbs: impl Iterator<Item = (u32, &'a str)> + Clone) -> Option<String> {
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
        album: t.album.map(|a| a.name).unwrap_or_default(),
        uri: watch_url(&t.id),
        track_no: t.track_nr.map(u32::from),
        duration: t.duration.map(f64::from),
        cover: best_thumb(&t.cover),
    }
}

fn album(a: AlbumItem) -> Collection {
    Collection {
        kind: Kind::Album,
        subtitle: match a.year {
            Some(y) => format!("{} · {y}", names(&a.artists)),
            None => names(&a.artists),
        },
        id: a.id,
        title: a.name,
        cover: best_thumb(&a.cover),
    }
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
        assert!(!yt.logged_in().await);
    }
}
