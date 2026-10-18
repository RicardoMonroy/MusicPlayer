//! Shared engine for desktop and TUI. Playback (libmpv) lands in M1.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub album: String,
    /// Local path or stream URL, whatever the player can open.
    pub uri: String,
    pub track_no: Option<u32>,
    /// Seconds.
    pub duration: Option<f64>,
    /// Remote cover art URL; local files read embedded art instead.
    pub cover: Option<String>,
}

pub const APP_NAME: &str = "MusicPlayer";

pub mod player;
