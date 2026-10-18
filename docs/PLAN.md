# Plan — MusicPlayer

Status: stack approved 2026-10-18. M0 done. M1 in progress.

## Stack

| Layer | Choice | Why |
|---|---|---|
| Core | Rust workspace | One engine shared by GUI and TUI. |
| Playback | libmpv (`libmpv2` crate) | Plays mp3/flac/ogg/opus/m4a/wav, gapless, ReplayGain, and YouTube streams through yt-dlp. Available on all three OS. |
| Tags/library | `lofty` (+ SQLite cache later) | Fast tag reading, covers. M1 rescans the folder on start; add SQLite when big libraries make startup slow. |
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

## Views

- **Full view**: mix of Dopamine (clean library, album grid), Amberol (cover-driven adaptive colors), Tauon (fast browsing, gallery). Dynamic accent color from the cover art.
- **Compact view**: small borderless window. Skin is a plugin: `skins/<name>/` with `skin.json` + Svelte component + assets.
  - Initial skins: Cassette (Kaset-like, reels spin with progress), Vinyl (record spins, tonearm moves with progress), Classic (Winamp 2-like, spectrum visualizer).
- **Tray**: minimizes to tray. Click opens a popup with the active skin's mini controls.
- **TUI (Linux first)**: ytm-player-like, but big buttons, mouse support, tabs (Library, Search, Queue, Playlists), visible key hints.

## Streaming services — real constraints

- **YouTube Music**: no official API. Use `ytmusicapi` logic (Python sidecar first, or the Rust port `ytmapi-rs`) plus yt-dlp for streams. Works, but against YouTube ToS and breaks when YouTube changes things. Login via browser cookies/headers.
- **Qobuz**: partner API exists. Community clients (streamrip, qobuz-dl) use the web player app_id. Hi-res FLAC streams work well with mpv. Do it when the account is active.
- **Amazon Music**: no public playback API and DRM (Widevine). Tauri webviews on Linux (WebKitGTK) do not support Widevine. Options: (a) embed the web player in a separate Electron window with Widevine (castLabs build), (b) control-only integration (now-playing + media keys of the official app), (c) drop it. Recommendation: (b) or postpone.

## Roadmap

1. **M0 scaffold**: Rust workspace, Tauri app, CI build for Linux/Windows/macOS.
2. **M1 local playback**: core + local provider + basic full view + queue.
3. **M2 compact + skins**: compact window, skin system, Cassette/Vinyl/Classic, tray popup, media keys.
4. **M3 TUI**: ratatui client on the same core.
5. **M4 YouTube Music**: search, library, playlists, playback.
6. **M5 polish**: visualizer, lyrics, ReplayGain, AUR package (`PKGBUILD`), Windows/macOS installers.
7. **M6 Qobuz**, then Amazon decision.

## Delegation to Codex (candidates)

- Local provider (scan + tags + SQLite) — well-defined, testable.
- Individual skins once the skin contract is fixed.
- TUI screens once the core API is stable.
- PKGBUILD / CI packaging.
