//! Window-manager hints for the mini player. Tiling compositors would otherwise
//! stretch it into a tile; floating desktops already honour `alwaysOnTop`.

use std::{process::Command, sync::Mutex};

use tauri::{AppHandle, LogicalSize, Manager};

/// Last size the compact page asked for, re-applied whenever the window is shown.
pub static COMPACT_SIZE: Mutex<Option<(f64, f64)>> = Mutex::new(None);

fn hyprland() -> bool {
    std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some()
}

const TITLE: &str = "MusicPlayer Mini";

/// On Hyprland, (re)register a rule that floats and pins the mini player.
/// Named, so registering again replaces it; done before every show because a
/// Hyprland config reload drops runtime rules.
pub fn prepare_compact() {
    if !hyprland() {
        return;
    }
    let lua = format!(
        r#"hl.window_rule({{ name = "musicplayer-mini", match = {{ class = "^(musicplayer-desktop)$", title = "^({TITLE})$" }}, float = true, pin = true, no_blur = true, no_shadow = true, border_size = 0, rounding = 0 }})"#
    );
    let ok = Command::new("hyprctl")
        .args(["eval", &lua])
        .output()
        .is_ok_and(|o| o.status.success() && !String::from_utf8_lossy(&o.stdout).contains("rror"));
    if !ok {
        // Hyprland before 0.55 (hyprlang config, no Lua `eval`).
        let m = format!("title:^({TITLE})$");
        let batch = ["float", "pin", "noblur", "noshadow", "noborder"]
            .map(|r| format!("keyword windowrulev2 {r},{m}"))
            .join(" ; ");
        let _ = Command::new("hyprctl").args(["--batch", &batch]).output();
    }
}

/// Size the compact window, growing/shrinking downwards from its top-left corner.
pub fn resize_compact(app: &AppHandle, width: f64, height: f64) {
    *COMPACT_SIZE.lock().unwrap() = Some((width, height));
    let Some(w) = app.get_webview_window("compact") else {
        return;
    };
    let _ = w.set_size(LogicalSize::new(width, height));
    if !hyprland() || !w.is_visible().unwrap_or(false) {
        return;
    }
    // Hyprland ignores client resizes of floating windows and recenters on resize,
    // so resize through the compositor and put the top-left corner back.
    let Some((x, y)) = hyprland_position() else {
        return;
    };
    let sel = format!("title:^({TITLE})$");
    let lua = format!(
        r#"hl.dispatch(hl.dsp.window.resize({{ x = {width}, y = {height}, window = "{sel}" }})); hl.dispatch(hl.dsp.window.move({{ x = {x}, y = {y}, window = "{sel}" }}))"#
    );
    let _ = Command::new("hyprctl").args(["eval", &lua]).output();
}

fn hyprland_position() -> Option<(i64, i64)> {
    let out = Command::new("hyprctl")
        .args(["clients", "-j"])
        .output()
        .ok()?;
    let clients: Vec<serde_json::Value> = serde_json::from_slice(&out.stdout).ok()?;
    let c = clients.iter().find(|c| c["title"] == TITLE)?;
    Some((c["at"][0].as_i64()?, c["at"][1].as_i64()?))
}

/// Re-apply the last requested size once the window is mapped again.
pub fn restore_compact_size(app: &AppHandle) {
    let Some((w, h)) = *COMPACT_SIZE.lock().unwrap() else {
        return;
    };
    let app = app.clone();
    std::thread::spawn(move || {
        // The compositor maps the window asynchronously after show().
        std::thread::sleep(std::time::Duration::from_millis(150));
        resize_compact(&app, w, h);
    });
}
