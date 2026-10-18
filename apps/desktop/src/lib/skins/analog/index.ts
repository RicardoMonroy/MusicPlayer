import type { Skin, FullTheme } from "..";
import Player from "./Player.svelte";
import Equalizer from "./Equalizer.svelte";
import Hero from "./Hero.svelte";
const theme: FullTheme = {
  scheme: "dark", bg: "#21150f", sidebar: "#21150f", panel: "#21150f",
  surface: "#352319", surface2: "#493022", border: "#947042",
  fg: "#ffedc7", dim: "#d0b58e", artBg: "#21150f", accent: "244, 175, 79", onAccent: "#21150f",
  font: 'Georgia, "Times New Roman", serif', radius: "6px", radiusLg: "6px", radiusPill: "6px",
  raised: "0 2px 4px #0008", sunken: "inset 0 2px 5px #0008",
};
export const skin: Skin = {
  id: "analog", name: "Análogo", width: 440, height: 300, component: Player,
  hero: Hero, equalizer: { component: Equalizer, height: 230 }, theme, tui: "analog",
  playlist: { bg: theme.bg, fg: theme.fg, dim: theme.dim, current: theme.fg,
    hover: theme.surface2, border: theme.border, font: theme.font },
};
