use crate::app::App;
use mp_core::eq::{Equalizer, PRESETS, RANGE};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Stylize},
    widgets::{Block, Paragraph},
};

pub const LABELS: [&str; 11] = [
    "PREAMP", "60", "170", "310", "600", "1K", "3K", "6K", "12K", "14K", "16K",
];

pub struct EqState {
    pub value: Equalizer,
    pub selected: usize,
    pub preset: usize,
}

pub fn matching_preset(eq: &Equalizer) -> Option<usize> {
    PRESETS.iter().position(|(_, gains)| {
        gains
            .iter()
            .zip(eq.gains)
            .all(|(a, b)| (a - b).abs() < 0.001)
    })
}

impl EqState {
    pub fn new(value: Equalizer) -> Self {
        let preset = matching_preset(&value).unwrap_or(0);
        Self {
            value: value.clamped(),
            selected: 0,
            preset,
        }
    }
    pub fn select(&mut self, delta: isize) {
        self.selected = self.selected.saturating_add_signed(delta).min(10);
    }
    pub fn gain(&self, i: usize) -> f32 {
        if i == 0 {
            self.value.preamp
        } else {
            self.value.gains[i - 1]
        }
    }
    pub fn set_gain(&mut self, gain: f32) {
        let gain = gain.clamp(-RANGE, RANGE);
        if self.selected == 0 {
            self.value.preamp = gain;
        } else {
            self.value.gains[self.selected - 1] = gain;
        }
        self.value.enabled = true;
    }
    pub fn adjust(&mut self, delta: f32) {
        self.set_gain(self.gain(self.selected) + delta);
    }
    pub fn apply_preset(&mut self, i: usize) {
        self.preset = i.min(PRESETS.len() - 1);
        self.value.gains = PRESETS[self.preset].1;
        self.value.preamp = 0.0;
        self.value.enabled = true;
    }
    pub fn cycle_preset(&mut self, delta: isize) {
        let current = matching_preset(&self.value).unwrap_or(self.preset);
        self.apply_preset((current as isize + delta).rem_euclid(PRESETS.len() as isize) as usize);
    }
}

/// Top and bottom rows are +12 and -12 dB; snap to half a decibel.
pub fn gain_at(rect: Rect, y: u16) -> f32 {
    if rect.height <= 1 {
        return 0.0;
    }
    let offset = y.saturating_sub(rect.y).min(rect.height - 1);
    ((RANGE - 2.0 * RANGE * offset as f32 / (rect.height - 1) as f32) * 2.0).round() / 2.0
}

pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    let p = app.palette;
    let name = matching_preset(&app.eq.value).map_or("Personalizado", |i| PRESETS[i].0);
    let block = Block::bordered()
        .title(format!(
            " Ecualizador · {} · {name} ",
            if app.eq.value.enabled { "ON" } else { "OFF" }
        ))
        .border_style(Style::new().fg(p.accent));
    let inner = block.inner(area);
    f.render_widget(block, area);
    if inner.width < 20 || inner.height < 6 {
        return;
    }
    let chart_height = inner.height.saturating_sub(4).max(2);
    let chart = Rect::new(
        inner.x + 4,
        inner.y + 1,
        inner.width.saturating_sub(4),
        chart_height,
    );
    for (offset, label) in [
        (0, "+12"),
        ((chart.height - 1) / 2, "  0"),
        (chart.height - 1, "-12"),
    ] {
        f.render_widget(
            Paragraph::new(label).fg(p.dim),
            Rect::new(inner.x, chart.y + offset, 4, 1),
        );
    }
    app.areas.eq_bars.clear();
    for (i, label) in LABELS.iter().enumerate() {
        let x = chart.x + (i as u32 * chart.width as u32 / 11) as u16;
        let end = chart.x + ((i + 1) as u32 * chart.width as u32 / 11) as u16;
        let r = Rect::new(x, chart.y, end - x, chart.height);
        app.areas.eq_bars.push(r);
        let gain = app.eq.gain(i);
        let color = if gain < 0.0 { p.cool } else { p.warm };
        let marker = (((RANGE - gain) / (2.0 * RANGE)) * (r.height - 1) as f32).round() as u16;
        let zero = (r.height - 1) / 2;
        for row in 0..r.height {
            let symbol = if row == marker {
                "██"
            } else if row >= marker.min(zero) && row <= marker.max(zero) {
                "┃ "
            } else if row == zero {
                "──"
            } else {
                "│ "
            };
            f.render_widget(
                Paragraph::new(symbol).fg(if row == zero && row != marker {
                    p.dim
                } else {
                    color
                }),
                Rect::new(x, r.y + row, r.width.min(2), 1),
            );
        }
        let label = if i == 0 && r.width < 6 { "PRE" } else { label };
        f.render_widget(
            Paragraph::new(label.to_string()).style(if i == app.eq.selected {
                Style::new().fg(p.ink).bg(p.accent).bold()
            } else {
                Style::new().fg(p.text)
            }),
            Rect::new(x, inner.y, r.width, 1),
        );
        f.render_widget(
            Paragraph::new(format!("{gain:+.1}")).fg(color),
            Rect::new(x, chart.bottom(), r.width, 1),
        );
    }
    // Full preamp name also remains visible on compact terminals.
    let selected = app.table.selected().unwrap_or(0).min(PRESETS.len() - 1);
    f.render_widget(
        Paragraph::new(format!(
            "PREAMP · Preajustes (j/k, Enter) · {}/{}",
            selected + 1,
            PRESETS.len()
        ))
        .fg(p.dim),
        Rect::new(inner.x, chart.bottom() + 1, inner.width, 1),
    );
    app.areas.eq_presets.clear();
    let slots = (inner.width / 20).max(1) as usize;
    let start = selected / slots * slots;
    for (col, (i, (name, _))) in PRESETS
        .iter()
        .enumerate()
        .skip(start)
        .take(slots)
        .enumerate()
    {
        let rect = Rect::new(
            inner.x + col as u16 * (inner.width / slots as u16),
            chart.bottom() + 2,
            inner.width / slots as u16,
            1,
        );
        app.areas.eq_presets.push((i, rect));
        f.render_widget(
            Paragraph::new(format!("{} {name}", if i == selected { "›" } else { " " })).style(
                if i == selected {
                    Style::new().fg(p.accent).bg(p.selected)
                } else {
                    Style::new().fg(p.text)
                },
            ),
            rect,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn presets_selection_and_clamping() {
        let mut eq = EqState::new(Equalizer::default());
        assert_eq!(matching_preset(&eq.value), Some(0));
        for i in 0..PRESETS.len() {
            eq.apply_preset(i);
            assert_eq!(matching_preset(&eq.value), Some(i));
        }
        eq.value.gains[0] = 11.9;
        assert_eq!(matching_preset(&eq.value), None);
        eq.select(-100);
        assert_eq!(eq.selected, 0);
        eq.adjust(100.0);
        assert_eq!(eq.value.preamp, 12.0);
        eq.select(100);
        assert_eq!(eq.selected, 10);
        eq.adjust(-100.0);
        assert_eq!(eq.value.gains[9], -12.0);
        assert!(eq.value.enabled);
    }
    #[test]
    fn mouse_mapping() {
        let r = Rect::new(4, 5, 3, 25);
        assert_eq!(gain_at(r, 0), 12.0);
        assert_eq!(gain_at(r, 17), 0.0);
        assert_eq!(gain_at(r, 100), -12.0);
        assert_eq!(gain_at(Rect::default(), 0), 0.0);
    }
}
