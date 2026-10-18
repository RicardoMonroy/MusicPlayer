//! Settings shared by the desktop app and the TUI, as small files in the config dir.

use std::path::PathBuf;

fn file(name: &str) -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("musicplayer").join(name))
}

/// The local music folder, if one was chosen.
pub fn music_folder() -> Option<PathBuf> {
    // "tui-folder": where TUI builds stored it before this setting was shared.
    let s = ["music-folder", "tui-folder"]
        .into_iter()
        .find_map(|name| std::fs::read_to_string(file(name)?).ok())?;
    Some(PathBuf::from(s.trim())).filter(|p| !p.as_os_str().is_empty())
}

pub fn set_music_folder(folder: &std::path::Path) -> std::io::Result<()> {
    let f = file("music-folder").ok_or(std::io::ErrorKind::NotFound)?;
    std::fs::create_dir_all(f.parent().unwrap())?;
    std::fs::write(f, folder.to_string_lossy().as_bytes())
}

/// SQLite cache of local tags; safe to delete, it is rebuilt on the next scan.
pub fn library_db() -> Option<PathBuf> {
    Some(
        dirs::cache_dir()?
            .join("musicplayer")
            .join("library.sqlite"),
    )
}

/// ReplayGain mode for mpv: "no", "track" or "album". Defaults to "track".
pub fn replaygain() -> String {
    file("replaygain")
        .and_then(|f| std::fs::read_to_string(f).ok())
        .map(|s| s.trim().to_string())
        .filter(|m| ["no", "track", "album"].contains(&m.as_str()))
        .unwrap_or_else(|| "track".into())
}

pub fn set_replaygain(mode: &str) -> std::io::Result<()> {
    let f = file("replaygain").ok_or(std::io::ErrorKind::NotFound)?;
    std::fs::create_dir_all(f.parent().unwrap())?;
    std::fs::write(f, mode)
}

pub fn equalizer() -> crate::eq::Equalizer {
    file("equalizer.json")
        .and_then(|f| std::fs::read(f).ok())
        .and_then(|b| serde_json::from_slice::<crate::eq::Equalizer>(&b).ok())
        .map(crate::eq::Equalizer::clamped)
        .unwrap_or_default()
}

pub fn set_equalizer(eq: &crate::eq::Equalizer) -> std::io::Result<()> {
    let f = file("equalizer.json").ok_or(std::io::ErrorKind::NotFound)?;
    std::fs::create_dir_all(f.parent().unwrap())?;
    std::fs::write(f, serde_json::to_vec(eq)?)
}

/// Global skin id chosen in the desktop app (the TUI picks a matching palette).
pub fn skin() -> Option<String> {
    let s = std::fs::read_to_string(file("skin")?).ok()?;
    Some(s.trim().to_string()).filter(|s| !s.is_empty())
}

pub fn set_skin(id: &str) -> std::io::Result<()> {
    let f = file("skin").ok_or(std::io::ErrorKind::NotFound)?;
    std::fs::create_dir_all(f.parent().unwrap())?;
    std::fs::write(f, id)
}
