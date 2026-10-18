use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Cell, Paragraph, Row, Table},
};

use crate::app::{App, TABS, Tab, fmt_time};

const ACCENT: Color = Color::Rgb(255, 110, 150);
const DIM: Color = Color::Rgb(130, 130, 145);
const BG_SEL: Color = Color::Rgb(45, 40, 55);

pub fn draw(f: &mut Frame, app: &mut App) {
    let [header, list, now, help] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(9),
        Constraint::Length(1),
    ])
    .areas(f.area());

    draw_header(f, app, header);
    draw_list(f, app, list);
    draw_now(f, app, now);
    draw_help(f, app, help);
}

fn draw_header(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(DIM))
        .title(" ♫ MusicPlayer ".bold().fg(ACCENT));
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Tabs as wide click targets: " Library " etc.
    app.areas.tabs.clear();
    let mut x = inner.x + 1;
    for tab in TABS {
        let label = format!("  {}  ", tab.title());
        let w = label.chars().count() as u16;
        let r = Rect::new(x, inner.y, w, 1).intersection(inner);
        let style = if tab == app.tab {
            Style::new().bg(ACCENT).fg(Color::Black).bold()
        } else {
            Style::new().fg(Color::White)
        };
        f.render_widget(Paragraph::new(label).style(style), r);
        app.areas.tabs.push(r);
        x += w + 1;
    }

    let query = if app.tab == Tab::Youtube {
        &app.yt_query
    } else {
        &app.filter
    };
    let search = if app.searching {
        Line::from(vec![
            Span::raw(" / ").fg(ACCENT),
            Span::raw(query),
            Span::raw("▏").fg(ACCENT),
        ])
    } else if query.is_empty() {
        Line::from(" / buscar ".fg(DIM))
    } else {
        Line::from(vec![
            Span::raw(" / ").fg(ACCENT),
            Span::raw(query).fg(ACCENT),
        ])
    };
    let w = 30.min(inner.width.saturating_sub(x - inner.x));
    let r = Rect::new(inner.right().saturating_sub(w), inner.y, w, 1);
    f.render_widget(Paragraph::new(search).alignment(Alignment::Right), r);
}

fn draw_list(f: &mut Frame, app: &mut App, area: Rect) {
    let playing_uri = app.now_playing().map(|t| t.uri.clone());
    let row_style = |uri: &str| {
        if Some(uri) == playing_uri.as_deref() {
            Style::new().fg(ACCENT).bold()
        } else {
            Style::new()
        }
    };

    let (header, widths, rows, title): (Row, Vec<Constraint>, Vec<Row>, String) = match app.tab {
        Tab::Library | Tab::Queue => {
            let tracks: Vec<_> = if app.tab == Tab::Library {
                app.filtered().into_iter().cloned().collect()
            } else {
                app.queue.clone()
            };
            let rows = tracks
                .iter()
                .map(|t| {
                    let mark = if Some(&t.uri) == playing_uri.as_ref() {
                        "▶"
                    } else {
                        " "
                    };
                    Row::new(vec![
                        Cell::from(mark),
                        Cell::from(t.title.clone()),
                        Cell::from(t.artist.clone()).fg(DIM),
                        Cell::from(t.album.clone()).fg(DIM),
                        Cell::from(fmt_time(t.duration.unwrap_or(0.0))).fg(DIM),
                    ])
                    .style(row_style(&t.uri))
                })
                .collect();
            (
                Row::new(["", "Título", "Artista", "Álbum", "Duración"]),
                vec![
                    Constraint::Length(2),
                    Constraint::Fill(3),
                    Constraint::Fill(2),
                    Constraint::Fill(2),
                    Constraint::Length(8),
                ],
                rows,
                format!(" {} · {} canciones ", app.tab.title(), tracks.len()),
            )
        }
        Tab::Youtube | Tab::Playlists => {
            let view = app.remote_view();
            let rows = view
                .rows
                .iter()
                .map(|r| {
                    let (marker, title, subtitle) = r.label();
                    let style = match r {
                        crate::youtube::YoutubeRow::Track(t) => row_style(&t.uri),
                        _ => Style::new(),
                    };
                    Row::new(vec![
                        Cell::from(marker),
                        Cell::from(title),
                        Cell::from(subtitle).fg(DIM),
                    ])
                    .style(style)
                })
                .collect();
            (
                Row::new(["Tipo", "Título", "Artista / descripción"]),
                vec![
                    Constraint::Length(12),
                    Constraint::Fill(3),
                    Constraint::Fill(2),
                ],
                rows,
                format!(
                    " {} ",
                    view.opened
                        .as_ref()
                        .map(|(title, _, _)| title.as_str())
                        .unwrap_or(app.tab.title())
                ),
            )
        }
        Tab::Albums => {
            let rows = app
                .albums
                .iter()
                .map(|a| {
                    let total: f64 = a.tracks.iter().filter_map(|t| t.duration).sum();
                    Row::new(vec![
                        Cell::from(a.name.clone()),
                        Cell::from(a.artist.clone()).fg(DIM),
                        Cell::from(format!(
                            "{} canci{}",
                            a.tracks.len(),
                            if a.tracks.len() == 1 { "ón" } else { "ones" }
                        ))
                        .fg(DIM),
                        Cell::from(fmt_time(total)).fg(DIM),
                    ])
                })
                .collect();
            (
                Row::new(["Álbum", "Artista", "Canciones", "Duración"]),
                vec![
                    Constraint::Fill(3),
                    Constraint::Fill(2),
                    Constraint::Length(13),
                    Constraint::Length(8),
                ],
                rows,
                format!(" Álbumes · {} ", app.albums.len()),
            )
        }
    };

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(DIM))
        .title(title.bold());
    app.areas.list = block.inner(area);

    if matches!(app.tab, Tab::Youtube | Tab::Playlists) {
        let error = app.remote_error();
        let message = app.message.as_deref().or(error);
        if let Some(message) = message {
            f.render_widget(
                Paragraph::new(message.replace(['\n', '\r'], " "))
                    .fg(if error.is_some() { Color::Red } else { DIM })
                    .block(block),
                area,
            );
            return;
        }
    }

    if rows.is_empty() {
        let msg = match app.tab {
            Tab::Youtube if app.remote_view().opened.is_some() => {
                "No hay canciones en esta colección.".into()
            }
            Tab::Youtube => "Pulsa / para buscar canciones, álbumes y playlists.".into(),
            Tab::Playlists if !app.logged_in => {
                "Inicia sesión en la app de escritorio (Conectar cuenta); la sesión se comparte."
                    .into()
            }
            Tab::Playlists => "No hay canciones en esta colección.".into(),
            Tab::Queue => "La cola está vacía. Elige algo en Biblioteca o Álbumes.".to_string(),
            _ if !app.filter.is_empty() => format!("Nada coincide con “{}”.", app.filter),
            _ => format!(
                "No hay música en {}.\nEjecuta: musicplayer-tui <carpeta>",
                app.folder.display()
            ),
        };
        f.render_widget(
            Paragraph::new(msg)
                .fg(DIM)
                .alignment(Alignment::Center)
                .block(block),
            area,
        );
        return;
    }

    let table = Table::new(rows, widths)
        .header(header.style(Style::new().fg(DIM).add_modifier(Modifier::UNDERLINED)))
        .row_highlight_style(Style::new().bg(BG_SEL))
        .highlight_symbol("› ")
        .column_spacing(2)
        .block(block);
    f.render_stateful_widget(table, area, &mut app.table);
}

fn draw_now(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(ACCENT))
        .title(" Reproduciendo ".bold().fg(ACCENT));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let [info, progress, controls] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(3),
    ])
    .spacing(1)
    .areas(inner.inner(ratatui::layout::Margin::new(1, 0)));

    let s = &app.status;
    let line = if let Some((msg, _)) = &app.notice {
        Line::from(msg.as_str().fg(Color::Rgb(255, 120, 120)))
    } else {
        match app.now_playing() {
            Some(t) => Line::from(vec![
                Span::raw(&t.title).bold(),
                Span::raw("  —  ").fg(DIM),
                Span::raw(&t.artist),
                Span::raw(" · ").fg(DIM),
                Span::raw(&t.album).fg(DIM),
            ]),
            None => Line::from("Nada en reproducción — pulsa Enter en una canción".fg(DIM)),
        }
    };
    f.render_widget(Paragraph::new(line), info);

    let [elapsed, bar, total] = Layout::horizontal([
        Constraint::Length(7),
        Constraint::Fill(1),
        Constraint::Length(7),
    ])
    .areas(progress);
    let ratio = if s.duration > 0.0 {
        (s.position / s.duration).clamp(0.0, 1.0)
    } else {
        0.0
    };
    f.render_widget(Paragraph::new(fmt_time(s.position)), elapsed);
    f.render_widget(
        Paragraph::new(fmt_time(s.duration)).alignment(Alignment::Right),
        total,
    );
    f.render_widget(meter(ratio, bar.width, ACCENT), bar);
    app.areas.progress = bar;

    // Narrow terminals get icon-only buttons so labels never get clipped.
    let narrow = controls.width < 96;
    let (bw, pw) = if narrow { (9, 11) } else { (16, 18) };
    let [b0, b1, b2, viz, vol_label, vol] = Layout::horizontal([
        Constraint::Length(bw),
        Constraint::Length(pw),
        Constraint::Length(bw),
        Constraint::Fill(1),
        Constraint::Length(10),
        Constraint::Max(16),
    ])
    .spacing(1)
    .areas(controls);

    if viz.width >= 5 {
        f.render_widget(spectrum(&app.bars, viz), viz);
    }

    let playing = s.index.is_some() && !s.paused;
    let pick = |full: &'static str, icon: &'static str| if narrow { icon } else { full };
    let buttons = [
        (b0, pick("⏮  Anterior", "⏮"), false),
        (
            b1,
            if playing {
                pick("⏸  Pausar", "⏸")
            } else {
                pick("▶  Reproducir", "▶")
            },
            true,
        ),
        (b2, pick("Siguiente  ⏭", "⏭"), false),
    ];
    for (i, (r, label, primary)) in buttons.into_iter().enumerate() {
        let style = if primary {
            Style::new().fg(Color::Black).bg(ACCENT).bold()
        } else {
            Style::new().fg(Color::White)
        };
        f.render_widget(
            Paragraph::new(label)
                .alignment(Alignment::Center)
                .style(style)
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .border_style(if primary { style } else { Style::new().fg(DIM) }),
                ),
            r,
        );
        app.areas.buttons[i] = r;
    }

    let vol_line = Rect::new(vol_label.x, vol_label.y + 1, vol_label.width, 1);
    f.render_widget(
        Paragraph::new(format!("Vol {:>3.0}%", s.volume)).alignment(Alignment::Right),
        vol_line,
    );
    let vol_bar = Rect::new(vol.x, vol.y + 1, vol.width, 1);
    f.render_widget(
        meter(s.volume / 100.0, vol_bar.width, Color::Rgb(120, 200, 255)),
        vol_bar,
    );
    app.areas.volume = vol_bar;
}

/// Vertical bars with eighth-block resolution, centred in `area`.
/// Narrow areas merge neighbouring bands (loudest wins) so every band stays represented.
fn spectrum(bands: &[f32], area: Rect) -> Paragraph<'static> {
    const STEPS: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let n = bands.len().min((area.width as usize).div_ceil(2)).max(1);
    let bars: Vec<f32> = (0..n)
        .map(|i| {
            let (lo, hi) = (
                i * bands.len() / n,
                ((i + 1) * bands.len() / n).max(i * bands.len() / n + 1),
            );
            bands[lo..hi].iter().copied().fold(0.0, f32::max)
        })
        .collect();
    let col = ((area.width as usize + 1) / bars.len()).clamp(2, 3) - 1;
    let used = bars.len() * (col + 1) - 1;
    let pad = " ".repeat((area.width as usize).saturating_sub(used) / 2);
    let rows = area.height as usize;
    let lines = (0..rows)
        .map(|row| {
            // Row 0 is the top; each row holds 8 eighths of the bar height.
            let floor = (rows - 1 - row) * 8;
            let mut spans = vec![Span::raw(pad.clone())];
            for (i, &v) in bars.iter().enumerate() {
                let eighths = (v.clamp(0.0, 1.0) * (rows * 8) as f32).round() as usize;
                let ch = STEPS[eighths.saturating_sub(floor).min(8)];
                // Low bands warm, high bands cool, like the desktop accent.
                let hue = i as f32 / (bars.len().max(2) - 1) as f32;
                let color =
                    Color::Rgb(255, (110.0 + 90.0 * hue) as u8, (150.0 + 100.0 * hue) as u8);
                spans.push(Span::raw(ch.to_string().repeat(col)).fg(color));
                spans.push(Span::raw(" "));
            }
            spans.pop();
            Line::from(spans)
        })
        .collect::<Vec<_>>();
    Paragraph::new(lines)
}

/// One-line bar; ratatui's Gauge leaves a gap for its centered label.
fn meter(ratio: f64, width: u16, color: Color) -> Paragraph<'static> {
    let filled = (ratio.clamp(0.0, 1.0) * width as f64).round() as usize;
    Paragraph::new(Line::from(vec![
        Span::raw("█".repeat(filled)).fg(color),
        Span::raw("░".repeat(width as usize - filled)).fg(BG_SEL),
    ]))
}

fn draw_help(f: &mut Frame, app: &App, area: Rect) {
    let keys: &[(&str, &str)] = if app.searching {
        if app.tab == Tab::Youtube {
            &[
                ("escribe", "consulta"),
                ("Enter", "buscar"),
                ("Esc", "cancelar"),
            ]
        } else {
            &[
                ("escribe", "filtrar"),
                ("Enter", "listo"),
                ("Esc", "borrar"),
            ]
        }
    } else if matches!(app.tab, Tab::Youtube | Tab::Playlists) {
        if app.remote_view().opened.is_some() {
            &[
                ("Enter", "reproducir"),
                ("Esc/⌫", "volver"),
                ("Espacio", "pausar"),
                ("1-5/Tab", "pestaña"),
                ("q", "salir"),
            ]
        } else if app.tab == Tab::Youtube {
            &[
                ("/", "buscar"),
                ("Enter", "reproducir/abrir"),
                ("1-5/Tab", "pestaña"),
                ("q", "salir"),
            ]
        } else {
            &[
                ("Enter", "abrir"),
                ("1-5/Tab", "pestaña"),
                ("Espacio", "pausar"),
                ("q", "salir"),
            ]
        }
    } else {
        &[
            ("Espacio", "reproducir/pausar"),
            ("q", "salir"),
            ("Enter", "reproducir"),
            ("n/p", "sig./ant."),
            ("←/→", "±5 s"),
            ("+/-", "volumen"),
            ("1-5/Tab", "pestaña"),
            ("/", "filtrar biblioteca"),
        ]
    };
    let spans: Vec<Span> = keys
        .iter()
        .flat_map(|(k, v)| {
            [
                Span::raw(format!(" {k} ")).fg(Color::Black).bg(DIM),
                Span::raw(format!(" {v}  ")).fg(DIM),
            ]
        })
        .collect();
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tests::{app, track};
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn renders_and_records_click_areas() {
        let mut a = app(vec![track("Midnight City", "M83", "Hurry Up")]);
        let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
        term.draw(|f| draw(f, &mut a)).unwrap();
        let screen: String = term
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(screen.contains("Midnight City"));
        assert!(screen.contains("Reproducir"));
        assert_eq!(a.areas.tabs.len(), 5);
        assert!(a.areas.buttons.iter().all(|r| r.height == 3));
        assert!(a.areas.progress.width > 0 && a.areas.volume.width > 0);
    }

    #[test]
    fn spectrum_bars_fill_from_the_bottom() {
        let mut bars = [0.0; 19];
        bars[0] = 1.0;
        bars[18] = 0.5;
        let p = spectrum(&bars, Rect::new(0, 0, 38, 3));
        let mut term = Terminal::new(TestBackend::new(38, 3)).unwrap();
        term.draw(|f| f.render_widget(p, f.area())).unwrap();
        let b = term.backend().buffer();
        assert_eq!(b[(0, 0)].symbol(), "█", "full bar reaches the top");
        assert_eq!(b[(0, 2)].symbol(), "█");
        let last = 18 * 2;
        assert_eq!(
            b[(last, 0)].symbol(),
            " ",
            "half bar stays below the top row"
        );
        assert_eq!(b[(last, 2)].symbol(), "█");
        assert_eq!(b[(2, 2)].symbol(), " ", "silent band is empty");
    }

    #[test]
    fn narrow_spectrum_merges_bands() {
        let mut bars = [0.0; 19];
        bars[18] = 1.0;
        let mut term = Terminal::new(TestBackend::new(9, 1)).unwrap();
        term.draw(|f| f.render_widget(spectrum(&bars, f.area()), f.area()))
            .unwrap();
        let row: String = term
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert_eq!(
            row.trim_end(),
            "        █",
            "5 bars; the top band survives in the last one"
        );
    }
}
