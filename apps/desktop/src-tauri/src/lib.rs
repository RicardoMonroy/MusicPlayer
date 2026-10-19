mod media;
mod translate;
mod tray;
mod wm;

use std::{path::PathBuf, sync::Mutex};

use mp_core::{Track, player::Player};
use mp_ytmusic::{ArtistPage, Kind, Library, SearchResults, Shelf, YtMusic};
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent, ipc::Response};

/// What mpv is playing, in order; `Status::index` points into it.
pub type Queue = Mutex<Vec<Track>>;
type CmdResult<T = ()> = Result<T, String>;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// The shared music folder (also used by the TUI).
#[tauri::command]
fn music_folder() -> Option<PathBuf> {
    mp_core::config::music_folder()
}

/// Scan `path` and remember it as the music folder.
#[tauri::command]
async fn scan_folder(path: PathBuf, app: AppHandle) -> CmdResult<Vec<Track>> {
    mp_core::config::set_music_folder(&path).map_err(err)?;
    tauri::async_runtime::spawn_blocking(move || match mp_core::config::library_db() {
        Some(db) => mp_local::scan_cached_with_progress(&path, &db, |done, total| {
            let _ = app.emit("scan-progress", (done, total));
        }),
        None => mp_local::scan(&path),
    })
    .await
    .map_err(err)
}

/// Small cached cover for album grids; empty when the album has no art.
#[tauri::command]
async fn thumbnail(uri: String) -> Response {
    let bytes = tauri::async_runtime::spawn_blocking(move || {
        let dir = dirs::cache_dir()?.join("musicplayer").join("thumbs");
        mp_local::thumbnail(uri.as_ref(), &dir)
    })
    .await
    .ok()
    .flatten()
    .unwrap_or_default();
    Response::new(bytes)
}

/// Replace the queue with `tracks` and start at `index`.
#[tauri::command]
fn play(
    tracks: Vec<Track>,
    index: usize,
    queue: State<Queue>,
    player: State<Player>,
    app: AppHandle,
) -> CmdResult {
    let uris: Vec<String> = tracks.iter().map(|t| t.uri.clone()).collect();
    player.play_queue(&uris, index).map_err(err)?;
    let _ = app.emit("queue", &tracks);
    *queue.lock().unwrap() = tracks;
    Ok(())
}

/// The current queue, for a window that just opened (later changes arrive as `queue` events).
#[tauri::command]
fn queue(queue: State<Queue>) -> Vec<Track> {
    queue.lock().unwrap().clone()
}

/// Play queue entry `index`.
#[tauri::command]
fn jump(index: usize, player: State<Player>) -> CmdResult {
    player.jump(index).map_err(err)
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

#[tauri::command]
fn skin() -> Option<String> {
    mp_core::config::skin()
}

/// Remember the global skin and tell every window.
#[tauri::command]
fn set_skin(id: String, app: AppHandle) -> CmdResult {
    mp_core::config::set_skin(&id).map_err(err)?;
    let _ = app.emit("skin", &id);
    Ok(())
}

#[tauri::command]
fn equalizer() -> mp_core::eq::Equalizer {
    mp_core::config::equalizer()
}

#[tauri::command]
fn eq_presets() -> Vec<(&'static str, [f32; 10])> {
    mp_core::eq::PRESETS.to_vec()
}

/// Apply, remember and broadcast equalizer settings (every window shows them).
#[tauri::command]
fn set_equalizer(eq: mp_core::eq::Equalizer, player: State<Player>, app: AppHandle) -> CmdResult {
    let eq = eq.clamped();
    player.set_equalizer(&eq).map_err(err)?;
    let _ = app.emit("equalizer", &eq);
    mp_core::config::set_equalizer(&eq).map_err(err)
}

#[tauri::command]
fn stop(player: State<Player>) -> CmdResult {
    player.stop().map_err(err)
}

#[tauri::command]
fn set_repeat(mode: String, player: State<Player>) -> CmdResult {
    player.set_repeat(&mode).map_err(err)
}

/// Shuffle or restore the queue, then re-sync our copy with mpv's new order.
#[tauri::command]
fn set_shuffle(on: bool, player: State<Player>, queue: State<Queue>, app: AppHandle) -> CmdResult {
    player.set_shuffle(on).map_err(err)?;
    let mut q = queue.lock().unwrap();
    let mut pool: Vec<Option<Track>> = q.drain(..).map(Some).collect();
    *q = player
        .playlist_uris()
        .iter()
        .filter_map(|uri| {
            pool.iter_mut()
                .find(|t| t.as_ref().is_some_and(|t| &t.uri == uri))
                .and_then(Option::take)
        })
        .collect();
    let _ = app.emit("queue", &*q);
    Ok(())
}

#[tauri::command]
fn replaygain() -> String {
    mp_core::config::replaygain()
}

/// Set and remember the ReplayGain mode ("no", "track", "album").
#[tauri::command]
fn set_replaygain(mode: String, player: State<Player>) -> CmdResult {
    if !["no", "track", "album"].contains(&mode.as_str()) {
        return Err(format!("modo de ReplayGain inválido: {mode}"));
    }
    player.set_replaygain(&mode).map_err(err)?;
    mp_core::config::set_replaygain(&mode).map_err(err)
}

/// Size the mini player window (skin + optional playlist).
#[tauri::command]
fn resize_compact(width: f64, height: f64, app: AppHandle) {
    wm::resize_compact(&app, width, height);
}

/// Switch between the full (`main`) and `compact` views.
#[tauri::command]
fn show_view(label: String, app: AppHandle) {
    tray::show(&app, &label);
}

/// Raw cover bytes, empty when the track has none. Remote art is fetched here
/// so the webview can read its pixels for the accent color (no CORS taint).
#[tauri::command]
async fn cover(track: Track) -> Response {
    let bytes = match track.cover {
        Some(url) => fetch(&url).await.unwrap_or_default(),
        None => tauri::async_runtime::spawn_blocking(move || {
            mp_local::cover(track.uri.as_ref())
                .map(|(_, b)| b)
                .unwrap_or_default()
        })
        .await
        .unwrap_or_default(),
    };
    Response::new(bytes)
}

async fn fetch(url: &str) -> reqwest::Result<Vec<u8>> {
    Ok(reqwest::get(url)
        .await?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec())
}

/// Plain-text lyrics for any track, or None.
#[tauri::command]
async fn lyrics(track: Track, yt: State<'_, YtMusic>) -> CmdResult<Option<String>> {
    if track.uri.starts_with("http") {
        return yt.lyrics(&track.uri).await.map_err(err);
    }
    tauri::async_runtime::spawn_blocking(move || mp_local::lyrics(track.uri.as_ref()))
        .await
        .map_err(err)
}

/// Spanish translation of a track's lyrics (cached; original if already Spanish).
#[tauri::command]
async fn translate_lyrics(uri: String, text: String) -> CmdResult<String> {
    translate::to_spanish(&uri, &text).await
}

#[tauri::command]
async fn yt_search(query: String, yt: State<'_, YtMusic>) -> CmdResult<SearchResults> {
    yt.search(&query).await.map_err(err)
}

#[tauri::command]
async fn yt_open(kind: Kind, id: String, yt: State<'_, YtMusic>) -> CmdResult<Vec<Track>> {
    yt.open(&kind, &id).await.map_err(err)
}

/// YouTube Music's Home feed (personalized when logged in).
#[tauri::command]
async fn yt_home(yt: State<'_, YtMusic>) -> CmdResult<Vec<Shelf>> {
    yt.home().await.map_err(err)
}

#[tauri::command]
async fn yt_artist(id: String, yt: State<'_, YtMusic>) -> CmdResult<ArtistPage> {
    yt.artist(&id).await.map_err(err)
}

#[tauri::command]
async fn yt_logged_in(yt: State<'_, YtMusic>) -> CmdResult<bool> {
    Ok(yt.logged_in().await)
}

/// Log in from a cookies.txt file chosen by the user.
#[tauri::command]
async fn yt_login(path: PathBuf, yt: State<'_, YtMusic>, player: State<'_, Player>) -> CmdResult {
    let txt = std::fs::read_to_string(&path).map_err(err)?;
    yt.login(&txt).await.map_err(err)?;
    player
        .set_ytdl_cookies(yt.cookies_file().as_deref())
        .map_err(err)
}

#[tauri::command]
async fn yt_logout(yt: State<'_, YtMusic>, player: State<'_, Player>) -> CmdResult {
    yt.logout().await.map_err(err)?;
    player.set_ytdl_cookies(None).map_err(err)
}

#[tauri::command]
async fn yt_library(yt: State<'_, YtMusic>) -> CmdResult<Library> {
    yt.library().await.map_err(err)
}

#[tauri::command]
async fn yt_liked(yt: State<'_, YtMusic>) -> CmdResult<Vec<Track>> {
    yt.liked().await.map_err(err)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let player = Player::new().expect("libmpv failed to start; is mpv installed?");
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(player)
        .manage(Queue::default())
        .setup(|app| {
            let dir = app.path().app_data_dir()?.join("ytmusic");
            std::fs::create_dir_all(&dir)?;
            let yt = YtMusic::new(&dir).map_err(|e| e.to_string())?;
            app.state::<Player>()
                .set_ytdl_cookies(yt.cookies_file().as_deref())?;
            app.manage(yt);
            tray::build(app.handle())?;
            media::spawn(app.handle().clone());
            media::spawn_spectrum(app.handle().clone());
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
            queue,
            jump,
            music_folder,
            play,
            toggle_pause,
            next,
            prev,
            seek,
            set_volume,
            now,
            show_view,
            resize_compact,
            replaygain,
            set_replaygain,
            skin,
            set_skin,
            equalizer,
            eq_presets,
            set_equalizer,
            stop,
            set_repeat,
            set_shuffle,
            cover,
            thumbnail,
            lyrics,
            translate_lyrics,
            yt_search,
            yt_open,
            yt_artist,
            yt_home,
            yt_logged_in,
            yt_login,
            yt_logout,
            yt_library,
            yt_liked
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_, event| {
            // Background threads keep using mpv until the process ends; exiting here
            // skips the destructor that would free mpv under them.
            if let tauri::RunEvent::Exit = event {
                std::process::exit(0);
            }
        });
}
