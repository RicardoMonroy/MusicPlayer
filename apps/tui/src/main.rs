mod app;
mod ui;
mod youtube;

use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use app::{App, TABS};
use mp_core::player::Player;
use ratatui::crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let folder = pick_folder();
    let player = Player::new().map_err(|e| format!("libmpv failed to start: {e}"))?;
    let mut app = App::new(player, folder);

    let mut terminal = ratatui::init();
    execute!(std::io::stdout(), EnableMouseCapture)?;
    let result = run(&mut terminal, &mut app);
    let _ = execute!(std::io::stdout(), DisableMouseCapture);
    ratatui::restore();
    result
}

/// Folder from the first argument (remembered), else the shared setting, else ~/Music.
fn pick_folder() -> PathBuf {
    if let Some(arg) = std::env::args_os().nth(1) {
        let folder = std::path::absolute(PathBuf::from(arg)).unwrap_or_default();
        let _ = mp_core::config::set_music_folder(&folder);
        return folder;
    }
    mp_core::config::music_folder()
        .or_else(dirs::audio_dir)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn run(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    while !app.quit {
        app.tick();
        terminal.draw(|f| ui::draw(f, app))?;
        app.run_pending();
        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(k) if k.kind == KeyEventKind::Press => on_key(app, k),
                Event::Mouse(m) => on_mouse(app, m),
                _ => {}
            }
        }
    }
    Ok(())
}

fn on_key(app: &mut App, k: KeyEvent) {
    if app.searching {
        let youtube = app.tab == app::Tab::Youtube;
        let query = if youtube {
            &mut app.yt_query
        } else {
            &mut app.filter
        };
        match k.code {
            KeyCode::Esc => {
                query.clear();
                app.searching = false;
            }
            KeyCode::Enter => {
                app.searching = false;
                if youtube {
                    app.schedule(app::NetworkAction::Search, "Buscando…");
                }
            }
            KeyCode::Backspace => {
                query.pop();
            }
            KeyCode::Char(c) => query.push(c),
            _ => return,
        }
        app.table.select((app.row_count() > 0).then_some(0));
        return;
    }
    let p = &app.player;
    match k.code {
        KeyCode::Char('q') => app.quit = true,
        KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => app.quit = true,
        KeyCode::Char(' ') => drop(p.toggle_pause()),
        KeyCode::Char('n') => drop(p.next()),
        KeyCode::Char('p') => drop(p.prev()),
        KeyCode::Left => app.seek_by(-5.0),
        KeyCode::Right => app.seek_by(5.0),
        KeyCode::Char('+') | KeyCode::Char('=') => app.volume_by(5.0),
        KeyCode::Char('-') => app.volume_by(-5.0),
        KeyCode::Char('/') => {
            if app.tab != app::Tab::Youtube {
                app.set_tab(app::Tab::Library);
            }
            app.searching = true;
        }
        KeyCode::Esc | KeyCode::Backspace => app.back(),
        KeyCode::Tab => app.cycle_tab(1),
        KeyCode::BackTab => app.cycle_tab(-1),
        KeyCode::Char(c @ '1'..='5') => app.set_tab(TABS[c as usize - '1' as usize]),
        KeyCode::Up | KeyCode::Char('k') => app.move_selection(-1),
        KeyCode::Down | KeyCode::Char('j') => app.move_selection(1),
        KeyCode::PageUp => app.move_selection(-10),
        KeyCode::PageDown => app.move_selection(10),
        KeyCode::Home | KeyCode::Char('g') => app.move_selection(isize::MIN / 2),
        KeyCode::End | KeyCode::Char('G') => app.move_selection(isize::MAX / 2),
        KeyCode::Enter => app.play_selected(),
        _ => {}
    }
}

fn hit(r: ratatui::layout::Rect, x: u16, y: u16) -> bool {
    r.contains(ratatui::layout::Position { x, y })
}

/// Fraction of `r`'s width at column `x`.
fn ratio_at(r: ratatui::layout::Rect, x: u16) -> f64 {
    ((x - r.x) as f64 + 0.5) / r.width.max(1) as f64
}

fn on_mouse(app: &mut App, m: MouseEvent) {
    let (x, y) = (m.column, m.row);
    match m.kind {
        MouseEventKind::ScrollUp => app.move_selection(-3),
        MouseEventKind::ScrollDown => app.move_selection(3),
        MouseEventKind::Down(MouseButton::Left) => {
            let a = &app.areas;
            if let Some(i) = a.tabs.iter().position(|r| hit(*r, x, y)) {
                app.set_tab(TABS[i]);
            } else if let Some(i) = a.buttons.iter().position(|r| hit(*r, x, y)) {
                let _ = [App::prev, App::toggle, App::next][i](app);
            } else if hit(a.progress, x, y) {
                let _ = app
                    .player
                    .seek(ratio_at(a.progress, x) * app.status.duration);
            } else if hit(a.volume, x, y) {
                let _ = app.player.set_volume(ratio_at(a.volume, x) * 100.0);
            } else if let Some(row) = app.row_at(y) {
                // Click selects, a second click on the same row within 400 ms plays.
                let double = matches!(app.last_click, Some((r, t)) if r == row && t.elapsed() < Duration::from_millis(400));
                app.table.select(Some(row));
                app.last_click = Some((row, Instant::now()));
                if double {
                    app.play_selected();
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{NetworkAction, Tab, tests::app};

    fn key(app: &mut App, code: KeyCode) {
        on_key(app, KeyEvent::new(code, KeyModifiers::NONE));
    }

    #[test]
    fn youtube_prompt_schedules_search_without_filtering_library() {
        let mut app = app(Vec::new());
        app.filter = "local".into();
        key(&mut app, KeyCode::Char('4'));
        assert_eq!(app.tab, Tab::Youtube);
        key(&mut app, KeyCode::Char('/'));
        key(&mut app, KeyCode::Char('a'));
        assert_eq!(app.yt_query, "a");
        assert_eq!(app.filter, "local");
        assert!(app.pending.is_none());
        key(&mut app, KeyCode::Enter);
        assert!(!app.searching);
        assert!(matches!(app.pending, Some(NetworkAction::Search)));
        assert_eq!(app.message.as_deref(), Some("Buscando…"));
    }

    #[test]
    fn fifth_tab_defers_library_load_and_tab_click_cancels_prompt() {
        let mut app = app(Vec::new());
        key(&mut app, KeyCode::Char('/'));
        assert!(app.searching);
        app.set_tab(Tab::Playlists);
        assert!(!app.searching);
        assert!(matches!(app.pending, Some(NetworkAction::Library)));
        app.pending = None;
        app.library_loaded = true;
        key(&mut app, KeyCode::Char('1'));
        key(&mut app, KeyCode::Char('5'));
        assert_eq!(app.tab, Tab::Playlists);
        assert!(app.pending.is_none());
    }
}
