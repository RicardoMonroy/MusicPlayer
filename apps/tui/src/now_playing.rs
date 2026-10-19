use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, mpsc},
};

use image::DynamicImage;
use mp_core::Track;
use mp_ytmusic::YtMusic;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Paragraph},
};
use ratatui_image::{StatefulImage, picker::Picker, protocol::StatefulProtocol};
use tokio::runtime::Runtime;

#[derive(Default, Debug, PartialEq)]
pub enum Load<T> {
    #[default]
    Idle,
    Loading,
    Ready(T),
    Error(String),
}

#[derive(Default)]
pub struct Entry {
    pub cover: Load<Option<DynamicImage>>,
    pub lyrics: Load<Option<String>>,
    pub translation: Load<String>,
}

enum ResultMsg {
    Cover(String, Option<DynamicImage>),
    Lyrics(String, Result<Option<String>, String>),
    Translation(String, Result<String, String>),
}

pub struct Panel {
    pub picker: Picker,
    pub visible: bool,
    pub focused: bool,
    pub pixel: bool,
    pub lyrics_visible: bool,
    pub translate: bool,
    pub requested_width: Option<u16>,
    pub uri: Option<String>,
    pub cache: HashMap<String, Entry>,
    pub scroll: u16,
    protocol: Option<StatefulProtocol>,
    channel: (mpsc::Sender<ResultMsg>, mpsc::Receiver<ResultMsg>),
}

impl Default for Panel {
    fn default() -> Self {
        Self {
            picker: Picker::halfblocks(),
            visible: true,
            focused: false,
            pixel: false,
            lyrics_visible: true,
            translate: false,
            requested_width: None,
            uri: None,
            cache: HashMap::new(),
            scroll: 0,
            protocol: None,
            channel: mpsc::channel(),
        }
    }
}

impl Panel {
    pub fn width(&self, terminal: u16, available: u16) -> u16 {
        if !self.visible || terminal < 100 || available < 56 {
            return 0;
        }
        self.requested_width
            .unwrap_or((u32::from(available) * 34 / 100) as u16)
            .clamp(28, available / 2)
    }

    pub fn resize(&mut self, delta: i16, current: u16) {
        if current > 0 {
            self.requested_width = Some(current.saturating_add_signed(delta).max(28));
        }
    }

    pub fn toggle_pixel(&mut self) {
        self.pixel = !self.pixel;
        self.protocol = None;
    }

    pub fn move_by(&mut self, delta: isize) {
        self.scroll = (self.scroll as usize)
            .saturating_add_signed(delta)
            .min(u16::MAX as usize) as u16;
    }

    pub fn loading(&self) -> bool {
        self.uri
            .as_ref()
            .and_then(|uri| self.cache.get(uri))
            .is_some_and(|e| {
                matches!(e.cover, Load::Loading)
                    || matches!(e.lyrics, Load::Loading)
                    || matches!(e.translation, Load::Loading)
            })
    }

    fn apply(&mut self, result: ResultMsg) {
        match result {
            ResultMsg::Cover(uri, image) => {
                if self.uri.as_ref() == Some(&uri) {
                    self.protocol = None;
                }
                self.cache.entry(uri).or_default().cover = Load::Ready(image);
            }
            ResultMsg::Lyrics(uri, text) => {
                self.cache.entry(uri).or_default().lyrics = match text {
                    Ok(text) => Load::Ready(text.filter(|s| !s.trim().is_empty())),
                    Err(e) => Load::Error(e),
                };
            }
            ResultMsg::Translation(uri, text) => {
                self.cache.entry(uri).or_default().translation = match text {
                    Ok(text) => Load::Ready(text),
                    Err(e) => Load::Error(e),
                };
            }
        }
    }

    pub fn tick(
        &mut self,
        track: Option<&Track>,
        runtime: Option<&Runtime>,
        youtube: Option<Arc<YtMusic>>,
    ) {
        while let Ok(result) = self.channel.1.try_recv() {
            self.apply(result);
        }
        let uri = track.map(|t| t.uri.clone());
        if self.uri != uri {
            self.uri = uri;
            self.scroll = 0;
            self.protocol = None;
        }
        let Some(track) = track else {
            return;
        };
        let entry = self.cache.entry(track.uri.clone()).or_default();
        let Some(runtime) = runtime else {
            entry.cover = Load::Ready(None);
            entry.lyrics = Load::Error("No se pudo iniciar Tokio".into());
            return;
        };
        if matches!(entry.cover, Load::Idle) {
            entry.cover = Load::Loading;
            let track = track.clone();
            let tx = self.channel.0.clone();
            runtime.spawn(async move {
                let image = if track.uri.starts_with("https://") {
                    let bytes = async {
                        let url = track.cover.as_ref()?;
                        reqwest::Client::builder()
                            .timeout(std::time::Duration::from_secs(20))
                            .build()
                            .ok()?
                            .get(url)
                            .send()
                            .await
                            .ok()?
                            .error_for_status()
                            .ok()?
                            .bytes()
                            .await
                            .ok()
                    }
                    .await;
                    tokio::task::spawn_blocking(move || {
                        bytes.and_then(|b| image::load_from_memory(&b).ok())
                    })
                    .await
                    .ok()
                    .flatten()
                } else {
                    let uri = track.uri.clone();
                    tokio::task::spawn_blocking(move || {
                        mp_local::cover(Path::new(&uri))
                            .and_then(|(_, b)| image::load_from_memory(&b).ok())
                    })
                    .await
                    .ok()
                    .flatten()
                };
                let _ = tx.send(ResultMsg::Cover(track.uri, image));
            });
        }
        if matches!(entry.lyrics, Load::Idle) {
            entry.lyrics = Load::Loading;
            let uri = track.uri.clone();
            let tx = self.channel.0.clone();
            runtime.spawn(async move {
                let result = if uri.starts_with("https://") {
                    match youtube {
                        Some(yt) => yt
                            .lyrics(&uri)
                            .await
                            .map_err(|e| format!("No se pudo cargar la letra: {e}")),
                        None => Err("YouTube Music no está disponible".into()),
                    }
                } else {
                    let path = uri.clone();
                    tokio::task::spawn_blocking(move || mp_local::lyrics(Path::new(&path)))
                        .await
                        .map_err(|e| format!("No se pudo cargar la letra: {e}"))
                };
                let _ = tx.send(ResultMsg::Lyrics(uri, result));
            });
        }
        if self.translate
            && matches!(entry.translation, Load::Idle)
            && let Load::Ready(Some(text)) = &entry.lyrics
        {
            let text = text.clone();
            entry.translation = Load::Loading;
            let uri = track.uri.clone();
            let tx = self.channel.0.clone();
            runtime.spawn(async move {
                let result = mp_ytmusic::translate::to_spanish(&uri, &text).await;
                let _ = tx.send(ResultMsg::Translation(uri, result));
            });
        }
    }
}

/// Wrap each pair together so the next original line starts beside its translation.
fn aligned(original: &str, translated: &str, left: u16, right: u16) -> (Vec<String>, Vec<String>) {
    let a: Vec<_> = original.split('\n').collect();
    let b: Vec<_> = translated.split('\n').collect();
    let mut out = (Vec::new(), Vec::new());
    for i in 0..a.len().max(b.len()) {
        let aa = crate::ui::lyrics_lines(a.get(i).copied().unwrap_or(""), left);
        let bb = crate::ui::lyrics_lines(b.get(i).copied().unwrap_or(""), right);
        for j in 0..aa.len().max(bb.len()).max(1) {
            out.0.push(aa.get(j).cloned().unwrap_or_default());
            out.1.push(bb.get(j).cloned().unwrap_or_default());
        }
    }
    out
}

pub fn draw(f: &mut Frame, app: &mut crate::app::App, area: Rect) {
    let p = app.palette;
    let track = app.now_playing().cloned();
    let panel = &mut app.panel;
    let block = Block::bordered()
        .title(" Reproduciendo ")
        .border_style(Style::new().fg(if panel.focused { p.accent } else { p.dim }));
    let inner = block.inner(area);
    f.render_widget(block, area);
    if inner.width == 0 || inner.height == 0 {
        return;
    }
    // Cells are ~2:1, so a square cover is width/2 rows; leave room for lyrics.
    let max_cover = if panel.lyrics_visible {
        inner.height * 45 / 100
    } else {
        inner.height.saturating_sub(7)
    };
    let height = (inner.width / 2).min(max_cover).max(1);
    let [cover_row, title, artist, body] = Layout::vertical([
        Constraint::Length(height),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(inner);
    let width = (height * 2).min(inner.width);
    let cover = Rect::new(
        cover_row.x + (cover_row.width - width) / 2,
        cover_row.y,
        width,
        cover_row.height,
    );
    let entry = panel.uri.as_ref().and_then(|uri| panel.cache.get(uri));
    if panel.protocol.is_none()
        && let Some(Entry {
            cover: Load::Ready(Some(image)),
            ..
        }) = entry
    {
        panel.protocol = Some(if panel.pixel {
            Picker::halfblocks().new_resize_protocol(image.clone())
        } else {
            panel.picker.new_resize_protocol(image.clone())
        });
    }
    if let Some(protocol) = &mut panel.protocol {
        f.render_stateful_widget(StatefulImage::default(), cover, protocol);
    } else {
        let block = Block::bordered().border_style(Style::new().fg(p.dim));
        f.render_widget(block, cover);
        f.render_widget(
            Paragraph::new("♪").centered().fg(p.accent),
            Rect::new(
                cover.x + 1,
                cover.y + cover.height / 2,
                cover.width.saturating_sub(2),
                1,
            ),
        );
    }
    f.render_widget(
        Paragraph::new(
            track
                .as_ref()
                .map_or("Sin reproducción", |t| t.title.as_str()),
        )
        .bold(),
        title,
    );
    f.render_widget(
        Paragraph::new(
            track
                .as_ref()
                .map_or(String::new(), |t| format!("{} · {}", t.artist, t.album)),
        )
        .fg(p.dim),
        artist,
    );
    if !panel.lyrics_visible {
        f.render_widget(Paragraph::new("l: mostrar letra").fg(p.dim), body);
        return;
    }
    let heading = if panel.translate {
        "Traducir al español"
    } else {
        "Letra"
    };
    let block = Block::bordered()
        .title(heading)
        .border_style(Style::new().fg(p.dim));
    let lyrics_area = block.inner(body);
    f.render_widget(block, body);
    let original = match entry.map(|e| &e.lyrics) {
        Some(Load::Ready(Some(text))) => text.as_str(),
        Some(Load::Error(e)) => e.as_str(),
        Some(Load::Loading) => "Buscando letra…",
        _ => "No hay letra para esta canción",
    };
    let translation = match entry.map(|e| &e.translation) {
        Some(Load::Ready(text)) if text == original => "La letra ya está en español.",
        Some(Load::Ready(text)) => text.as_str(),
        Some(Load::Error(e)) => e.as_str(),
        Some(Load::Loading) => "Traduciendo…",
        _ => "",
    };
    let [mut left, divider, mut right] = if panel.translate {
        Layout::horizontal([
            Constraint::Ratio(1, 2),
            Constraint::Length(1),
            Constraint::Ratio(1, 2),
        ])
        .areas(lyrics_area)
    } else {
        [lyrics_area, Rect::default(), Rect::default()]
    };
    if panel.translate {
        for (column, label, color) in [
            (&mut left, "Original", p.text),
            (&mut right, "Español", p.accent),
        ] {
            let [header, rest] =
                Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(*column);
            f.render_widget(Paragraph::new(label).bold().fg(color), header);
            *column = rest;
        }
    }
    let (a, b) = aligned(
        original,
        if panel.translate { translation } else { "" },
        left.width,
        right.width,
    );
    panel.scroll = panel
        .scroll
        .min((a.len().min(u16::MAX as usize) as u16).saturating_sub(left.height));
    let paragraph = |lines: Vec<String>| {
        Paragraph::new(lines.into_iter().map(Line::from).collect::<Vec<_>>())
            .scroll((panel.scroll, 0))
    };
    f.render_widget(paragraph(a), left);
    if panel.translate {
        f.render_widget(
            Paragraph::new("│\n".repeat(divider.height as usize)).fg(p.dim),
            divider,
        );
        f.render_widget(paragraph(b).fg(p.accent), right);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn width_clamps_and_visibility_survives_terminal_resize() {
        let mut panel = Panel::default();
        assert_eq!(panel.width(80, 74), 0);
        assert_eq!(panel.width(99, 77), 0);
        assert_eq!(panel.width(100, 78), 28);
        assert_eq!(panel.width(160, 138), 46);
        panel.resize(-100, 46);
        assert_eq!(panel.width(160, 138), 28);
        panel.resize(100, 28);
        assert_eq!(panel.width(160, 138), 69);
        assert_eq!(panel.width(120, 98), 49);
        assert_eq!(panel.width(80, 74), 0);
        assert_eq!(panel.width(160, 138), 69);
        panel.visible = false;
        assert_eq!(panel.width(160, 138), 0);
    }

    #[test]
    fn different_line_counts_and_wrapping_stay_aligned() {
        let (a, b) = aligned("long line\nnext\nlast", "uno\nsegunda línea", 4, 7);
        assert_eq!(a, ["long", "line", "next", "", "last"]);
        assert_eq!(b, ["uno", "", "segunda", "línea", ""]);
        let (a, b) = aligned("one", "uno\ndos\ntres", 10, 10);
        assert_eq!(a, ["one", "", ""]);
        assert_eq!(b, ["uno", "dos", "tres"]);
        let (a, b) = aligned("uno\n\nfin", "one\n\nend", 10, 10);
        assert_eq!(a[1], "");
        assert_eq!(b[1], "");
    }

    #[test]
    fn results_are_cached_by_uri_including_missing_covers_and_errors() {
        let mut panel = Panel {
            uri: Some("current".into()),
            ..Panel::default()
        };
        panel.cache.insert(
            "current".into(),
            Entry {
                cover: Load::Loading,
                lyrics: Load::Loading,
                ..Entry::default()
            },
        );
        assert!(panel.loading());
        panel.apply(ResultMsg::Cover(
            "previous".into(),
            Some(DynamicImage::new_rgb8(4, 4)),
        ));
        assert!(matches!(panel.cache["current"].cover, Load::Loading));
        assert!(matches!(
            panel.cache["previous"].cover,
            Load::Ready(Some(_))
        ));
        panel.apply(ResultMsg::Cover("current".into(), None));
        panel.apply(ResultMsg::Lyrics("current".into(), Ok(Some(" \n".into()))));
        assert!(!panel.loading());
        assert!(matches!(panel.cache["current"].cover, Load::Ready(None)));
        assert_eq!(panel.cache["current"].lyrics, Load::Ready(None));
        panel.apply(ResultMsg::Lyrics(
            "current".into(),
            Ok(Some("Hello".into())),
        ));
        panel.cache.get_mut("current").unwrap().translation = Load::Loading;
        assert!(panel.loading());
        let quota =
            "Se alcanzó el límite diario de traducciones gratuitas; vuelve a intentarlo mañana.";
        panel.apply(ResultMsg::Translation("current".into(), Err(quota.into())));
        assert_eq!(
            panel.cache["current"].translation,
            Load::Error(quota.into())
        );
        assert!(!panel.loading());
        panel.apply(ResultMsg::Translation("previous".into(), Ok("Hola".into())));
        assert_eq!(
            panel.cache["previous"].translation,
            Load::Ready("Hola".into())
        );
        panel.apply(ResultMsg::Lyrics(
            "previous".into(),
            Err("Error de letra".into()),
        ));
        assert_eq!(
            panel.cache["previous"].lyrics,
            Load::Error("Error de letra".into())
        );
    }

    fn screen(terminal: &Terminal<TestBackend>) -> String {
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect()
    }

    #[test]
    fn panel_renders_at_supported_sizes_and_hides_at_eighty_columns() {
        for (w, h) in [(120, 35), (160, 45), (80, 24)] {
            let mut app = crate::app::tests::app(Vec::new());
            let track = crate::app::tests::track("Canción de prueba", "Artista", "Álbum");
            app.queue = vec![track];
            app.status.index = Some(0);
            let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
            terminal.draw(|f| crate::ui::draw(f, &mut app)).unwrap();
            let text = screen(&terminal);
            assert!(text.contains("i: panel"));
            if w >= 100 {
                assert!(text.contains("Reproduciendo"));
                let panel_text: String = (0..app.areas.panel.bottom())
                    .flat_map(|y| (app.areas.panel.x..w).map(move |x| (x, y)))
                    .map(|pos| terminal.backend().buffer()[pos].symbol())
                    .collect();
                assert!(panel_text.contains("Canción de prueba"));
                assert!(panel_text.contains("Artista · Álbum"));
                assert!(panel_text.contains('♪'));
                assert!(app.areas.panel.width >= 28);
                assert!(
                    text.contains("c pixel")
                        && text.contains("T traducir")
                        && text.contains("[ ] ancho")
                );
                assert!(app.areas.panel.bottom() <= app.areas.buttons[0].y);
            } else {
                assert_eq!(app.areas.panel.width, 0);
                let content: String = (0..h.saturating_sub(10))
                    .flat_map(|y| (0..w).map(move |x| (x, y)))
                    .map(|pos| terminal.backend().buffer()[pos].symbol())
                    .collect();
                assert!(!content.contains("Reproduciendo"));
            }
        }
    }

    #[test]
    fn translation_messages_render_and_halfblocks_resize() {
        let mut app = crate::app::tests::app(Vec::new());
        let track = crate::app::tests::track("Título", "Artista", "Álbum");
        let uri = track.uri.clone();
        app.queue = vec![track];
        app.status.index = Some(0);
        app.panel.uri = Some(uri.clone());
        app.panel.translate = true;
        app.panel.requested_width = Some(69);
        app.panel.cache.insert(
            uri.clone(),
            Entry {
                cover: Load::Ready(Some(DynamicImage::new_rgb8(20, 20))),
                lyrics: Load::Ready(Some("Hola".into())),
                translation: Load::Loading,
            },
        );
        let mut terminal = Terminal::new(TestBackend::new(160, 45)).unwrap();
        for (state, expected) in [
            (Load::Loading, "Traduciendo…"),
            (Load::Error("Límite diario".into()), "Límite diario"),
            (Load::Ready("Hola".into()), "La letra ya está en español."),
            (Load::Ready("Hello".into()), "Hello"),
        ] {
            app.panel.cache.get_mut(&uri).unwrap().translation = state;
            terminal.draw(|f| crate::ui::draw(f, &mut app)).unwrap();
            // Wrapped state text is checked separately from terminal row boundaries.
            let right = app.areas.panel;
            let rows: String = (0..right.bottom())
                .map(|y| {
                    (right.x..right.right())
                        .map(|x| terminal.backend().buffer()[(x, y)].symbol())
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("\n");
            assert!(rows.contains(expected), "{rows}");
            assert!(app.panel.protocol.is_some());
        }
        app.panel.toggle_pixel();
        assert!(app.panel.pixel && app.panel.protocol.is_none());
        terminal.resize(Rect::new(0, 0, 120, 35)).unwrap();
        terminal.draw(|f| crate::ui::draw(f, &mut app)).unwrap();
        assert!(app.panel.protocol.is_some());
    }

    #[test]
    fn local_workers_decode_and_cache_without_refetch_on_ticks() {
        let runtime = Runtime::new().unwrap();
        let folder = std::env::temp_dir().join(format!("mp-tui-panel-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("track.flac");
        std::fs::write(path.with_extension("lrc"), "[00:01]Letra local").unwrap();
        DynamicImage::new_rgb8(4, 4)
            .save(folder.join("cover.png"))
            .unwrap();
        let track = Track {
            uri: path.to_string_lossy().into_owned(),
            ..Track::default()
        };
        let mut panel = Panel::default();
        panel.tick(Some(&track), Some(&runtime), None);
        assert!(matches!(panel.cache[&track.uri].cover, Load::Loading));
        let start = std::time::Instant::now();
        while panel.loading() && start.elapsed().as_secs() < 5 {
            std::thread::sleep(std::time::Duration::from_millis(5));
            panel.tick(Some(&track), Some(&runtime), None);
        }
        assert!(matches!(
            panel.cache[&track.uri].cover,
            Load::Ready(Some(_))
        ));
        assert_eq!(
            panel.cache[&track.uri].lyrics,
            Load::Ready(Some("Letra local".into()))
        );
        std::fs::remove_dir_all(&folder).unwrap();
        panel.tick(None, Some(&runtime), None);
        panel.tick(Some(&track), Some(&runtime), None);
        assert!(!panel.loading());
        assert!(matches!(
            panel.cache[&track.uri].cover,
            Load::Ready(Some(_))
        ));
        assert_eq!(panel.cache.len(), 1);
    }
}
