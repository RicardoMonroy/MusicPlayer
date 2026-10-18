//! Shared engine for desktop and TUI. Playback (libmpv) lands in M1.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub album: String,
    /// Local path or stream URL, whatever the player can open.
    pub uri: String,
}

pub const APP_NAME: &str = "MusicPlayer";
