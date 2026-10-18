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

    let search = if app.searching {
        Line::from(vec![
            Span::raw(" / ").fg(ACCENT),
            Span::raw(&app.filter),
            Span::raw("▏").fg(ACCENT),
        ])
    } else if app.filter.is_empty() {
        Line::from(" / buscar ".fg(DIM))
    } else {
        Line::from(vec![
            Span::raw(" / ").fg(ACCENT),
            Span::raw(&app.filter).fg(ACCENT),
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

    if rows.is_empty() {
        let msg = match app.tab {
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
    let line = match app.now_playing() {
        Some(t) => Line::from(vec![
            Span::raw(&t.title).bold(),
            Span::raw("  —  ").fg(DIM),
            Span::raw(&t.artist),
            Span::raw(" · ").fg(DIM),
            Span::raw(&t.album).fg(DIM),
        ]),
        None => Line::from("Nada en reproducción — pulsa Enter en una canción".fg(DIM)),
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

    let [b0, b1, b2, _, vol_label, vol] = Layout::horizontal([
        Constraint::Length(16),
        Constraint::Length(18),
        Constraint::Length(16),
        Constraint::Fill(1),
        Constraint::Length(10),
        Constraint::Length(24),
    ])
    .spacing(1)
    .areas(controls);

    let playing = s.index.is_some() && !s.paused;
    let buttons = [
        (b0, "⏮  Anterior", false),
        (
            b1,
            if playing {
                "⏸  Pausar"
            } else {
                "▶  Reproducir"
            },
            true,
        ),
        (b2, "Siguiente  ⏭", false),
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
        &[
            ("escribe", "filtrar"),
            ("Enter", "listo"),
            ("Esc", "borrar"),
        ]
    } else {
        &[
            ("Espacio", "reproducir/pausar"),
            ("q", "salir"),
            ("Enter", "reproducir"),
            ("n/p", "sig./ant."),
            ("←/→", "±5 s"),
            ("+/-", "volumen"),
            ("Tab", "pestaña"),
            ("/", "buscar"),
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
        assert_eq!(a.areas.tabs.len(), 3);
        assert!(a.areas.buttons.iter().all(|r| r.height == 3));
        assert!(a.areas.progress.width > 0 && a.areas.volume.width > 0);
    }
}
