import type { Skin, FullTheme } from "..";
import Player from "./Player.svelte";
import Equalizer from "./Equalizer.svelte";
import Hero from "./Hero.svelte";
const theme: FullTheme = {
  scheme: "dark", bg: "#030b06", sidebar: "#030b06", panel: "#030b06",
  surface: "#0c2012", surface2: "#16341e", border: "#366a43",
  fg: "#83ffa1", dim: "#72b886", artBg: "#030b06", accent: "91, 255, 132", onAccent: "#030b06",
  font: '"Courier New", monospace', radius: "0", radiusLg: "0", radiusPill: "0",
  raised: "0 2px 4px #0008", sunken: "inset 0 2px 5px #0008",
};
export const skin: Skin = {
  id: "hacker", name: "Hacker", width: 440, height: 300, component: Player,
  hero: Hero, equalizer: { component: Equalizer, height: 230 }, theme, tui: "hacker",
  playlist: { bg: theme.bg, fg: theme.fg, dim: theme.dim, current: theme.fg,
    hover: theme.surface2, border: theme.border, font: theme.font },
};
