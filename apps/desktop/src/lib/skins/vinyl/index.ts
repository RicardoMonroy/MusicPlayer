import Hero from "./Hero.svelte";
import type { Skin } from "..";
import Vinyl from "./Vinyl.svelte";

import Equalizer from "./Equalizer.svelte";

export const skin: Skin = {
  id: "vinyl",
  name: "Vinilo",
  width: 380,
  height: 440,
  component: Vinyl,
  hero: Hero,
  equalizer: { component: Equalizer, height: 210 },
  playlist: {
    bg: "#3e2a1c",
    fg: "#f3ece4",
    dim: "#c2ad98",
    current: "#f2b366",
    hover: "#56402e",
    border: "#2a1c12",
    font: '"Inter", system-ui, sans-serif',
  },
  theme: {
    scheme: "dark",
    bg: "#1f1610",
    sidebar: "#170f0a",
    panel: "#2a1d14",
    surface: "#33241a",
    surface2: "#432f21",
    border: "#5a4030",
    fg: "#f3ece4",
    dim: "#c2ad98",
    artBg: "#3e2a1c",
    accent: "242, 179, 102",
    onAccent: "#2a1c12",
    font: '"Inter", system-ui, sans-serif',
    radius: "8px",
    radiusLg: "16px",
    radiusPill: "999px",
    raised: "none",
    sunken: "none",
  },
  tui: "vinyl",
};
