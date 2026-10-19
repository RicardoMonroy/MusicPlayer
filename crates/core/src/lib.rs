//! Shared engine for desktop and TUI. Playback (libmpv) lands in M1.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
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
    /// Groups albums with guest artists or various artists under one name.
    #[serde(default)]
    pub album_artist: Option<String>,
    #[serde(default)]
    pub year: Option<u32>,
    #[serde(default)]
    pub disc_no: Option<u32>,
    #[serde(default)]
    pub genre: Option<String>,
}

pub const APP_NAME: &str = "MusicPlayer";

pub mod config;
pub mod eq;
pub mod player;
