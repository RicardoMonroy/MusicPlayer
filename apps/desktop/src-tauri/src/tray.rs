use tauri::{
    AppHandle, Manager,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

use mp_core::player::Player;

/// Show one view (`main` or `compact`) and hide the other.
pub fn show(app: &AppHandle, label: &str) {
    if label == "compact" {
        crate::wm::prepare_compact();
    }
    for w in app.webview_windows().values() {
        if w.label() == label {
            let _ = w.show();
            let _ = w.unminimize();
            let _ = w.set_focus();
        } else {
            let _ = w.hide();
        }
    }
    if label == "compact" {
        crate::wm::restore_compact_size(app);
    }
}

fn toggle_compact(app: &AppHandle) {
    let visible = app
        .get_webview_window("compact")
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false);
    if visible {
        if let Some(w) = app.get_webview_window("compact") {
            let _ = w.hide();
        }
    } else {
        show(app, "compact");
    }
}

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let item = |id: &str, text: &str| MenuItem::with_id(app, id, text, true, None::<&str>);
    let sep = || PredefinedMenuItem::separator(app);
    let menu = Menu::with_items(
        app,
        &[
            &item("main", "Mostrar reproductor")?,
            &item("compact", "Mini reproductor")?,
            &sep()?,
            &item("toggle", "Reproducir / Pausar")?,
            &item("next", "Siguiente")?,
            &item("prev", "Anterior")?,
            &sep()?,
            &item("quit", "Salir")?,
        ],
    )?;

    TrayIconBuilder::with_id("tray")
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("MusicPlayer")
        .menu(&menu)
        // Left click opens the mini player (Windows/macOS); Linux trays only show the menu.
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, e| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = e
            {
                toggle_compact(tray.app_handle());
            }
        })
        .on_menu_event(|app, e| {
            let p = app.state::<Player>();
            let _ = match e.id.as_ref() {
                "main" | "compact" => {
                    show(app, e.id.as_ref());
                    Ok(())
                }
                "toggle" => p.toggle_pause(),
                "next" => p.next(),
                "prev" => p.prev(),
                "quit" => {
                    app.exit(0);
                    Ok(())
                }
                _ => Ok(()),
            };
        })
        .build(app)?;
    Ok(())
}
