# MusicPlayer

Cross-platform music player (Windows, Linux, macOS) + Linux TUI. Sources: local files (mp3, flac, common formats), YouTube Music, Qobuz (later), Amazon Music (limited, see docs/PLAN.md).

## Working style
- Always active: `caveman` skill for chat replies, `ponytail` skill for code. Use `caveman-commit` for commit messages and `ponytail-review` / `caveman-review` for reviews.
- Use `tauri-v2` skill for anything under `src-tauri/` or IPC.
- Plan and roadmap live in `docs/PLAN.md`. Update it when a decision changes.
- Codex (`codex` CLI) is available: delegate well-scoped, self-contained tasks (a crate, a skin, tests) with a precise brief, then review its diff.

## Layout (target)
- `crates/core` — playback engine (libmpv), queue, library, provider trait.
- `crates/providers/*` — local, ytmusic, qobuz.
- `apps/desktop` — Tauri v2 + Svelte UI, full mode + compact skins + tray.
- `apps/tui` — ratatui client.
