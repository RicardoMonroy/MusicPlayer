//! Audio playback on top of libmpv. The queue is mpv's own playlist, so
//! gapless playback, next/prev and end-of-track advance come for free.

use libmpv2::Mpv;
use serde::Serialize;

pub use libmpv2::Error;

#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct Status {
    /// Index into the queue passed to `play_queue`. None when idle.
    pub index: Option<usize>,
    pub paused: bool,
    pub position: f64,
    pub duration: f64,
    pub volume: f64,
}

pub struct Player {
    mpv: Mpv,
}

impl Player {
    pub fn new() -> Result<Self, Error> {
        let mpv = Mpv::with_initializer(|init| {
            init.set_option("vo", "null")?;
            init.set_option("video", "no")?;
            init.set_option("idle", "yes")?;
            init.set_option("gapless-audio", "weak")?;
            init.set_option("audio-display", "no")?;
            // Streams (YouTube via yt-dlp): fetch audio only.
            init.set_option("ytdl-format", "bestaudio/best")?;
            Ok(())
        })?;
        // State is read on demand; mpv's event queue is bounded, undrained events are dropped.
        Ok(Self { mpv })
    }

    /// Replace the queue and start playing `uris[start]`.
    pub fn play_queue(&self, uris: &[String], start: usize) -> Result<(), Error> {
        self.mpv.command("stop", &[])?;
        self.mpv.command("playlist-clear", &[])?;
        for uri in uris {
            self.mpv.command("loadfile", &[uri, "append"])?;
        }
        self.mpv.set_property("pause", false)?;
        self.mpv
            .command("playlist-play-index", &[&start.to_string()])
    }

    /// Cookies for yt-dlp, for YouTube tracks that only stream when signed in. None clears them.
    // shortcut: mpv's key=value list splits on ',', so a path containing ',' breaks; quote it if that shows up.
    pub fn set_ytdl_cookies(&self, path: Option<&std::path::Path>) -> Result<(), Error> {
        let value = path.map(|p| format!("cookies={}", p.display()));
        self.mpv
            .set_property("ytdl-raw-options", value.unwrap_or_default())
    }

    pub fn toggle_pause(&self) -> Result<(), Error> {
        self.mpv.command("cycle", &["pause"])
    }

    pub fn next(&self) -> Result<(), Error> {
        self.mpv.command("playlist-next", &[])
    }

    pub fn prev(&self) -> Result<(), Error> {
        // Restart the track if we are past its first seconds, like most players.
        if self.mpv.get_property::<f64>("time-pos").unwrap_or(0.0) > 3.0 {
            return self.seek(0.0);
        }
        self.mpv.command("playlist-prev", &[])
    }

    pub fn seek(&self, seconds: f64) -> Result<(), Error> {
        self.mpv
            .command("seek", &[&seconds.to_string(), "absolute"])
    }

    /// 0–100.
    pub fn set_volume(&self, volume: f64) -> Result<(), Error> {
        self.mpv.set_property("volume", volume.clamp(0.0, 100.0))
    }

    /// Queue indexes that failed to load since the last call (e.g. a YouTube track
    /// that needs sign-in). mpv already skipped them; this only reports. Call from one thread.
    pub fn take_failures(&self) -> Vec<usize> {
        use libmpv2_sys as sys;
        let mut ids = Vec::new();
        loop {
            // Raw API: libmpv2's wrapper drops end-file's playlist_entry_id, which we need.
            let ev = unsafe { *sys::mpv_wait_event(self.mpv.ctx.as_ptr(), 0.0) };
            match ev.event_id {
                sys::mpv_event_id_MPV_EVENT_NONE => break,
                sys::mpv_event_id_MPV_EVENT_END_FILE => {
                    let end = unsafe { *(ev.data as *const sys::mpv_event_end_file) };
                    if end.reason == sys::mpv_end_file_reason_MPV_END_FILE_REASON_ERROR {
                        ids.push(end.playlist_entry_id);
                    }
                }
                _ => {}
            }
        }
        if ids.is_empty() {
            return ids.into_iter().map(|_| 0).collect();
        }
        let count = self.mpv.get_property::<i64>("playlist-count").unwrap_or(0);
        (0..count)
            .filter(|i| {
                self.mpv
                    .get_property::<i64>(&format!("playlist/{i}/id"))
                    .is_ok_and(|id| ids.contains(&id))
            })
            .map(|i| i as usize)
            .collect()
    }

    pub fn status(&self) -> Status {
        let f = |name| self.mpv.get_property::<f64>(name).unwrap_or(0.0);
        let idle = self.mpv.get_property::<bool>("idle-active").unwrap_or(true);
        Status {
            index: match self.mpv.get_property::<i64>("playlist-pos") {
                Ok(i) if i >= 0 && !idle => Some(i as usize),
                _ => None,
            },
            paused: self.mpv.get_property::<bool>("pause").unwrap_or(false),
            position: f("time-pos"),
            duration: f("duration"),
            volume: f("volume"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time::Duration};

    fn tone(secs: u32) -> String {
        format!("av://lavfi:sine=frequency=440:duration={secs}")
    }

    #[test]
    fn plays_queue_and_advances() {
        let p = Player::new().unwrap();
        p.mpv.set_property("ao", "null").unwrap();
        p.play_queue(&[tone(10), tone(10)], 0).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert_eq!(p.status().index, Some(0));

        p.next().unwrap();
        thread::sleep(Duration::from_millis(500));
        let s = p.status();
        assert_eq!(s.index, Some(1));
        assert!(s.duration > 0.0); // lavfi only estimates duration

        p.toggle_pause().unwrap();
        assert!(p.status().paused);
        p.set_volume(150.0).unwrap();
        assert_eq!(p.status().volume, 100.0);
    }

    #[test]
    fn reports_tracks_that_fail_to_load() {
        let p = Player::new().unwrap();
        p.mpv.set_property("ao", "null").unwrap();
        p.play_queue(&["/nonexistent/a.flac".into(), tone(10)], 0)
            .unwrap();
        thread::sleep(Duration::from_millis(800));
        assert_eq!(p.take_failures(), vec![0]);
        assert_eq!(p.status().index, Some(1), "mpv skips to the next track");
        assert!(p.take_failures().is_empty(), "reported once");
    }
}
