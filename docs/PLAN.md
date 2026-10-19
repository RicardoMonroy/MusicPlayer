# Plan — MusicPlayer

Status: stack approved 2026-10-18. M0–M4 done, M5 in progress. License: GPL-3.0-or-later (rustypipe and libmpv are GPL).

## Stack

| Layer | Choice | Why |
|---|---|---|
| Core | Rust workspace | One engine shared by GUI and TUI. |
| Playback | libmpv (`libmpv2` crate) | Plays mp3/flac/ogg/opus/m4a/wav, gapless, ReplayGain, and YouTube streams through yt-dlp. Available on all three OS. |
| Tags/library | `lofty` + SQLite cache (`rusqlite`, bundled; table `tracks_v3` with album artist, year, disc, genre) | Tags re-read only for new/changed files (mtime+size). Cache at `<cache>/musicplayer/library.sqlite`, safe to delete. |
| Desktop UI | Tauri v2 + Svelte + TypeScript | Skins are HTML/CSS/Canvas: easy cassette, vinyl and Winamp looks. Small binaries. |
| Tray / media keys | Tauri tray + popup window; `souvlaki` (MPRIS on Linux, SMTC on Windows, Now Playing on macOS) | Native controls everywhere. |
| TUI | `ratatui` + `crossterm` | Same core, large mouse-clickable controls. |

## Architecture

```
crates/core         Player (libmpv), Queue, Library, events
crates/providers/
  local             folder scan, watch, tags, covers
  ytmusic           search, library, playlists, stream URL (yt-dlp)
  qobuz             later
apps/desktop        Tauri v2: full view, compact view, tray popup
apps/tui            ratatui client
```

Queue = mpv playlist (gapless, auto-advance for free). The UI polls player status ~4x/s. Sources will implement one `Provider` trait (add it with the second provider): `search`, `browse`, `resolve_stream(track) -> url/path`. The player only sees a URL or a path.

## Equalizer

`mp_core::eq`: Winamp's 10 bands (60 Hz–16 kHz) + preamp, ±12 dB, Winamp presets. One lavfi chain `@eq` placed before the visualizer; slider moves use `af-command` (no audio gap), falling back to a rebuild. Saved in `<config>/musicplayer/equalizer.json`, shared by GUI and TUI. Each skin draws its own EQ panel (`Skin.equalizer`). TUI: view 6 "Ecualizador" on the same API and config.

## Local library

Views: Álbumes (default; grouped by album artist + album; sort by artist/year/title/genre, genre filter), Artistas (artist page with discography), Carpetas (folder tree, play a folder recursively), Canciones (table sortable by every column, incl. genre). TUI: same sub-views (`v` cycles), artist view in columns, album view with disc separators, background scan with progress. Album page: disc separators, play / shuffle the whole album. Scans report progress (`scan-progress` event) so a first scan of a big library shows "Escaneando… n / total". Covers in grids are 300 px JPEG thumbnails cached in `<cache>/musicplayer/thumbs`.

## Shared settings

`<config>/musicplayer/music-folder` (`mp_core::config`) is the local music folder for both apps; the TUI's CLI arg updates it.

## Views

- **Full view**: mix of Dopamine (clean library, album grid), Amberol (cover-driven adaptive colors), Tauon (fast browsing, gallery). Dynamic accent color from the cover art.
- **Global skins**: a skin is the mini player + its EQ panel + playlist theme + a `FullTheme` (colors, fonts, radii, bevels) applied as CSS variables to the whole full player, + a TUI palette id. Choice stored in `<config>/musicplayer/skin`, shared with the TUI (re-read every 2 s, `t` cycles). Skins: MusicPlayer (default), Moderno, Minimalista, Clásica (Winamp), Casete, Vinilo, iPod, Windows 95, Hacker, Análogo. Each skin may add a `hero` (280 px signature piece: spinning record, reels, LCD, tubes...) shown in the full player now-playing panel. Settings (⚙ in the sidebar): skin gallery, ReplayGain, equalizer (the active skin's panel); EQ button in the transport bar.
- **Compact view**: small borderless window. Skin is a plugin: `skins/<name>/` with `skin.json` + Svelte component + assets.
  - Initial skins: Cassette (Kaset-like, reels spin with progress), Vinyl (record spins, tonearm moves with progress), Classic (Winamp 2-like, spectrum visualizer).
- **Tray**: closing a window hides it to the tray. Left click toggles the mini player (Windows/macOS). Linux trays (StatusNotifierItem) only deliver menu events, so on Linux the tray menu has Mini player / Play-Pause / Next / Previous / Quit.
- **Media keys**: `souvlaki` (MPRIS on Linux, SMTC on Windows, Now Playing on macOS) fed by a Rust poller that also pushes `now` events to every window.
- **Search (both apps)**: YouTube Music style — top result (exact artist match, else first song), songs (+ videos), artists, albums, playlists; artist page with popular tracks and discography. TUI shows them in columns (2x2 under ~100 cols, single column with switcher under ~60).
- **TUI (Linux first)**: left sidebar (1 Biblioteca, 2 Álbumes, 3 Cola, 4 Buscar, 5 Playlists, account state); dedicated search box. `musicplayer-tui [folder]` (folder remembered in `~/.config/musicplayer/tui-folder`, default ~/Music). Tabs Library / Albums / Queue, `/` search, big clickable Prev/Play/Next buttons, clickable seek and volume bars, mouse wheel, double-click to play, key hints footer. Playlists tab comes with YouTube Music.

## Streaming services — real constraints

- **YouTube Music** (`crates/providers/ytmusic`): metadata via `rustypipe` (search, albums, playlists, saved library, liked). Playback: mpv opens `music.youtube.com/watch?v=` URLs through its yt-dlp hook (`ytdl-format=bestaudio/best`), so **yt-dlp must be on PATH**. Login = cookies.txt exported from a private browser session; stored in rustypipe's cache under the app data dir (`<data>/dev.ricardo.musicplayer/ytmusic`), shared by desktop and TUI. Unofficial API: against YouTube ToS and may break when YouTube changes.
  - Playlists: tried anonymously, then signed in (private lists), then listed with `yt-dlp --flat-playlist` when rustypipe rejects them (old 13-char IDs, mixed "album" playlists). Podcast lists (SE, is_podcast) are hidden.
  - shortcut: yt-dlp does not get the cookies, so private/age-restricted tracks may fail to play; pass `ytdl-raw-options=cookies=<file>` if needed.
- **Qobuz**: partner API exists. Community clients (streamrip, qobuz-dl) use the web player app_id. Hi-res FLAC streams work well with mpv. Do it when the account is active.
- **Amazon Music**: no public playback API and DRM (Widevine). Tauri webviews on Linux (WebKitGTK) do not support Widevine. Options: (a) embed the web player in a separate Electron window with Widevine (castLabs build), (b) control-only integration (now-playing + media keys of the official app), (c) drop it. Recommendation: (b) or postpone.

## Roadmap

1. **M0 scaffold**: Rust workspace, Tauri app, CI build for Linux/Windows/macOS.
2. **M1 local playback**: core + local provider + basic full view + queue.
3. **M2 compact + skins**: compact window, skin system, Cassette/Vinyl/Classic, tray popup, media keys.
4. **M3 TUI**: ratatui client on the same core.
5. **M4 YouTube Music**: search, library, playlists, playback.
6. **M5 polish**: done — skipped-track notices (mpv end-file errors), shared music folder, SQLite tag cache, lyrics (YouTube Music + embedded/.lrc), MPRIS cover art, non-blocking TUI network, responsive TUI buttons, PKGBUILD + release workflow (see packaging/README.md). Real visualizer: 19-band lavfi analyser (`Player::set_visualizer`/`spectrum`), on only while the compact window is visible or the TUI runs; bars run a few hundred ms ahead of the audio (see docs/spikes/visualizer.md). ReplayGain (`<config>/musicplayer/replaygain`: no|track|album, default track, selector in the sidebar). Track lists render only visible rows past 200 tracks. Pending — real-device visualizer sync check on Windows/macOS.
7. **M6 Qobuz**, then Amazon decision.

## Delegation to Codex (candidates)

- Local provider (scan + tags + SQLite) — well-defined, testable.
- Individual skins once the skin contract is fixed.
- TUI screens once the core API is stable.
- PKGBUILD / CI packaging.
