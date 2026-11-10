use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph},
};

pub const HELP: &[(&str, &str, &str)] = &[
    (
        "General",
        "1–9 / 0",
        "Secciones; 0 abre el ecualizador (allí pone la banda a cero)",
    ),
    (
        "General",
        "Tab / ⇧Tab",
        "Columna siguiente/anterior en resultados, artista local e inicio YouTube; en otras vistas cambia sección",
    ),
    (
        "General",
        "? / F1",
        "Abrir ayuda; al escribir, ? es texto y F1 abre ayuda",
    ),
    (
        "General",
        "q / Ctrl+c",
        "Salir (al escribir se introducen caracteres)",
    ),
    ("General", "t", "Cambiar paleta (minúscula)"),
    (
        "General",
        "Esc / ? / F1 / q",
        "Cerrar ayuda; ↑/↓, PgUp/PgDn y rueda la desplazan",
    ),
    ("Reproducción", "Espacio", "Pausar / continuar"),
    (
        "Reproducción",
        "y",
        "Copiar URL del álbum de YouTube Music (fila de la cola o canción actual)",
    ),
    (
        "Navegación y listas",
        "a / d",
        "Ir al artista / disco de YouTube Music (fila de la cola o canción actual); Esc vuelve",
    ),
    (
        "Reproducción",
        "n / p",
        "Siguiente / anterior; p tiene prioridad de colección local o remota abierta y de preajuste en ecualizador",
    ),
    (
        "Reproducción",
        "← / →",
        "Retroceder / avanzar 5 s (en ecualizador seleccionan banda sin foco en panel)",
    ),
    (
        "Reproducción",
        "+ / = / -",
        "Subir / subir / bajar volumen 5 puntos",
    ),
    (
        "Navegación y listas",
        "↑/↓ / k/j",
        "Fila anterior/siguiente; ↑/↓ desplazan el panel con foco o ajustan el ecualizador",
    ),
    ("Navegación y listas", "PgUp / PgDn", "Mover 10 filas"),
    (
        "Navegación y listas",
        "Inicio/g / Fin/G",
        "Primera / última fila",
    ),
    (
        "Navegación y listas",
        "Enter",
        "Abrir artista, álbum, carpeta o colección; reproducir canción con su lista; aplicar preajuste en ecualizador",
    ),
    (
        "Navegación y listas",
        "Esc / Backspace",
        "Volver; sin historial restablecer selección/filtro (Esc quita primero el foco del panel)",
    ),
    (
        "Navegación y listas",
        "/",
        "Buscar local desde secciones locales; buscar en YouTube desde las demás",
    ),
    (
        "Navegación y listas",
        "o",
        "Cambiar orden en cualquier vista local",
    ),
    (
        "Navegación y listas",
        "p / s",
        "Álbum local: reproducir completo / mezclar; carpeta local: p reproduce todo",
    ),
    (
        "Navegación y listas",
        "p",
        "Colección abierta en Inicio, Buscar YouTube o Mi biblioteca: reproducir completa sin foco en panel",
    ),
    (
        "Navegación y listas",
        "r",
        "Inicio: recargar si no está cargando; Buscar YouTube y Mi biblioteca: borrar búsqueda y recargar biblioteca/cuenta",
    ),
    (
        "Búsqueda",
        "Texto / Backspace",
        "Con buscador enfocado: escribir / borrar último carácter (también ?, q, t y T son texto)",
    ),
    (
        "Búsqueda",
        "Enter",
        "Buscar en YouTube o ejecutar búsqueda local; salir del buscador",
    ),
    ("Búsqueda", "Esc", "Salir del buscador conservando consulta"),
    (
        "Búsqueda",
        "Tab / ⇧Tab",
        "Sin escribir: cambiar columna en resultados locales o YouTube",
    ),
    (
        "Panel Reproduciendo",
        "i",
        "Mostrar / ocultar panel (necesita al menos 100 columnas)",
    ),
    (
        "Panel Reproduciendo",
        "[ / ]",
        "Reducir / ampliar ancho solicitado 4 columnas si el panel ocupa espacio",
    ),
    (
        "Panel Reproduciendo",
        "c",
        "Alternar portada en píxeles (sin Ctrl)",
    ),
    (
        "Panel Reproduciendo",
        "l",
        "Mostrar / ocultar letra; al mostrarla enfoca el panel visible",
    ),
    (
        "Panel Reproduciendo",
        "T",
        "Activar / desactivar traducción al español (mayúscula)",
    ),
    (
        "Panel Reproduciendo",
        "Clic / ↑ / ↓",
        "Clic en panel: enfocar; con foco, flechas desplazan una línea",
    ),
    (
        "Panel Reproduciendo",
        "Esc / Tab",
        "Quitar foco del panel; rueda sobre panel desplaza 3 líneas",
    ),
    (
        "Ecualizador",
        "← / →",
        "Seleccionar banda o preamplificador (sin foco en panel)",
    ),
    (
        "Ecualizador",
        "↑ / ↓",
        "Ajustar ±0.5 dB; con ⇧, ±2 dB (sin foco en panel)",
    ),
    (
        "Ecualizador",
        "0 / o",
        "Banda a 0 dB / activar o desactivar (sin foco en panel)",
    ),
    (
        "Ecualizador",
        "p / P",
        "Preajuste siguiente / anterior (sin foco en panel)",
    ),
    (
        "Ecualizador",
        "j/k / Enter",
        "Seleccionar preajuste en lista / aplicar seleccionado",
    ),
    (
        "Ratón",
        "Clic",
        "Seleccionar fila, sección, columna, buscador o resultado principal; botones controlan reproducción",
    ),
    (
        "Ratón",
        "Doble clic",
        "Abrir / reproducir fila seleccionada (dos clics en menos de 400 ms)",
    ),
    (
        "Ratón",
        "Rueda",
        "Mover 3 filas; sobre columna cambia foco; sobre panel desplaza letra",
    ),
    (
        "Ratón",
        "Clic en barras",
        "Progreso: buscar posición; volumen: ajustar nivel",
    ),
    (
        "Ratón",
        "Clic / arrastrar",
        "En banda del ecualizador: seleccionar y ajustar ganancia; rueda ±0.5 dB; clic en preajuste solo selecciona",
    ),
];

#[derive(Default)]
pub struct Help {
    pub open: bool,
    pub scroll: usize,
    pub max_scroll: usize,
    pub page: usize,
}

impl Help {
    pub fn move_by(&mut self, delta: isize) {
        self.scroll = self
            .scroll
            .saturating_add_signed(delta)
            .min(self.max_scroll);
    }
}

fn lines(width: u16, p: crate::palette::Palette) -> Vec<Line<'static>> {
    let keys_width = width.saturating_sub(4).min(22);
    let action_width = width.saturating_sub(keys_width + 3).max(1);
    let mut lines = Vec::new();
    let mut previous = "";
    for &(group, keys, action) in HELP {
        if previous != group {
            if !previous.is_empty() {
                lines.push(Line::default());
            }
            lines.push(Line::from(group).fg(p.accent).bold());
            previous = group;
        }
        let keys = crate::ui::lyrics_lines(keys, keys_width.max(1));
        let actions = crate::ui::lyrics_lines(action, action_width);
        for i in 0..keys.len().max(actions.len()) {
            let key = keys.get(i).cloned().unwrap_or_default();
            let padding = (keys_width as usize).saturating_sub(Span::raw(&key).width());
            lines.push(Line::from(vec![
                Span::raw(format!("{key}{}", " ".repeat(padding))).fg(p.accent),
                Span::raw(" │ ").fg(p.dim),
                Span::raw(actions.get(i).cloned().unwrap_or_default()).fg(p.text),
            ]));
        }
    }
    lines
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let screen = f.area();
    let width = (u32::from(screen.width) * 80 / 100) as u16;
    let height = (u32::from(screen.height) * 80 / 100) as u16;
    let area = Rect::new(
        screen.x + (screen.width - width) / 2,
        screen.y + (screen.height - height) / 2,
        width,
        height,
    );
    let p = app.palette;
    let block = Block::bordered()
        .title(" Atajos de teclado ")
        .border_style(Style::new().fg(p.accent))
        .style(Style::new().fg(p.text).bg(p.background));
    let inner = block.inner(area);
    let lines = lines(inner.width, p);
    app.help.page = usize::from(inner.height).max(1);
    app.help.max_scroll = lines.len().saturating_sub(inner.height as usize);
    app.help.scroll = app.help.scroll.min(app.help.max_scroll);
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    f.render_widget(
        Paragraph::new(lines).scroll((app.help.scroll as u16, 0)),
        inner,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn overlay_renders_all_groups_by_scrolling_at_supported_sizes() {
        for (width, height) in [(80, 24), (160, 45)] {
            let mut app = crate::app::tests::app(Vec::new());
            app.help.open = true;
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            let mut screens = String::new();
            loop {
                terminal.draw(|f| crate::ui::draw(f, &mut app)).unwrap();
                let buffer = terminal.backend().buffer();
                let screen = (0..height)
                    .map(|y| {
                        (0..width)
                            .map(|x| buffer[(x, y)].symbol())
                            .collect::<String>()
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                assert!(screen.contains(" Atajos de teclado "));
                assert!(
                    screen
                        .lines()
                        .last()
                        .unwrap()
                        .trim_start()
                        .starts_with("? ayuda")
                );
                screens.push_str(&screen);
                if app.help.scroll == app.help.max_scroll {
                    break;
                }
                app.help.move_by(1);
            }
            for group in HELP.iter().map(|binding| binding.0) {
                assert!(
                    screens.contains(group),
                    "missing {group} at {width}x{height}"
                );
            }
            assert!(screens.contains("T"));
            assert!(screens.contains("traducir") || screens.contains("traducción"));
        }
    }

    #[test]
    fn footer_help_is_first_in_every_context() {
        for tab in crate::app::TABS {
            for searching in [false, true] {
                let mut app = crate::app::tests::app(Vec::new());
                app.set_tab(tab);
                app.searching = searching;
                let mut terminal = Terminal::new(TestBackend::new(160, 45)).unwrap();
                terminal.draw(|f| crate::ui::draw(f, &mut app)).unwrap();
                let buffer = terminal.backend().buffer();
                let y = if tab == crate::app::Tab::Equalizer {
                    43
                } else {
                    44
                };
                let footer = (0..160)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>();
                assert!(footer.trim_start().starts_with("? ayuda"), "{footer}");
                assert!(footer.contains("T traducir"), "{footer}");
            }
        }
    }
}
