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
        .map(|entry| {
            let path = entry.path();
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
            }
        })
        .collect();
    tracks.sort_by(|a, b| {
        (&a.artist, &a.album, a.track_no, &a.title)
            .cmp(&(&b.artist, &b.album, b.track_no, &b.title))
    });
    tracks
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
