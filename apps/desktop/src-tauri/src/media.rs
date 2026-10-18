//! Status poller: pushes `now` events to every window and keeps the OS media
//! controls (MPRIS / SMTC / Now Playing) in sync. Media keys map to the player.

use std::{thread, time::Duration};

use mp_core::{Track, player::Player, player::Status};
use serde::Serialize;
use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
};
use tauri::{AppHandle, Emitter, Manager};

use crate::Queue;

#[derive(Serialize, Clone, PartialEq)]
pub struct Now {
    pub status: Status,
    pub track: Option<Track>,
}

pub fn now(app: &AppHandle) -> Now {
    let status = app.state::<Player>().status();
    let track = status
        .index
        .and_then(|i| app.state::<Queue>().lock().unwrap().get(i).cloned());
    Now { status, track }
}

pub fn spawn(app: AppHandle) {
    // Windows needs the main window handle; pass it as an integer so it can cross threads.
    #[cfg(windows)]
    let hwnd = app
        .get_webview_window("main")
        .and_then(|w| w.hwnd().ok())
        .map(|h| h.0 as isize);

    thread::spawn(move || {
        #[cfg(windows)]
        let hwnd = hwnd.map(|h| h as *mut std::ffi::c_void);
        #[cfg(not(windows))]
        let hwnd = None;

        let mut controls = MediaControls::new(PlatformConfig {
            display_name: "MusicPlayer",
            dbus_name: "musicplayer",
            hwnd,
        })
        .ok();
        if let Some(c) = controls.as_mut() {
            let app = app.clone();
            let _ = c.attach(move |e| on_media_key(&app, e));
        }

        let mut last: Option<Now> = None;
        loop {
            let n = now(&app);
            if last.as_ref() != Some(&n) {
                let _ = app.emit("now", &n);
                if let Some(c) = controls.as_mut() {
                    sync_controls(c, &n, last.as_ref());
                }
                last = Some(n);
            }
            thread::sleep(Duration::from_millis(250));
        }
    });
}

fn sync_controls(c: &mut MediaControls, n: &Now, last: Option<&Now>) {
    let uri = |x: Option<&Now>| x.and_then(|x| x.track.as_ref()).map(|t| t.uri.clone());
    if uri(Some(n)) != uri(last)
        && let Some(t) = &n.track
    {
        let _ = c.set_metadata(MediaMetadata {
            title: Some(&t.title),
            artist: Some(&t.artist),
            album: Some(&t.album),
            duration: t.duration.map(Duration::from_secs_f64),
            cover_url: None,
        });
    }
    let progress = Some(MediaPosition(Duration::from_secs_f64(
        n.status.position.max(0.0),
    )));
    let _ = c.set_playback(match (&n.track, n.status.paused) {
        (None, _) => MediaPlayback::Stopped,
        (Some(_), true) => MediaPlayback::Paused { progress },
        (Some(_), false) => MediaPlayback::Playing { progress },
    });
}

fn on_media_key(app: &AppHandle, e: MediaControlEvent) {
    let p = app.state::<Player>();
    let paused = p.status().paused;
    let _ = match e {
        MediaControlEvent::Toggle => p.toggle_pause(),
        MediaControlEvent::Play if paused => p.toggle_pause(),
        MediaControlEvent::Pause if !paused => p.toggle_pause(),
        MediaControlEvent::Next => p.next(),
        MediaControlEvent::Previous => p.prev(),
        MediaControlEvent::SetPosition(MediaPosition(d)) => p.seek(d.as_secs_f64()),
        MediaControlEvent::Raise => {
            crate::tray::show(app, "main");
            Ok(())
        }
        MediaControlEvent::Quit => {
            app.exit(0);
            Ok(())
        }
        _ => Ok(()),
    };
}
