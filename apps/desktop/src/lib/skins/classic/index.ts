import Hero from "./Hero.svelte";
import type { Skin } from "../index";
import Classic from "./Classic.svelte";

import Equalizer from "./Equalizer.svelte";

export const skin: Skin = {
  id: "classic",
  name: "Clásica",
  width: 550,
  height: 232,
  component: Classic,
  hero: Hero,
  equalizer: { component: Equalizer, height: 210 },
  playlist: {
    bg: "#000000",
    fg: "#00e000",
    dim: "#2f8f2f",
    current: "#ffffff",
    hover: "#0000c6",
    border: "#2b2d35",
    font: 'ui-monospace, "DejaVu Sans Mono", monospace',
  },
  theme: {
    scheme: "dark",
    bg: "#22232a",
    sidebar: "#1a1b20",
    panel: "#2b2d35",
    surface: "#14161a",
    surface2: "#33363f",
    border: "#4b4e58",
    fg: "#d7d9df",
    dim: "#8e919a",
    artBg: "#000000",
    accent: "0, 224, 0",
    onAccent: "#001a00",
    font: 'ui-monospace, "DejaVu Sans Mono", monospace',
    radius: "2px",
    radiusLg: "3px",
    radiusPill: "3px",
    raised: "inset 1px 1px 0 #6b6e78, inset -1px -1px 0 #121317",
    sunken: "inset 1px 1px 0 #0b0c0f, inset -1px -1px 0 #555863",
  },
  tui: "classic",
};
