use ratatui::style::Color;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    pub id: &'static str,
    pub accent: Color,
    pub dim: Color,
    pub selected: Color,
    pub text: Color,
    pub background: Color,
    pub zebra: Color,
    pub cool: Color,
    pub warm: Color,
    pub hot: Color,
    pub ink: Color,
}

pub const IDS: [&str; 10] = [
    "musicplayer",
    "classic",
    "cassette",
    "vinyl",
    "ipod",
    "win95",
    "modern",
    "minimal",
    "hacker",
    "analog",
];

pub fn lookup(id: &str) -> Palette {
    let (id, accent, bg, cool, warm, hot) = match id {
        "classic" => (
            "classic",
            (100, 240, 95),
            (8, 16, 10),
            (100, 230, 80),
            (245, 210, 65),
            (245, 90, 45),
        ),
        "cassette" => (
            "cassette",
            (245, 180, 100),
            (36, 28, 23),
            (125, 195, 190),
            (245, 165, 75),
            (245, 110, 70),
        ),
        "vinyl" => (
            "vinyl",
            (225, 175, 90),
            (30, 23, 18),
            (140, 180, 170),
            (225, 155, 65),
            (225, 105, 65),
        ),
        "ipod" => (
            "ipod",
            (160, 195, 225),
            (28, 36, 46),
            (120, 190, 230),
            (225, 185, 125),
            (230, 130, 130),
        ),
        "win95" => (
            "win95",
            (125, 190, 210),
            (15, 30, 55),
            (80, 195, 185),
            (220, 205, 130),
            (235, 125, 110),
        ),
        "modern" => (
            "modern",
            (195, 145, 255),
            (24, 20, 40),
            (85, 220, 235),
            (225, 145, 245),
            (255, 115, 160),
        ),
        "minimal" => (
            "minimal",
            (175, 190, 195),
            (25, 27, 29),
            (160, 175, 185),
            (205, 205, 195),
            (235, 235, 225),
        ),
        "hacker" => (
            "hacker",
            (80, 255, 120),
            (3, 12, 6),
            (65, 210, 100),
            (165, 235, 90),
            (245, 180, 80),
        ),
        "analog" => (
            "analog",
            (255, 180, 75),
            (32, 21, 14),
            (180, 195, 135),
            (245, 155, 65),
            (255, 110, 60),
        ),
        _ => (
            "musicplayer",
            (255, 110, 150),
            (23, 23, 29),
            (120, 200, 255),
            (255, 150, 175),
            (255, 120, 120),
        ),
    };
    let rgb = |(r, g, b)| Color::Rgb(r, g, b);
    let (dim, text) = match id {
        "hacker" => ((100, 170, 115), (180, 240, 190)),
        "analog" => ((185, 155, 120), (245, 230, 200)),
        _ => ((160, 165, 175), (230, 230, 232)),
    };
    Palette {
        id,
        accent: rgb(accent),
        dim: rgb(dim),
        selected: rgb((bg.0 + 24, bg.1 + 24, bg.2 + 24)),
        text: rgb(text),
        background: rgb(bg),
        zebra: rgb((bg.0 + 9, bg.1 + 9, bg.2 + 9)),
        cool: rgb(cool),
        warm: rgb(warm),
        hot: rgb(hot),
        ink: Color::Rgb(12, 14, 18),
    }
}

impl Palette {
    pub fn next(self) -> Self {
        let i = IDS.iter().position(|id| *id == self.id).unwrap_or(0);
        lookup(IDS[(i + 1) % IDS.len()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cycle_order_and_wrap() {
        let expected = [
            "musicplayer",
            "classic",
            "cassette",
            "vinyl",
            "ipod",
            "win95",
            "modern",
            "minimal",
            "hacker",
            "analog",
        ];
        assert_eq!(IDS, expected);
        let mut palette = lookup("musicplayer");
        for id in expected.into_iter().skip(1).chain(["musicplayer"]) {
            palette = palette.next();
            assert_eq!(palette.id, id);
        }
    }

    #[test]
    fn new_skin_colors() {
        let hacker = lookup("hacker");
        assert_eq!(hacker.accent, Color::Rgb(80, 255, 120));
        assert_eq!(hacker.dim, Color::Rgb(100, 170, 115));
        assert_eq!(hacker.background, Color::Rgb(3, 12, 6));
        let analog = lookup("analog");
        assert_eq!(analog.accent, Color::Rgb(255, 180, 75));
        assert_eq!(analog.text, Color::Rgb(245, 230, 200));
        assert_eq!(analog.background, Color::Rgb(32, 21, 14));
    }

    #[test]
    fn lookup_and_fallback() {
        assert_eq!(lookup("unknown"), lookup("musicplayer"));
        for id in IDS {
            assert_eq!(lookup(id).id, id);
        }
    }
}
