//! 10-band graphic equalizer, Winamp's bands and presets.

use serde::{Deserialize, Serialize};

/// Band centre frequencies in Hz.
pub const FREQS: [u32; 10] = [60, 170, 310, 600, 1000, 3000, 6000, 12000, 14000, 16000];
/// Gain range of every slider, in dB.
pub const RANGE: f32 = 12.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Equalizer {
    pub enabled: bool,
    /// dB applied before the bands.
    pub preamp: f32,
    pub gains: [f32; 10],
}

impl Equalizer {
    pub fn clamped(mut self) -> Self {
        self.preamp = self.preamp.clamp(-RANGE, RANGE);
        for g in &mut self.gains {
            *g = g.clamp(-RANGE, RANGE);
        }
        self
    }
}

/// Winamp's classic presets (dB per band, from its stock .eqf set), clamped to ±12.
pub const PRESETS: &[(&str, [f32; 10])] = &[
    ("Plano", [0.0; 10]),
    (
        "Clásica",
        [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -7.2, -7.2, -7.2, -9.6],
    ),
    ("Club", [0.0, 0.0, 8.0, 5.6, 5.6, 5.6, 3.2, 0.0, 0.0, 0.0]),
    (
        "Dance",
        [9.6, 7.2, 2.4, 0.0, 0.0, -5.6, -7.2, -7.2, 0.0, 0.0],
    ),
    (
        "Graves",
        [-8.0, 9.6, 9.6, 5.6, 1.6, -4.0, -8.0, -10.4, -11.2, -11.2],
    ),
    (
        "Graves y agudos",
        [7.2, 5.6, 0.0, -7.2, -4.8, 1.6, 8.0, 11.2, 12.0, 12.0],
    ),
    (
        "Agudos",
        [-9.6, -9.6, -9.6, -4.0, 2.4, 11.2, 12.0, 12.0, 12.0, 12.0],
    ),
    (
        "Audífonos",
        [4.8, 11.2, 5.6, -3.2, -2.4, 1.6, 4.8, 9.6, 12.0, 12.0],
    ),
    (
        "Sala grande",
        [10.4, 10.4, 5.6, 5.6, 0.0, -4.8, -4.8, -4.8, 0.0, 0.0],
    ),
    (
        "En vivo",
        [-4.8, 0.0, 4.0, 5.6, 5.6, 5.6, 4.0, 2.4, 2.4, 2.4],
    ),
    ("Fiesta", [7.2, 7.2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 7.2, 7.2]),
    (
        "Pop",
        [-1.6, 4.8, 7.2, 8.0, 5.6, 0.0, -2.4, -2.4, -1.6, -1.6],
    ),
    (
        "Reggae",
        [0.0, 0.0, 0.0, -5.6, 0.0, 6.4, 6.4, 0.0, 0.0, 0.0],
    ),
    (
        "Rock",
        [8.0, 4.8, -5.6, -8.0, -3.2, 4.0, 8.8, 11.2, 11.2, 11.2],
    ),
    (
        "Ska",
        [-2.4, -4.8, -4.0, 0.0, 4.0, 5.6, 8.8, 9.6, 11.2, 9.6],
    ),
    (
        "Suave",
        [4.8, 1.6, 0.0, -2.4, 0.0, 4.0, 8.0, 9.6, 11.2, 12.0],
    ),
    (
        "Rock suave",
        [4.0, 4.0, 2.4, 0.0, -4.0, -5.6, -3.2, 0.0, 2.4, 8.8],
    ),
    (
        "Techno",
        [8.0, 5.6, 0.0, -5.6, -4.8, 0.0, 8.0, 9.6, 9.6, 8.8],
    ),
    // Metal: gentle curves; heavy guitars are already dense, big boosts only add harshness.
    // Tight lows, less low-mid mud, presence for riffs and cymbals.
    (
        "Heavy Metal",
        [4.0, 3.0, -1.0, -2.0, -1.0, 2.0, 4.0, 3.0, 2.0, 1.0],
    ),
    // The classic mid scoop: punchy kick, crunchy highs.
    (
        "Thrash (scoop)",
        [5.0, 4.0, 0.0, -4.0, -5.0, -1.0, 3.0, 4.0, 3.0, 2.0],
    ),
    // Weight for low tunings while keeping the riffs readable.
    (
        "Death / Doom",
        [6.0, 5.0, 2.0, -2.0, -2.0, 1.0, 2.0, 2.0, 1.0, 0.0],
    ),
    // Thin, trebly productions: add body, tame the fizz.
    (
        "Black Metal",
        [3.0, 4.0, 3.0, 1.0, 0.0, -1.0, -2.0, -3.0, -3.0, -4.0],
    ),
    // Vocals and orchestra forward.
    (
        "Power / Sinfónico",
        [3.0, 2.0, 0.0, -1.0, 1.0, 3.0, 3.0, 2.0, 2.0, 2.0],
    ),
    // Mostly flat with a little extra detail at both ends.
    (
        "Progresivo",
        [2.0, 1.0, 0.0, -1.0, 0.0, 1.0, 2.0, 2.0, 1.0, 1.0],
    ),
];

/// Gain applied before the bands: the user's preamp minus the largest boost,
/// so a preset never pushes the signal past full scale (audible clipping).
pub(crate) fn headroom_preamp(eq: &Equalizer) -> f32 {
    let boost = eq.gains.iter().copied().fold(0.0, f32::max);
    eq.preamp - boost
}

/// mpv `af` entry: one peaking filter per band (octave wide) and a preamp.
/// Instances are named so `af-command` can change gains without rebuilding the chain.
pub(crate) fn filter(eq: &Equalizer) -> String {
    let bands: Vec<String> = FREQS
        .iter()
        .zip(eq.gains)
        .enumerate()
        .map(|(i, (f, g))| format!("equalizer@b{i}=f={f}:t=o:w=1:g={g}"))
        .collect();
    format!(
        "@eq:lavfi=[volume@pre=volume={}dB,{}]",
        headroom_preamp(eq),
        bands.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headroom_cancels_the_biggest_boost() {
        let mut eq = Equalizer {
            enabled: true,
            preamp: 0.0,
            gains: PRESETS.iter().find(|p| p.0 == "Rock").unwrap().1,
        };
        assert_eq!(headroom_preamp(&eq), -11.2);
        eq.preamp = 3.0;
        assert!((headroom_preamp(&eq) - -8.2).abs() < 1e-4);
        eq.gains = [-3.0; 10];
        assert_eq!(headroom_preamp(&eq), 3.0, "cuts need no headroom");
        assert!(
            PRESETS
                .iter()
                .all(|(_, g)| g.iter().all(|v| v.abs() <= RANGE))
        );
        assert!(PRESETS.iter().any(|p| p.0 == "Heavy Metal"));
    }
}
