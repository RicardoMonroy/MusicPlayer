import Hero from "./Hero.svelte";
import type { Skin } from "..";
import Cassette from "./Cassette.svelte";

import Equalizer from "./Equalizer.svelte";

export const skin: Skin = {
  id: "cassette",
  name: "Casete",
  width: 420,
  height: 300,
  component: Cassette,
  hero: Hero,
  equalizer: { component: Equalizer, height: 210 },
  playlist: {
    bg: "#efe8d8",
    fg: "#2a2622",
    dim: "#7d7366",
    current: "#c4501b",
    hover: "#e2d8c3",
    border: "#26262b",
    font: '"Courier New", ui-monospace, monospace',
  },
  theme: {
    scheme: "dark",
    bg: "#1d1b19",
    sidebar: "#151311",
    panel: "#26231f",
    surface: "#2e2a25",
    surface2: "#3a352e",
    border: "#4a4339",
    fg: "#efe8d8",
    dim: "#a89f8e",
    artBg: "#3a352e",
    accent: "214, 104, 52",
    onAccent: "#fff8ec",
    font: '"Courier New", ui-monospace, monospace',
    radius: "6px",
    radiusLg: "12px",
    radiusPill: "6px",
    raised: "0 2px 0 #00000088",
    sunken: "inset 0 2px 4px #00000088",
  },
  tui: "cassette",
};
