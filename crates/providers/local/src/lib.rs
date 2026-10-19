use std::{fs, path::Path};

use lofty::{
    file::{AudioFile, TaggedFileExt},
    picture::PictureType,
    tag::Accessor,
};
use mp_core::Track;
use walkdir::WalkDir;

fn is_audio(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            [
                "mp3", "flac", "ogg", "opus", "m4a", "aac", "wav", "wv", "ape",
            ]
            .iter()
            .any(|allowed| ext.eq_ignore_ascii_case(allowed))
        })
}

/// Recursively scan audio files, retaining files whose metadata cannot be read.
pub fn scan(root: &Path) -> Vec<Track> {
    let Ok(root) = std::path::absolute(root) else {
        return Vec::new();
    };
    let mut tracks: Vec<_> = WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file() && is_audio(entry.path()))
        .map(|entry| read_track(entry.path()))
        .collect();
    sort_tracks(&mut tracks);
    tracks
}

/// Discography order: album artist, then albums by year, then disc and track.
fn sort_tracks(tracks: &mut [Track]) {
    tracks.sort_by(|a, b| {
        let key = |t: &Track| {
            (
                t.album_artist
                    .clone()
                    .unwrap_or_else(|| t.artist.clone())
                    .to_lowercase(),
                t.year,
                t.album.to_lowercase(),
                t.disc_no,
                t.track_no,
                t.title.clone(),
            )
        };
        key(a).cmp(&key(b))
    });
}

fn read_track(path: &Path) -> Track {
    let file = lofty::read_from_path(path).ok();
    let tag = file
        .as_ref()
        .and_then(|file| file.primary_tag().or_else(|| file.first_tag()));
    Track {
        title: tag
            .and_then(|tag| tag.title())
            .map(|s| s.into_owned())
            .unwrap_or_else(|| {
                path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned()
            }),
        artist: tag
            .and_then(|tag| tag.artist())
            .map(|s| s.into_owned())
            .unwrap_or_else(|| "Artista desconocido".into()),
        album: tag
            .and_then(|tag| tag.album())
            .map(|s| s.into_owned())
            .unwrap_or_else(|| "Álbum desconocido".into()),
        uri: path.to_string_lossy().into_owned(),
        track_no: tag.and_then(|tag| tag.track()),
        duration: file
            .as_ref()
            .map(|file| file.properties().duration().as_secs_f64()),
        cover: None,
        album_artist: tag.and_then(|tag| {
            tag.get_string(lofty::tag::ItemKey::AlbumArtist)
                .map(str::to_owned)
        }),
        year: tag.and_then(|tag| tag.date()).map(|d| u32::from(d.year)),
        disc_no: tag.and_then(|tag| tag.disk()),
        genre: tag.and_then(|tag| tag.genre()).map(|g| g.into_owned()),
    }
}

/// Scan a library, reusing tags when file modification time and size match.
pub fn scan_cached(root: &Path, db: &Path) -> Vec<Track> {
    scan_cached_with_progress(root, db, |_, _| {})
}

/// Like `scan_cached`, calling `progress(done, total)` as files are read, so a UI
/// can show how far a first (slow, tag-reading) scan has got.
pub fn scan_cached_with_progress(
    root: &Path,
    db: &Path,
    mut progress: impl FnMut(usize, usize),
) -> Vec<Track> {
    cached(root, db, &mut progress).unwrap_or_else(|| scan(root))
}

fn cached(root: &Path, db: &Path, progress: &mut dyn FnMut(usize, usize)) -> Option<Vec<Track>> {
    use rusqlite::{Connection, OptionalExtension, params};

    let root = std::path::absolute(root).ok()?;
    if let Some(parent) = db.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        fs::create_dir_all(parent).ok()?;
    }
    let mut conn = Connection::open(db).ok()?;
    conn.execute_batch(
        // v3 adds genre (v2: album_artist, year, disc_no); old tables are just left behind.
        "DROP TABLE IF EXISTS tracks; DROP TABLE IF EXISTS tracks_v2;
        CREATE TABLE IF NOT EXISTS tracks_v3 (
            path TEXT PRIMARY KEY, mtime TEXT NOT NULL, size TEXT NOT NULL,
            title TEXT NOT NULL, artist TEXT NOT NULL, album TEXT NOT NULL,
            track_no INTEGER, duration REAL, cover TEXT,
            album_artist TEXT, year INTEGER, disc_no INTEGER, genre TEXT
        );",
    )
    .ok()?;
    let tx = conn.transaction().ok()?;
    let mut tracks = Vec::new();
    // Listing first is cheap and gives the total for progress reporting.
    let files: Vec<_> = WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file() && is_audio(e.path()))
        .collect();
    let total = files.len();
    progress(0, total);
    for (done, entry) in files.iter().enumerate() {
        if done % 25 == 0 {
            progress(done, total);
        }
        let path = entry.path();
        let uri = path.to_string_lossy().into_owned();
        let fingerprint = fs::metadata(path).ok().and_then(|metadata| {
            Some((
                format!("{:?}", metadata.modified().ok()?),
                metadata.len().to_string(),
            ))
        });
        let Some((mtime, size)) = fingerprint else {
            tracks.push(read_track(path));
            continue;
        };
        let track = tx
            .query_row(
                "SELECT title, artist, album, track_no, duration, cover, album_artist, year, disc_no,
             genre FROM tracks_v3 WHERE path = ?1 AND mtime = ?2 AND size = ?3",
                params![uri, mtime, size],
                |row| {
                    Ok(Track {
                        uri: uri.clone(),
                        title: row.get(0)?,
                        artist: row.get(1)?,
                        album: row.get(2)?,
                        track_no: row.get(3)?,
                        duration: row.get(4)?,
                        cover: row.get(5)?,
                        album_artist: row.get(6)?,
                        year: row.get(7)?,
                        disc_no: row.get(8)?,
                        genre: row.get(9)?,
                    })
                },
            )
            .optional()
            .ok()?;
        let track = match track {
            Some(track) => track,
            None => {
                let track = read_track(path);
                tx.execute(
                    "INSERT OR REPLACE INTO tracks_v3
                     (path, mtime, size, title, artist, album, track_no, duration, cover,
                      album_artist, year, disc_no, genre)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                    params![
                        uri,
                        mtime,
                        size,
                        track.title,
                        track.artist,
                        track.album,
                        track.track_no,
                        track.duration,
                        track.cover,
                        track.album_artist,
                        track.year,
                        track.disc_no,
                        track.genre
                    ],
                )
                .ok()?;
                track
            }
        };
        tracks.push(track);
    }
    let paths = {
        let mut statement = tx.prepare("SELECT path FROM tracks_v3").ok()?;
        statement
            .query_map([], |row| row.get::<_, String>(0))
            .ok()?
            .collect::<Result<Vec<_>, _>>()
            .ok()?
    };
    for path in paths {
        let file = Path::new(&path);
        if file.starts_with(&root) && file.try_exists().ok() == Some(false) {
            tx.execute("DELETE FROM tracks_v3 WHERE path = ?1", [&path])
                .ok()?;
        }
    }
    tx.commit().ok()?;
    progress(total, total);
    sort_tracks(&mut tracks);
    Some(tracks)
}

/// Embedded unsynced lyrics, else a sibling `.lrc`/`.txt` with LRC timestamps stripped.
pub fn lyrics(path: &Path) -> Option<String> {
    use lofty::tag::ItemKey;
    let embedded = lofty::read_from_path(path).ok().and_then(|f| {
        f.tags().iter().find_map(|t| {
            t.get_string(ItemKey::Lyrics)
                .or(t.get_string(ItemKey::UnsyncLyrics))
                .map(str::to_owned)
        })
    });
    embedded
        .or_else(|| {
            ["lrc", "txt"]
                .iter()
                .find_map(|ext| fs::read_to_string(path.with_extension(ext)).ok())
        })
        .map(|text| strip_lrc(&text))
        .filter(|text| !text.trim().is_empty())
}

/// Drop `[mm:ss.xx]` timestamps and `[ar:...]`-style header lines.
fn strip_lrc(text: &str) -> String {
    text.lines()
        .filter_map(|line| {
            let mut rest = line.trim_start();
            while let Some(after) = rest.strip_prefix('[') {
                let (tag, tail) = after.split_once(']')?;
                if !tag.starts_with(|c: char| c.is_ascii_digit()) {
                    return None;
                }
                rest = tail;
            }
            Some(rest.trim())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// A 300 px JPEG of the track's cover, cached in `cache_dir` by album folder so a
/// grid of albums never holds full-size embedded art (often several MB each).
pub fn thumbnail(path: &Path, cache_dir: &Path) -> Option<Vec<u8>> {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    path.parent()?.hash(&mut h);
    let cached = cache_dir.join(format!("{:016x}.jpg", h.finish()));
    if let Ok(bytes) = fs::read(&cached) {
        return Some(bytes);
    }
    let (_, original) = cover(path)?;
    let img = image::load_from_memory(&original).ok()?.thumbnail(300, 300);
    let mut out = std::io::Cursor::new(Vec::new());
    img.into_rgb8()
        .write_to(&mut out, image::ImageFormat::Jpeg)
        .ok()?;
    let bytes = out.into_inner();
    let _ = fs::create_dir_all(cache_dir).and_then(|_| fs::write(&cached, &bytes));
    Some(bytes)
}

/// Read embedded artwork, then try conventional cover filenames beside the file.
pub fn cover(path: &Path) -> Option<(String, Vec<u8>)> {
    if let Ok(file) = lofty::read_from_path(path) {
        let pictures: Vec<_> = file.tags().iter().flat_map(|tag| tag.pictures()).collect();
        if let Some(picture) = pictures
            .iter()
            .find(|picture| picture.pic_type() == PictureType::CoverFront)
            .or_else(|| pictures.first())
        {
            return Some((
                picture
                    .mime_type()
                    .map(|mime| mime.to_string())
                    .unwrap_or_else(|| "application/octet-stream".into()),
                picture.data().to_vec(),
            ));
        }
    }
    let entries: Vec<_> = fs::read_dir(path.parent()?)
        .ok()?
        .filter_map(Result::ok)
        .collect();
    for name in ["cover.jpg", "cover.png", "folder.jpg", "front.jpg"] {
        for entry in &entries {
            if entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case(name)
                && let Ok(bytes) = fs::read(entry.path())
            {
                let mime = if name.ends_with(".png") {
                    "image/png"
                } else {
                    "image/jpeg"
                };
                return Some((mime.into(), bytes));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_populates_reuses_refreshes_and_removes() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("music");
        fs::create_dir(&root).unwrap();
        let path = root.join("song.mp3");
        fs::write(&path, b"not audio").unwrap();
        let db = dir.path().join("cache/library.sqlite");
        assert_eq!(scan_cached(&root, &db), scan(&root));
        let conn = rusqlite::Connection::open(&db).unwrap();
        assert_eq!(
            conn.query_row("SELECT count(*) FROM tracks_v3", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
        conn.execute("UPDATE tracks_v3 SET title = 'cached title'", [])
            .unwrap();
        assert_eq!(scan_cached(&root, &db)[0].title, "cached title");
        fs::write(&path, b"changed file with different size").unwrap();
        assert_eq!(scan_cached(&root, &db), scan(&root));
        fs::remove_file(&path).unwrap();
        assert!(scan_cached(&root, &db).is_empty());
        assert_eq!(
            conn.query_row("SELECT count(*) FROM tracks_v3", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn unusable_cache_falls_back_to_scan() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("song.mp3"), b"not audio").unwrap();
        let db = dir.path().join("garbage.sqlite");
        fs::write(&db, b"not a database").unwrap();
        assert_eq!(scan_cached(dir.path(), &db), scan(dir.path()));
        assert_eq!(scan_cached(dir.path(), dir.path()), scan(dir.path()));
    }

    #[test]
    fn lyrics_from_sibling_lrc() {
        let dir = tempfile::tempdir().unwrap();
        let song = dir.path().join("x.mp3");
        fs::write(&song, b"not audio").unwrap();
        assert_eq!(lyrics(&song), None);
        fs::write(
            dir.path().join("x.lrc"),
            "[ar:Someone]\n[00:01.00]First line\n[00:02.50][00:09.00]Chorus\nplain",
        )
        .unwrap();
        assert_eq!(lyrics(&song).unwrap(), "First line\nChorus\nplain");
    }

    #[test]
    fn extension_filter() {
        for ext in [
            "mp3", "FLAC", "OgG", "opus", "m4a", "aac", "wav", "wv", "ape",
        ] {
            assert!(is_audio(Path::new(&format!("song.{ext}"))));
        }
        for name in ["song", "song.txt", "song.mp3.bak", "mp3"] {
            assert!(!is_audio(Path::new(name)));
        }
    }

    #[test]
    fn unreadable_metadata_uses_fallbacks() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("nested");
        fs::create_dir(&nested).unwrap();
        let path = nested.join("x.mp3");
        fs::write(&path, b"not audio").unwrap();
        fs::write(nested.join("ignored.txt"), b"not audio").unwrap();
        assert_eq!(
            scan(dir.path()),
            vec![Track {
                title: "x".into(),
                artist: "Artista desconocido".into(),
                album: "Álbum desconocido".into(),
                uri: path.to_string_lossy().into_owned(),
                track_no: None,
                duration: None,
                cover: None,
                ..Default::default()
            }]
        );
    }

    #[test]
    fn external_cover_is_case_insensitive() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x.mp3");
        assert_eq!(cover(&path), None);
        fs::write(dir.path().join("CoVeR.PnG"), b"picture").unwrap();
        assert_eq!(
            cover(&path),
            Some(("image/png".into(), b"picture".to_vec()))
        );
    }
}
