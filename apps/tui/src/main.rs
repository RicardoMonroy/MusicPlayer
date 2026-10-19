mod app;
mod equalizer;
mod local;
mod palette;
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
    let player = Player::new().map_err(|e| format!("No se pudo iniciar libmpv: {e}"))?;
    // The TUI always shows the spectrum; without the needed ffmpeg filters it just stays flat.
    let _ = player.set_visualizer(true);
    let mut app = App::new(player, folder);
    app.start_scan();

    let mut terminal = ratatui::init();
    execute!(std::io::stdout(), EnableMouseCapture)?;
    let result = run(&mut terminal, &mut app);
    app.flush_eq();
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
        // ~25 fps while the spectrum moves, 4 fps when idle.
        let frame = if app.is_playing()
            || app.message.is_some()
            || app.lyrics.as_ref().is_some_and(|p| p.loading)
        {
            40
        } else {
            250
        };
        if event::poll(Duration::from_millis(frame))? {
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
    if app.tab == app::Tab::Equalizer && app.lyrics.is_none() {
        let step = if k.modifiers.contains(KeyModifiers::SHIFT) {
            2.0
        } else {
            0.5
        };
        let changed = match k.code {
            KeyCode::Left => {
                app.eq.select(-1);
                return;
            }
            KeyCode::Right => {
                app.eq.select(1);
                return;
            }
            KeyCode::Up => {
                app.eq.adjust(step);
                true
            }
            KeyCode::Down => {
                app.eq.adjust(-step);
                true
            }
            KeyCode::Char('0') => {
                app.eq.set_gain(0.0);
                true
            }
            KeyCode::Char('o') => {
                app.eq.value.enabled = !app.eq.value.enabled;
                true
            }
            KeyCode::Char('p') => {
                app.eq.cycle_preset(1);
                true
            }
            KeyCode::Char('P') => {
                app.eq.cycle_preset(-1);
                true
            }
            _ => false,
        };
        if changed {
            app.eq_changed();
            return;
        }
    }
    let p = &app.player;
    match k.code {
        KeyCode::Char('v') if app.is_local() => app.cycle_local(),
        KeyCode::Char('o') if app.is_local() => {
            app.local.order = app.local.order.next();
            app.table.select((app.row_count() > 0).then_some(0));
        }
        KeyCode::Char('p')
            if app.is_local()
                && matches!(
                    app.local.state.page,
                    local::Page::Album(_) | local::Page::Folders(_)
                ) =>
        {
            app.local_collection(false)
        }
        KeyCode::Char('s')
            if app.is_local() && matches!(app.local.state.page, local::Page::Album(_)) =>
        {
            app.local_collection(true)
        }
        KeyCode::Char('t') => app.cycle_palette(),
        KeyCode::Char('l') => app.toggle_lyrics(),
        KeyCode::Char('r') if matches!(app.tab, app::Tab::Youtube | app::Tab::Playlists) => {
            app.yt_query.clear();
            app.yt_view = Default::default();
            if app.logged_in {
                app.schedule(app::NetworkAction::Library, "Cargando biblioteca…");
            } else {
                app.account_checking = true;
                app.schedule(app::NetworkAction::Account, "Comprobando cuenta…");
            }
        }
        KeyCode::Char('q') => app.quit = true,
        KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => app.quit = true,
        KeyCode::Char(' ') => drop(p.toggle_pause()),
        KeyCode::Char('n') => drop(p.next()),
        KeyCode::Char('p')
            if matches!(app.tab, app::Tab::Youtube | app::Tab::Playlists)
                && app.remote_view().opened.is_some()
                && app.lyrics.is_none() =>
        {
            app.play_collection()
        }
        KeyCode::Char('p') => drop(p.prev()),
        KeyCode::Left => app.seek_by(-5.0),
        KeyCode::Right => app.seek_by(5.0),
        KeyCode::Char('+') | KeyCode::Char('=') => app.volume_by(5.0),
        KeyCode::Char('-') => app.volume_by(-5.0),
        KeyCode::Char('/') => {
            if !app.is_local() {
                app.set_tab(app::Tab::Youtube);
            }
            app.searching = true;
        }
        KeyCode::Esc | KeyCode::Backspace => app.back(),
        KeyCode::Char('f') if app.tab == app::Tab::Library => app.searching = true,
        KeyCode::Tab if app.search_results() || app.local_artist() => app.focus_column(1),
        KeyCode::BackTab if app.search_results() || app.local_artist() => app.focus_column(-1),
        KeyCode::Tab => app.cycle_tab(1),
        KeyCode::BackTab => app.cycle_tab(-1),
        KeyCode::Char(c @ '1'..='6') => app.set_tab(TABS[c as usize - '1' as usize]),
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
    if app.tab == app::Tab::Equalizer && app.lyrics.is_none() {
        if let Some((i, rect)) = app
            .areas
            .eq_bars
            .iter()
            .copied()
            .enumerate()
            .find(|(_, r)| hit(*r, x, y))
        {
            app.eq.selected = i;
            match m.kind {
                MouseEventKind::Down(MouseButton::Left)
                | MouseEventKind::Drag(MouseButton::Left) => {
                    app.eq.set_gain(equalizer::gain_at(rect, y))
                }
                MouseEventKind::ScrollUp => app.eq.adjust(0.5),
                MouseEventKind::ScrollDown => app.eq.adjust(-0.5),
                _ => return,
            }
            app.eq_changed();
            return;
        }
        if let Some((i, _)) = app.areas.eq_presets.iter().find(|(_, r)| hit(*r, x, y))
            && matches!(m.kind, MouseEventKind::Down(MouseButton::Left))
        {
            app.table.select(Some(*i));
            return;
        }
    }
    if matches!(
        m.kind,
        MouseEventKind::ScrollUp | MouseEventKind::ScrollDown
    ) && let Some((column, _)) = app
        .areas
        .columns
        .iter()
        .find(|(_, r)| hit(*r, x, y))
        .copied()
        && column != app.column_focus()
    {
        app.focus_column(column as isize - app.column_focus() as isize);
    }
    match m.kind {
        MouseEventKind::ScrollUp => app.move_selection(-3),
        MouseEventKind::ScrollDown => app.move_selection(3),
        MouseEventKind::Down(MouseButton::Left) => {
            let a = &app.areas;
            if hit(a.top, x, y) {
                app.open_top();
            } else if hit(a.switcher, x, y) {
                let step = if x < a.switcher.x + a.switcher.width / 2 {
                    -1
                } else {
                    1
                };
                app.focus_column(step);
                app.searching = false;
            } else if hit(a.search, x, y) {
                app.searching = true;
            } else if let Some((column, rect)) = a
                .columns
                .iter()
                .find(|(_, r)| hit(r.outer(ratatui::layout::Margin::new(1, 1)), x, y))
                .copied()
            {
                app.searching = false;
                if column != app.column_focus() {
                    app.focus_column(column as isize - app.column_focus() as isize);
                }
                if !hit(rect, x, y) {
                    return;
                }
                let row = (y - rect.y) as usize + app.table.offset();
                if row < app.row_count() {
                    let double = matches!(app.last_click, Some((r, t)) if r == row && t.elapsed() < Duration::from_millis(400));
                    app.table.select(Some(row));
                    app.last_click = Some((row, Instant::now()));
                    if double {
                        app.play_selected();
                    }
                }
            } else if let Some(i) = a.tabs.iter().position(|r| hit(*r, x, y)) {
                app.set_tab(TABS[i]);
            } else if let Some(i) = a.buttons.iter().position(|r| hit(*r, x, y)) {
                let _ = [App::prev, App::toggle, App::next][i](app);
            } else if hit(a.progress, x, y) {
                let _ = app
                    .player
                    .seek(ratio_at(a.progress, x) * app.status.duration);
            } else if hit(a.volume, x, y) {
                let _ = app.player.set_volume(ratio_at(a.volume, x) * 100.0);
            } else if hit(a.list, x, y)
                && let Some(row) = app.row_at(y)
            {
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
    fn local_keys_filter_navigation_queue_and_folder_playback() {
        let mut a = app(crate::local::tests::discography());
        a.folder = PathBuf::from("/music");
        a.yt_query = "consulta remota".into();
        key(&mut a, KeyCode::Char('v'));
        assert_eq!(a.local.state.page, local::Page::Artists);
        key(&mut a, KeyCode::Char('/'));
        for c in "opeth".chars() {
            key(&mut a, KeyCode::Char(c));
        }
        key(&mut a, KeyCode::Enter);
        assert_eq!(a.row_count(), 1);
        assert_eq!(a.yt_query, "consulta remota");
        key(&mut a, KeyCode::Enter);
        assert!(a.local_artist());
        assert_eq!(a.column_focus(), 1);
        assert!(a.filter.is_empty());
        key(&mut a, KeyCode::Down);
        key(&mut a, KeyCode::Enter);
        assert!(matches!(a.local.state.page, local::Page::Album(_)));
        a.table.select(Some(2));
        key(&mut a, KeyCode::Enter);
        assert_eq!(a.queue.len(), 3);
        assert_eq!(a.queue[0].title, "Inicio");
        assert_eq!(a.player.status().index, Some(1));
        key(&mut a, KeyCode::Char('p'));
        assert_eq!(a.player.status().index, Some(0));
        key(&mut a, KeyCode::Char('s'));
        assert_eq!(a.queue.len(), 3);
        let mut names: Vec<_> = a.queue.iter().map(|t| t.title.as_str()).collect();
        names.sort();
        assert_eq!(names, ["Inicio", "Primera", "Segunda"]);
        key(&mut a, KeyCode::Esc);
        assert_eq!(a.column_focus(), 1);
        assert_eq!(a.table.selected(), Some(1));
        key(&mut a, KeyCode::Tab);
        assert_eq!(a.column_focus(), 0);
        a.table.select(Some(2));
        key(&mut a, KeyCode::Enter);
        assert_eq!(a.queue.len(), 4);
        assert_eq!(a.queue[0].title, "Temprana");
        key(&mut a, KeyCode::Esc);
        assert_eq!(a.filter, "opeth");
        a.open_local(local::Page::Folders(String::new()));
        key(&mut a, KeyCode::Enter);
        assert_eq!(a.local.state.page, local::Page::Folders("Opeth".into()));
        key(&mut a, KeyCode::Char('p'));
        assert_eq!(a.queue.len(), 4);
        key(&mut a, KeyCode::Backspace);
        assert_eq!(a.local.state.page, local::Page::Folders(String::new()));
        key(&mut a, KeyCode::Char('2'));
        assert_eq!(a.local.state.page, local::Page::Albums);
        key(&mut a, KeyCode::Char('o'));
        assert_eq!(a.local.order, local::Order::Year);
    }

    #[test]
    fn equalizer_keyboard_and_mouse_controls() {
        let mut a = app(Vec::new());
        key(&mut a, KeyCode::Char('6'));
        assert_eq!(a.tab, Tab::Equalizer);
        a.eq = equalizer::EqState::new(Default::default());
        key(&mut a, KeyCode::Right);
        assert_eq!(a.eq.selected, 1);
        key(&mut a, KeyCode::Up);
        assert_eq!(a.eq.value.gains[0], 0.5);
        on_key(&mut a, KeyEvent::new(KeyCode::Down, KeyModifiers::SHIFT));
        assert_eq!(a.eq.value.gains[0], -1.5);
        key(&mut a, KeyCode::Char('0'));
        assert_eq!(a.eq.value.gains[0], 0.0);
        key(&mut a, KeyCode::Char('o'));
        assert!(!a.eq.value.enabled);
        key(&mut a, KeyCode::Char('p'));
        assert_eq!(equalizer::matching_preset(&a.eq.value), Some(1));
        key(&mut a, KeyCode::Char('P'));
        assert_eq!(equalizer::matching_preset(&a.eq.value), Some(0));
        a.areas.eq_bars = vec![ratatui::layout::Rect::new(10, 5, 5, 25)];
        on_mouse(
            &mut a,
            MouseEvent {
                kind: MouseEventKind::Drag(MouseButton::Left),
                column: 12,
                row: 5,
                modifiers: KeyModifiers::NONE,
            },
        );
        assert_eq!(a.eq.selected, 0);
        assert_eq!(a.eq.value.preamp, 12.0);
        on_mouse(
            &mut a,
            MouseEvent {
                kind: MouseEventKind::ScrollDown,
                column: 12,
                row: 5,
                modifiers: KeyModifiers::NONE,
            },
        );
        assert_eq!(a.eq.value.preamp, 11.5);
        a.table.select(Some(13));
        key(&mut a, KeyCode::Enter);
        assert_eq!(equalizer::matching_preset(&a.eq.value), Some(13));
        assert!(a.eq.value.enabled);
    }

    #[test]
    fn album_shortcut_plays_collection_and_keeps_selection() {
        let mut a = app(Vec::new());
        a.tab = Tab::Youtube;
        a.yt_view.open(
            "Disco".into(),
            vec![
                crate::app::tests::track("primera", "A", "B"),
                crate::app::tests::track("segunda", "A", "B"),
            ],
            0,
        );
        a.table.select(Some(1));
        key(&mut a, KeyCode::Char('p'));
        assert_eq!(a.queue.len(), 2);
        assert_eq!(a.queue[0].title, "primera");
        assert_eq!(a.table.selected(), Some(1));
    }

    #[test]
    fn top_artist_click_schedules_artist_open() {
        let mut a = app(Vec::new());
        a.tab = Tab::Youtube;
        a.yt_view.search(mp_ytmusic::SearchResults {
            top: Some(mp_ytmusic::Top::Artist(mp_ytmusic::Collection {
                kind: mp_ytmusic::Kind::Artist,
                id: "artist".into(),
                title: "Artista".into(),
                subtitle: String::new(),
                cover: None,
            })),
            ..Default::default()
        });
        a.areas.top = ratatui::layout::Rect::new(6, 3, 74, 3);
        on_mouse(
            &mut a,
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 8,
                row: 4,
                modifiers: KeyModifiers::NONE,
            },
        );
        assert!(matches!(a.pending, Some(NetworkAction::Open(
            crate::youtube::YoutubeRow::Collection(ref c), _)) if c.id == "artist"));
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
    fn search_focus_is_explicit_and_escape_keeps_query() {
        let mut a = app(Vec::new());
        key(&mut a, KeyCode::Char('4'));
        key(&mut a, KeyCode::Char('/'));
        assert_eq!(a.tab, Tab::Youtube);
        key(&mut a, KeyCode::Char('z'));
        key(&mut a, KeyCode::Esc);
        key(&mut a, KeyCode::Char('z'));
        assert_eq!(a.yt_query, "z");
        assert!(!a.searching);
        key(&mut a, KeyCode::Char('1'));
        key(&mut a, KeyCode::Char('f'));
        key(&mut a, KeyCode::Char('x'));
        assert_eq!(a.filter, "x");
        assert_eq!(a.yt_query, "z");
    }

    #[test]
    fn fifth_tab_defers_library_load_and_tab_click_cancels_prompt() {
        let mut app = app(Vec::new());
        key(&mut app, KeyCode::Char('/'));
        assert!(app.searching);
        app.account_checking = false;
        app.set_tab(Tab::Playlists);
        assert!(!app.searching);
        assert!(matches!(app.pending, Some(NetworkAction::Account)));
        app.pending = None;
        app.library_loaded = true;
        app.logged_in = true;
        key(&mut app, KeyCode::Char('1'));
        key(&mut app, KeyCode::Char('5'));
        assert_eq!(app.tab, Tab::Playlists);
        assert!(app.pending.is_none());
    }
}
