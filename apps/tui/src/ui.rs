use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Cell, Paragraph, Row, Table, Wrap},
};

use crate::app::{App, TABS, Tab, fmt_time};

pub fn draw(f: &mut Frame, app: &mut App) {
    let p = app.palette;
    f.render_widget(
        Block::default().style(Style::new().fg(p.text).bg(p.background)),
        f.area(),
    );
    app.areas.eq_bars.clear();
    app.areas.eq_presets.clear();
    let [content, now, help] = Layout::vertical([
        Constraint::Min(5),
        Constraint::Length(9),
        Constraint::Length(if app.tab == Tab::Equalizer && app.lyrics.is_none() {
            2
        } else {
            1
        }),
    ])
    .areas(f.area());
    let [sidebar, mut main] = Layout::horizontal([
        Constraint::Length(sidebar_width(f.area().width)),
        Constraint::Min(0),
    ])
    .areas(content);
    app.areas.top = Rect::default();
    app.areas.search = Rect::default();
    app.areas.list = Rect::default();
    app.areas.switcher = Rect::default();
    app.areas.columns.clear();
    draw_sidebar(f, app, sidebar);
    if app.lyrics.is_none() && app.tab == Tab::Youtube {
        let [search, rest] =
            Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(main);
        main = rest;
        let text = if app.yt_query.is_empty() && !app.searching {
            "Busca canciones, artistas, álbumes, playlists…".to_string()
        } else {
            format!("{}{}", app.yt_query, if app.searching { "▏" } else { "" })
        };
        app.areas.search = search;
        f.render_widget(
            Paragraph::new(text).block(
                Block::bordered()
                    .title(" Buscar ")
                    .border_style(Style::new().fg(if app.searching { p.accent } else { p.dim })),
            ),
            search,
        );
    } else if app.lyrics.is_none() && app.is_local() {
        let [filter, rest] =
            Layout::vertical([Constraint::Length(2), Constraint::Min(0)]).areas(main);
        main = rest;
        app.areas.search = filter;
        f.render_widget(
            Paragraph::new(format!(
                "v: Canciones · Artistas · Discos · Carpetas\nFiltro (/): {}{}",
                app.filter,
                if app.searching { "▏" } else { "" }
            ))
            .fg(p.dim),
            filter,
        );
    }
    if app.tab == Tab::Equalizer && app.lyrics.is_none() {
        crate::equalizer::draw(f, app, main);
    } else if app.is_local() && !matches!(app.local.state.page, crate::local::Page::Songs) {
        draw_local(f, app, main);
    } else if app.search_results() && app.remote_error().is_none() {
        draw_results(f, app, main);
    } else {
        draw_list(f, app, main);
    }
    draw_now(f, app, now);
    draw_help(f, app, help);
}

fn sidebar_width(width: u16) -> u16 {
    if width < 90 { 6 } else { 22 }
}

fn tab_labels(width: u16) -> Vec<String> {
    let icons = ["♫", "◉", "≡", "⌕", "♥", "≋"];
    TABS.iter()
        .enumerate()
        .map(|(i, tab)| {
            if width > 6 {
                format!(" {} {} {} ", i + 1, icons[i], tab.title())
            } else {
                format!(" {} {} ", i + 1, icons[i])
            }
        })
        .collect()
}

fn draw_sidebar(f: &mut Frame, app: &mut App, area: Rect) {
    let p = app.palette;
    let narrow = area.width <= 6;
    f.render_widget(
        Paragraph::new(if narrow {
            " ♫ MP"
        } else {
            " ♫ MusicPlayer"
        })
        .fg(p.accent)
        .bold(),
        Rect::new(area.x, area.y, area.width, 1),
    );
    app.areas.tabs.clear();
    let height = if area.height >= 20 { 3 } else { 2 };
    for (i, (tab, label)) in TABS.into_iter().zip(tab_labels(area.width)).enumerate() {
        let rect = Rect::new(area.x, area.y + 1 + i as u16 * height, area.width, height)
            .intersection(area);
        f.render_widget(
            Paragraph::new(if area.height < 14 {
                label
            } else {
                format!("\n{label}")
            })
            .style(if tab == app.tab {
                Style::new().bg(p.selected).fg(p.accent).bold()
            } else {
                Style::new()
            }),
            rect,
        );
        app.areas.tabs.push(rect);
    }
    let (account, color) = if app.account_checking {
        (format!("◌ {}", spinner(app)), p.dim)
    } else if app.logged_in {
        ("● YouTube Music".into(), p.cool)
    } else {
        ("○ Sin cuenta".into(), p.dim)
    };
    f.render_widget(
        Paragraph::new(if narrow {
            account.chars().take(1).collect::<String>()
        } else {
            account
        })
        .fg(color),
        Rect::new(area.x, area.bottom().saturating_sub(1), area.width, 1),
    );
}

const COLUMN_TITLES: [&str; 4] = ["Canciones", "Artistas", "Álbumes", "Playlists"];

fn grid_columns(width: u16) -> usize {
    if width < 60 {
        1
    } else if width < 100 {
        2
    } else {
        4
    }
}

fn draw_results(f: &mut Frame, app: &mut App, area: Rect) {
    let p = app.palette;
    let artist = app.yt_view.artist_name.is_some();
    let titles: &[&str] = if artist {
        &["Populares", "Álbumes", "Playlists"][..app.yt_view.column_count()]
    } else {
        &COLUMN_TITLES
    };
    let [top, mut body] = Layout::vertical([
        Constraint::Length(if artist { 4 } else { 3 }),
        Constraint::Min(0),
    ])
    .areas(area);
    let (title, text) = if let Some(name) = &app.yt_view.artist_name {
        (
            format!(" {name} "),
            format!(
                "{} canciones populares · {} álbumes\nEnter en un álbum para escucharlo completo",
                app.yt_view.songs,
                app.yt_view.columns[1].len()
            ),
        )
    } else {
        app.areas.top = top;
        (
            " Resultado principal ".into(),
            match &app.yt_view.top {
                Some(mp_ytmusic::Top::Artist(a)) => format!("{} · {}", a.title, a.subtitle),
                Some(mp_ytmusic::Top::Track(t)) => format!("{} — {}", t.title, t.artist),
                None => "Sin resultado principal".into(),
            },
        )
    };
    let title = if app.message.is_some() {
        format!("{title} {}", spinner(app))
    } else {
        title
    };
    f.render_widget(
        Paragraph::new(text).block(Block::bordered().title(title)),
        top,
    );
    let count = grid_columns(body.width).min(titles.len());
    if count == 1 {
        let [switch, rest] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(body);
        body = rest;
        app.areas.switcher = switch;
        f.render_widget(
            Paragraph::new(format!("◀ {} ▶ · Tab", titles[app.yt_view.focus])).fg(p.accent),
            switch,
        );
    }
    let rows = if count == 1 {
        1
    } else {
        titles.len().div_ceil(count)
    };
    let bands = Layout::vertical(vec![Constraint::Fill(1); rows]).split(body);
    for (r, band) in bands.iter().enumerate() {
        let cells = Layout::horizontal(vec![Constraint::Fill(1); count]).split(*band);
        for (c, rect) in cells.iter().enumerate() {
            let index = if count == 1 {
                app.yt_view.focus
            } else {
                r * count + c
            };
            if index >= titles.len() {
                continue;
            }
            let focused = index == app.yt_view.focus;
            let block = Block::bordered()
                .title(format!(" {} ", titles[index]))
                .border_style(Style::new().fg(if focused { p.accent } else { p.dim }));
            app.areas.columns.push((index, block.inner(*rect)));
            let rows: Vec<_> = app.yt_view.columns[index]
                .iter()
                .enumerate()
                .map(|(i, row)| {
                    let (marker, title, subtitle) = row.label();
                    // The column title already names the kind: keep only its icon.
                    let marker = if index == 0 && i >= app.yt_view.songs {
                        "▷"
                    } else {
                        marker.split(' ').next().unwrap_or_default()
                    };
                    let subtitle = subtitle.strip_prefix("Artista · ").unwrap_or(&subtitle);
                    Row::new([format!("{marker} {title} — {subtitle}")]).style(stripe(i, p))
                })
                .collect();
            let table = Table::new(rows, [Constraint::Fill(1)])
                .block(block)
                .row_highlight_style(Style::new().bg(p.selected).fg(p.accent))
                .highlight_symbol("› ");
            let state = if focused {
                &mut app.table
            } else {
                &mut app.yt_view.selections[index]
            };
            f.render_stateful_widget(table, *rect, state);
        }
    }
}

fn draw_local(f: &mut Frame, app: &mut App, area: Rect) {
    use crate::local::{Page, Row as LocalRow};
    let p = app.palette;
    let mut body = area;
    if let Page::Artist(name) = &app.local.state.page {
        let albums: Vec<_> = app.albums.iter().filter(|a| a.artist == *name).collect();
        let songs: usize = albums.iter().map(|a| a.tracks.len()).sum();
        let [header, rest] =
            Layout::vertical([Constraint::Length(4), Constraint::Min(0)]).areas(body);
        body = rest;
        f.render_widget(
            Paragraph::new(format!(
                "{} · {songs} canciones\nTab: columna · Enter: abrir álbum / reproducir canción",
                crate::local::plural(albums.len(), "álbum", "álbumes")
            ))
            .block(Block::bordered().title(app.local.breadcrumb(&app.albums))),
            header,
        );
    }
    let artist = app.local_artist();
    let count = if artist {
        grid_columns(body.width).min(2)
    } else {
        1
    };
    if artist && count == 1 {
        let [switch, rest] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(body);
        body = rest;
        app.areas.switcher = switch;
        f.render_widget(
            Paragraph::new(format!(
                "◀ {} ▶ · Tab",
                ["Canciones", "Álbumes"][app.local.state.focus]
            ))
            .fg(p.accent),
            switch,
        );
    }
    let rects = Layout::horizontal(vec![Constraint::Fill(1); count]).split(body);
    for (i, rect) in rects.iter().enumerate() {
        let column = if artist && count == 1 {
            app.local.state.focus
        } else {
            i
        };
        let title = if artist {
            ["Canciones", "Álbumes"][column].to_string()
        } else {
            format!(
                "{} · Orden: {} (o)",
                app.local.breadcrumb(&app.albums),
                app.local.order.label()
            )
        };
        let block = Block::bordered().title(title).border_style(Style::new().fg(
            if column == app.local.state.focus {
                p.accent
            } else {
                p.dim
            },
        ));
        if artist {
            app.areas.columns.push((column, block.inner(*rect)));
        } else {
            app.areas.list = block.inner(*rect);
        }
        let rows: Vec<_> = app
            .local_rows(column)
            .iter()
            .enumerate()
            .map(|(index, row)| {
                let text = match row {
                    LocalRow::Album(a) if artist => {
                        crate::local::album_label(&app.albums[*a], true)
                    }
                    _ => crate::local::label(row, &app.albums, &app.library, &app.folder),
                };
                let style = if matches!(row, LocalRow::Disc(_)) {
                    Style::new().fg(p.accent).bold()
                } else {
                    stripe(index, p)
                };
                Row::new([text]).style(style)
            })
            .collect();
        let table = Table::new(rows, [Constraint::Fill(1)])
            .block(block)
            .row_highlight_style(Style::new().bg(p.selected).fg(p.accent))
            .highlight_symbol("› ");
        // Non-column mouse mapping reserves the first inner line for a table header.
        let table = if artist {
            table
        } else {
            table.header(Row::new(["Enter: abrir / reproducir"]).fg(p.dim))
        };
        let state = if column == app.local.state.focus {
            &mut app.table
        } else {
            &mut app.local.state.tables[column]
        };
        f.render_stateful_widget(table, *rect, state);
    }
}

fn scroll_indicator(selected: Option<usize>, total: usize) -> String {
    format!(
        "{}/{}",
        selected
            .filter(|_| total > 0)
            .map_or(0, |i| i.min(total - 1) + 1),
        total
    )
}

fn spinner(app: &App) -> &'static str {
    ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
        [(app.started.elapsed().as_millis() / 80 % 10) as usize]
}

fn track_cells(track: &mp_core::Track, index: usize, playing: bool) -> [String; 5] {
    [
        if playing {
            "▶".into()
        } else {
            (index + 1).to_string()
        },
        track.title.clone(),
        track.artist.clone(),
        track.album.clone(),
        track.duration.map(fmt_time).unwrap_or_else(|| "—".into()),
    ]
}

fn stripe(index: usize, p: crate::palette::Palette) -> Style {
    Style::new().fg(p.text).bg(if index % 2 == 1 {
        p.zebra
    } else {
        p.background
    })
}

fn lyrics_lines(text: &str, width: u16) -> Vec<String> {
    let mut rows = Vec::new();
    for line in text.lines() {
        let mut row = String::new();
        let mut used = 0;
        for ch in line.chars() {
            let size = Span::raw(ch.to_string()).width();
            if used + size > width.max(1) as usize && !row.is_empty() {
                rows.push(std::mem::take(&mut row));
                used = 0;
            }
            row.push(ch);
            used += size;
        }
        rows.push(row);
    }
    rows
}

fn draw_lyrics(f: &mut Frame, app: &mut App, area: Rect) {
    let p = app.palette;
    let spin = spinner(app);
    let panel = app.lyrics.as_mut().unwrap();
    let text = if panel.loading {
        "Buscando letra…"
    } else {
        panel
            .text
            .as_deref()
            .unwrap_or("No hay letra para esta canción")
    };
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(p.dim))
        .title(format!(
            " {}Letra · {} ",
            if panel.loading {
                format!("{spin} ")
            } else {
                String::new()
            },
            panel.title
        ))
        .padding(ratatui::widgets::Padding::new(3, 3, 1, 0));
    let inner = block.inner(area);
    app.areas.list = inner;
    let lines = lyrics_lines(text, inner.width);
    panel.scroll = panel
        .scroll
        .min((lines.len().min(u16::MAX as usize) as u16).saturating_sub(inner.height));
    let paragraph = Paragraph::new(lines.into_iter().map(Line::from).collect::<Vec<_>>());
    f.render_widget(paragraph.scroll((panel.scroll, 0)).block(block), area);
}

fn draw_list(f: &mut Frame, app: &mut App, area: Rect) {
    let p = app.palette;
    if app.lyrics.is_some() {
        draw_lyrics(f, app, area);
        return;
    }
    let playing_uri = app.now_playing().map(|t| t.uri.clone());
    let row_style = |uri: &str| {
        if Some(uri) == playing_uri.as_deref() {
            Style::new().fg(p.accent).bold()
        } else {
            Style::new()
        }
    };

    let (header, widths, rows, title): (Row, Vec<Constraint>, Vec<Row>, String) = match app.tab {
        Tab::Library | Tab::Queue => {
            let tracks: Vec<_> = if app.is_local() {
                app.filtered().into_iter().cloned().collect()
            } else {
                app.queue.clone()
            };
            let rows = tracks
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    let playing = Some(&t.uri) == playing_uri.as_ref();
                    Row::new(track_cells(t, i, playing))
                        .style(stripe(i, p).patch(row_style(&t.uri)))
                })
                .collect();
            (
                Row::new(["#", "Título", "Artista", "Álbum", "Duración"]),
                vec![
                    Constraint::Length(4),
                    Constraint::Fill(3),
                    Constraint::Fill(2),
                    Constraint::Fill(2),
                    Constraint::Length(8),
                ],
                rows,
                format!(
                    " {} · {} canciones ",
                    if app.is_local() {
                        app.local.breadcrumb(&app.albums)
                    } else {
                        app.tab.title().into()
                    },
                    tracks.len()
                ),
            )
        }
        Tab::Youtube | Tab::Playlists => {
            let view = app.remote_view();
            let rows = view
                .rows
                .iter()
                .enumerate()
                .map(|(i, r)| {
                    let cells = match r {
                        crate::youtube::YoutubeRow::Track(t) => {
                            track_cells(t, i, Some(&t.uri) == playing_uri.as_ref())
                        }
                        _ => {
                            let (marker, title, subtitle) = r.label();
                            [
                                marker.chars().next().unwrap_or(' ').to_string(),
                                title,
                                subtitle,
                                String::new(),
                                String::new(),
                            ]
                        }
                    };
                    let style = match r {
                        crate::youtube::YoutubeRow::Track(t) => {
                            stripe(i, p).patch(row_style(&t.uri))
                        }
                        _ => stripe(i, p),
                    };
                    Row::new(cells).style(style)
                })
                .collect();
            (
                Row::new(["#", "Título", "Artista", "Álbum", "Duración"]),
                vec![
                    Constraint::Length(4),
                    Constraint::Fill(3),
                    Constraint::Fill(2),
                    Constraint::Fill(2),
                    Constraint::Length(8),
                ],
                rows,
                format!(
                    " {} ",
                    view.opened
                        .as_ref()
                        .map(|(title, _, _)| title.as_str())
                        .unwrap_or(
                            if app.tab == Tab::Youtube && !app.yt_view.searched && app.logged_in {
                                "Tu biblioteca"
                            } else {
                                app.tab.title()
                            }
                        )
                ),
            )
        }

        Tab::Equalizer => unreachable!("EQ renders separately"),
        Tab::Albums => {
            let rows = app
                .albums
                .iter()
                .enumerate()
                .map(|(i, a)| {
                    let total: f64 = a.tracks.iter().filter_map(|t| t.duration).sum();
                    Row::new(vec![
                        Cell::from(a.name.clone()),
                        Cell::from(a.artist.clone()).fg(p.dim),
                        Cell::from(format!(
                            "{} canci{}",
                            a.tracks.len(),
                            if a.tracks.len() == 1 { "ón" } else { "ones" }
                        ))
                        .fg(p.dim),
                        Cell::from(fmt_time(total)).fg(p.dim),
                    ])
                    .style(stripe(i, p))
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

    let title = format!(
        "{title}· {} {}",
        scroll_indicator(app.table.selected(), rows.len()),
        if app.message.is_some() {
            spinner(app)
        } else {
            ""
        }
    );
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(p.dim))
        .title(title.bold());
    app.areas.list = block.inner(area);

    if matches!(app.tab, Tab::Youtube | Tab::Playlists) {
        let error = app.remote_error();
        let message = error;
        if let Some(message) = message {
            f.render_widget(
                Paragraph::new(message.replace(['\n', '\r'], " "))
                    .fg(if error.is_some() { p.hot } else { p.dim })
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
            Tab::Youtube if !app.logged_in => "Pulsa / para buscar. Para conectar: Conectar cuenta en la app de escritorio; la sesión se comparte.".into(),
            Tab::Youtube => "Tu biblioteca está vacía. Pulsa / para buscar o r para recargar.".into(),
            Tab::Playlists if !app.logged_in => {
                "Inicia sesión en la app de escritorio (Conectar cuenta); la sesión se comparte."
                    .into()
            }
            Tab::Playlists => "No hay canciones en esta colección.".into(),
            Tab::Queue => "La cola está vacía. Elige algo en Biblioteca o Álbumes.".to_string(),
            _ if app.scan.is_some() => {
                let (done, total) = app.scan.unwrap_or_default();
                if total == 0 {
                    "Buscando archivos de música…".into()
                } else {
                    let width = 30;
                    let full = done * width / total.max(1);
                    format!(
                        "Escaneando tu biblioteca…\n\n{}{}  {done} / {total}\n\nLa primera lectura puede tardar; las siguientes son casi instantáneas.",
                        "█".repeat(full),
                        "░".repeat(width - full)
                    )
                }
            }
            _ if !app.filter.is_empty() => format!("Nada coincide con “{}”.", app.filter),
            _ => format!(
                "No hay música en {}.\nEjecuta: musicplayer-tui <carpeta>",
                app.folder.display()
            ),
        };
        f.render_widget(
            Paragraph::new(msg)
                .wrap(Wrap { trim: true })
                .fg(p.dim)
                .alignment(Alignment::Center)
                .block(block),
            area,
        );
        return;
    }

    let table = Table::new(rows, widths)
        .header(header.style(Style::new().fg(p.dim).add_modifier(Modifier::UNDERLINED)))
        .row_highlight_style(
            Style::new()
                .bg(p.selected)
                .fg(p.accent)
                .remove_modifier(Modifier::DIM),
        )
        .highlight_symbol("› ")
        .column_spacing(2)
        .block(block);
    f.render_stateful_widget(table, area, &mut app.table);
}

fn draw_now(f: &mut Frame, app: &mut App, area: Rect) {
    let p = app.palette;
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(p.accent))
        .title(" Reproduciendo ".bold().fg(p.accent));
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
        Line::from(msg.as_str().fg(p.hot))
    } else {
        match app.now_playing() {
            Some(t) => Line::from(vec![
                Span::raw(&t.title).bold(),
                Span::raw("  —  ").fg(p.dim),
                Span::raw(&t.artist),
                Span::raw(" · ").fg(p.dim),
                Span::raw(&t.album).fg(p.dim),
            ]),
            None => Line::from("Nada en reproducción — pulsa Enter en una canción".fg(p.dim)),
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
    f.render_widget(meter(ratio, bar.width, p.accent, p), bar);
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
        f.render_widget(spectrum(&app.bars, viz, p), viz);
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
            Style::new().fg(p.ink).bg(p.accent).bold()
        } else {
            Style::new().fg(p.text)
        };
        f.render_widget(
            Paragraph::new(label)
                .alignment(Alignment::Center)
                .style(style)
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .border_style(if primary {
                            style
                        } else {
                            Style::new().fg(p.dim)
                        }),
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
    f.render_widget(meter(s.volume / 100.0, vol_bar.width, p.cool, p), vol_bar);
    app.areas.volume = vol_bar;
}

/// Vertical bars with eighth-block resolution, centred in `area`.
/// Narrow areas merge neighbouring bands (loudest wins) so every band stays represented.
fn spectrum(bands: &[f32], area: Rect, p: crate::palette::Palette) -> Paragraph<'static> {
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
                let color = if p.id == "classic" {
                    if row == 0 {
                        p.hot
                    } else if row * 2 < rows {
                        p.warm
                    } else {
                        p.cool
                    }
                } else if i * 2 < bars.len() {
                    p.warm
                } else {
                    p.cool
                };
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
fn meter(ratio: f64, width: u16, color: Color, p: crate::palette::Palette) -> Paragraph<'static> {
    let filled = (ratio.clamp(0.0, 1.0) * width as f64).round() as usize;
    Paragraph::new(Line::from(vec![
        Span::raw("█".repeat(filled)).fg(color),
        Span::raw("░".repeat(width as usize - filled)).fg(p.selected),
    ]))
}

fn help_groups(app: &App) -> Vec<(&'static str, &'static str)> {
    if app.lyrics.is_some() {
        return vec![
            ("Esc/l", "cerrar letra"),
            ("↑/↓", "desplazar"),
            ("Espacio", "pausar"),
            ("q", "salir"),
        ];
    }
    if app.searching {
        return vec![
            (
                "Enter",
                if app.tab == Tab::Youtube {
                    "buscar"
                } else {
                    "listo"
                },
            ),
            ("Esc", "cancelar"),
        ];
    }
    if app.tab == Tab::Equalizer {
        return vec![
            ("←/→", "banda"),
            ("↑/↓", "±0.5 dB"),
            ("⇧", "±2"),
            ("0", "cero"),
            ("o", "ON/OFF"),
            ("p/P", "preset"),
            ("j/k", "lista"),
            ("Enter", "aplicar"),
            ("t", "tema"),
            ("1-6", "vista"),
            ("q", "salir"),
        ];
    }
    let mut groups = vec![
        ("←/→", "±5 s"),
        ("Enter", "abrir/reproducir"),
        ("Espacio", "pausar"),
        ("l", "letra"),
        ("q", "salir"),
    ];
    if matches!(app.tab, Tab::Youtube | Tab::Playlists) {
        groups.push(("r", "recargar"));
        if app.remote_view().opened.is_some() {
            groups.insert(1, ("p", "álbum completo"));
        }
        if !app.remote_view().history.is_empty() {
            groups.insert(1, ("Esc/⌫", "volver"));
        }
    }
    if app.search_results() {
        groups.insert(1, ("Tab/⇧Tab", "columna"));
    }
    if app.is_local() {
        groups.insert(0, ("v", "subvista"));
        groups.insert(1, ("/", "filtrar"));
        groups.insert(2, ("Esc/⌫", "volver"));
        if app.local_artist() {
            groups.insert(2, ("Tab", "columna"));
        }
        if matches!(app.local.state.page, crate::local::Page::Albums) {
            groups.insert(2, ("o", "orden"));
        }
        if matches!(
            app.local.state.page,
            crate::local::Page::Album(_) | crate::local::Page::Folders(_)
        ) {
            groups.insert(2, ("p", "reproducir todo"));
        }
        if matches!(app.local.state.page, crate::local::Page::Album(_)) {
            groups.insert(2, ("s", "mezclar"));
        }
    }
    if !app.is_local() {
        groups.push(("/", "buscar"));
    }
    groups.extend([("1-6", "vista"), ("t", "tema"), ("+/-", "volumen")]);
    groups
}

fn fit_hints<'a>(groups: &[(&'a str, &'a str)], width: u16) -> Vec<(&'a str, &'a str)> {
    let mut remaining = width as usize;
    groups
        .iter()
        .copied()
        .filter(|(key, label)| {
            let length = Line::from(format!(" {key} {label}  ")).width();
            if length <= remaining {
                remaining -= length;
                true
            } else {
                false
            }
        })
        .collect()
}

fn draw_help(f: &mut Frame, app: &App, area: Rect) {
    let p = app.palette;
    let groups = help_groups(app);
    let chunks: Vec<_> = if area.height > 1 {
        vec![&groups[..6], &groups[6..]]
    } else {
        vec![groups.as_slice()]
    };
    let lines: Vec<_> = chunks
        .into_iter()
        .map(|groups| {
            let spans: Vec<_> = fit_hints(groups, area.width)
                .into_iter()
                .flat_map(|(key, label)| {
                    [
                        Span::raw(format!(" {key} ")).fg(p.ink).bg(p.dim),
                        Span::raw(format!("{label}  ")).fg(p.dim),
                    ]
                })
                .collect();
            Line::from(spans)
        })
        .collect();
    f.render_widget(Paragraph::new(lines), area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tests::{app, track};
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn equalizer_renders_labels_bars_and_hit_areas() {
        for (width, height) in [(80, 24), (160, 45)] {
            let mut a = app(Vec::new());
            a.set_tab(Tab::Equalizer);
            a.eq.apply_preset(13);
            let mut term = Terminal::new(TestBackend::new(width, height)).unwrap();
            term.draw(|f| draw(f, &mut a)).unwrap();
            let b = term.backend().buffer();
            let screen = (0..height)
                .map(|y| (0..width).map(|x| b[(x, y)].symbol()).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n");
            for label in crate::equalizer::LABELS.into_iter().chain([
                "+12",
                "-12",
                "ON",
                "Rock",
                "██",
                "Preajustes",
            ]) {
                assert!(screen.contains(label), "missing {label}: {screen}");
            }
            assert_eq!(a.areas.eq_bars.len(), 11);
            for r in &a.areas.eq_bars {
                assert!(r.height > 1);
                assert!((r.y..r.bottom()).any(|y| b[(r.x, y)].symbol() == "█"));
            }
            assert!(!a.areas.eq_presets.is_empty());
            assert!(!screen.contains("±5 s"));
        }
    }

    #[test]
    fn local_artist_and_multidisc_album_render_at_supported_sizes() {
        use crate::local::Page;
        for (width, height) in [(80, 24), (160, 45)] {
            let mut a = app(crate::local::tests::discography());
            a.open_local(Page::Artists);
            a.play_selected();
            assert!(a.local_artist());
            assert_eq!(a.column_focus(), 1);
            let mut term = Terminal::new(TestBackend::new(width, height)).unwrap();
            term.draw(|f| draw(f, &mut a)).unwrap();
            let screen = term
                .backend()
                .buffer()
                .content
                .iter()
                .map(|c| c.symbol())
                .collect::<String>();
            for text in [
                "Opeth",
                "2 álbumes · 4 canciones",
                "Canciones",
                "Álbumes",
                "1999 · Still Life · 3 canciones",
            ] {
                assert!(screen.contains(text), "missing {text}: {screen}");
            }
            assert_eq!(a.areas.columns.len(), 2);
            a.table.select(Some(1));
            a.play_selected();
            term.draw(|f| draw(f, &mut a)).unwrap();
            let screen = term
                .backend()
                .buffer()
                .content
                .iter()
                .map(|c| c.symbol())
                .collect::<String>();
            for text in [
                "Still Life",
                "Disco 1",
                "Disco 2",
                "Inicio",
                "Primera",
                "Segunda",
            ] {
                assert!(screen.contains(text), "missing {text}: {screen}");
            }
            assert!(a.areas.columns.is_empty());
            a.back();
            assert!(a.local_artist());
            assert_eq!(a.table.selected(), Some(1));
        }
    }

    #[test]
    fn artist_header_and_albums_render_at_supported_sizes() {
        for (width, height) in [(80, 24), (160, 45)] {
            let mut a = app(Vec::new());
            a.set_tab(Tab::Youtube);
            a.yt_view.artist(crate::youtube::tests::artist_page(), 0);
            a.table = a.yt_view.selections[a.yt_view.focus];
            let mut term = Terminal::new(TestBackend::new(width, height)).unwrap();
            term.draw(|f| draw(f, &mut a)).unwrap();
            let buffer = term.backend().buffer();
            let screen = (0..height)
                .map(|y| {
                    (0..width)
                        .map(|x| buffer[(x, y)].symbol())
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("\n");
            for text in [
                "Artista de prueba",
                "1 canciones populares · 2 álbumes",
                "Enter en un álbum para escucharlo completo",
                " Álbumes ",
                " Populares ",
            ] {
                assert!(screen.contains(text), "{screen}");
            }
            assert_eq!(a.areas.columns.len(), 2);
            assert!(a.areas.columns.iter().all(|(index, _)| *index < 2));
        }
    }

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
        assert!(screen.contains("▶"));
        assert_eq!(a.areas.tabs.len(), 6);
        assert!(a.areas.buttons.iter().all(|r| r.height == 3));
        assert!(a.areas.progress.width > 0 && a.areas.volume.width > 0);
    }

    #[test]
    fn sidebar_and_grid_breakpoints() {
        assert_eq!(sidebar_width(89), 6);
        assert_eq!(sidebar_width(90), 22);
        assert_eq!(tab_labels(22)[0], " 1 ♫ Biblioteca ");
        assert_eq!(tab_labels(6)[0], " 1 ♫ ");
        assert_eq!(
            (grid_columns(59), grid_columns(60), grid_columns(100)),
            (1, 2, 4)
        );
    }

    #[test]
    fn track_rows_share_columns_and_duration() {
        let t = track("Título", "Artista", "Álbum");
        assert_eq!(
            track_cells(&t, 11, false),
            ["12", "Título", "Artista", "Álbum", "2:05"]
        );
        assert_eq!(track_cells(&t, 0, true)[0], "▶");
        assert_eq!(scroll_indicator(Some(11), 240), "12/240");
        assert_eq!(scroll_indicator(Some(0), 0), "0/0");
        assert_eq!(scroll_indicator(Some(999), 3), "3/3");
    }

    #[test]
    fn hints_fit_only_whole_groups() {
        let groups = [("Enter", "reproducir"), ("q", "salir")];
        assert_eq!(fit_hints(&groups, 10), [("q", "salir")]);
        assert!(fit_hints(&groups, 3).is_empty());
    }

    #[test]
    fn lyrics_wrap_without_losing_text() {
        assert_eq!(
            lyrics_lines("Canción\nsegunda línea", 7),
            ["Canción", "segunda", " línea"]
        );
    }

    #[test]
    fn sidebar_search_and_columns_fit_supported_sizes() {
        for (width, height) in [(80, 24), (120, 35), (160, 45)] {
            let mut a = app(Vec::new());
            a.set_tab(Tab::Youtube);
            a.yt_view.search(mp_ytmusic::SearchResults::default());
            let mut term = Terminal::new(TestBackend::new(width, height)).unwrap();
            term.draw(|f| draw(f, &mut a)).unwrap();
            let buffer = term.backend().buffer();
            let screen = (0..height)
                .map(|y| {
                    (0..width)
                        .map(|x| buffer[(x, y)].symbol())
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("\n");
            for label in tab_labels(sidebar_width(width)) {
                assert!(screen.contains(label.trim()), "{screen}");
            }
            for title in COLUMN_TITLES {
                assert!(screen.contains(title), "{screen}");
            }
            assert!(screen.contains("Busca canciones, artistas, álbumes, playlists…"));
            assert!(screen.contains("Resultado principal"));
            assert_eq!(a.areas.columns.len(), 4);
            assert!(a.areas.tabs.iter().all(|r| r.height >= 2));
        }
    }

    #[test]
    fn spectrum_bars_fill_from_the_bottom() {
        let mut bars = [0.0; 19];
        bars[0] = 1.0;
        bars[18] = 0.5;
        let p = spectrum(
            &bars,
            Rect::new(0, 0, 38, 3),
            crate::palette::lookup("musicplayer"),
        );
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
        term.draw(|f| {
            f.render_widget(
                spectrum(&bars, f.area(), crate::palette::lookup("musicplayer")),
                f.area(),
            )
        })
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
