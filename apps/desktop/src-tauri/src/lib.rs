mod media;
mod tray;

use std::{path::PathBuf, sync::Mutex};

use mp_core::{Track, player::Player};
use tauri::{AppHandle, State, WindowEvent, ipc::Response};

pub type Library = Mutex<Vec<Track>>;
type CmdResult<T = ()> = Result<T, String>;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

#[tauri::command]
async fn scan_folder(path: PathBuf, library: State<'_, Library>) -> CmdResult<Vec<Track>> {
    let tracks = tauri::async_runtime::spawn_blocking(move || mp_local::scan(&path))
        .await
        .map_err(err)?;
    *library.lock().unwrap() = tracks.clone();
    Ok(tracks)
}

/// Queue the whole library and start at `index`.
#[tauri::command]
fn play(index: usize, library: State<Library>, player: State<Player>) -> CmdResult {
    let uris: Vec<String> = library
        .lock()
        .unwrap()
        .iter()
        .map(|t| t.uri.clone())
        .collect();
    player.play_queue(&uris, index).map_err(err)
}

#[tauri::command]
fn toggle_pause(player: State<Player>) -> CmdResult {
    player.toggle_pause().map_err(err)
}

#[tauri::command]
fn next(player: State<Player>) -> CmdResult {
    player.next().map_err(err)
}

#[tauri::command]
fn prev(player: State<Player>) -> CmdResult {
    player.prev().map_err(err)
}

#[tauri::command]
fn seek(seconds: f64, player: State<Player>) -> CmdResult {
    player.seek(seconds).map_err(err)
}

#[tauri::command]
fn set_volume(volume: f64, player: State<Player>) -> CmdResult {
    player.set_volume(volume).map_err(err)
}

/// Current state for a window that just opened; later changes arrive as `now` events.
#[tauri::command]
fn now(app: AppHandle) -> media::Now {
    media::now(&app)
}

/// Switch between the full (`main`) and `compact` views.
#[tauri::command]
fn show_view(label: String, app: AppHandle) {
    tray::show(&app, &label);
}

/// Raw cover bytes, empty when the track has none.
#[tauri::command]
async fn cover(uri: String) -> Response {
    let bytes = tauri::async_runtime::spawn_blocking(move || {
        mp_local::cover(uri.as_ref())
            .map(|(_, b)| b)
            .unwrap_or_default()
    })
    .await
    .unwrap_or_default();
    Response::new(bytes)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let player = Player::new().expect("libmpv failed to start; is mpv installed?");
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(player)
        .manage(Library::default())
        .setup(|app| {
            tray::build(app.handle())?;
            media::spawn(app.handle().clone());
            Ok(())
        })
        // Closing a window hides it to the tray; quit from the tray menu.
        .on_window_event(|w, e| {
            if let WindowEvent::CloseRequested { api, .. } = e {
                api.prevent_close();
                let _ = w.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            scan_folder,
            play,
            toggle_pause,
            next,
            prev,
            seek,
            set_volume,
            now,
            show_view,
            cover
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
