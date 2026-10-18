import Hero from "./Hero.svelte";
import type { Skin } from "../index";
import Win95 from "./Win95.svelte";
import Equalizer from "./Equalizer.svelte";
export const skin: Skin = {
  id: "win95", name: "Windows 95", width: 420, height: 200, component: Win95,
  hero: Hero,
  equalizer: { component: Equalizer, height: 240 },
  playlist: {"bg": "#c0c0c0", "fg": "#000000", "dim": "#373737", "current": "#000080", "hover": "#d5d5d5", "border": "#808080", "font": "\"Tahoma\", \"Segoe UI\", sans-serif"},
  theme: {"scheme": "light", "bg": "#c0c0c0", "sidebar": "#c0c0c0", "panel": "#c0c0c0", "surface": "#ffffff", "surface2": "#d5d5d5", "border": "#808080", "fg": "#000000", "dim": "#373737", "artBg": "#a0a0a0", "accent": "0, 0, 128", "onAccent": "#ffffff", "radius": "0", "radiusLg": "0", "radiusPill": "0", "raised": "inset 1px 1px 0 #fff, inset -1px -1px 0 #404040, inset 2px 2px 0 #dfdfdf, inset -2px -2px 0 #808080", "sunken": "inset 1px 1px 0 #808080, inset -1px -1px 0 #fff, inset 2px 2px 0 #404040", "font": "\"Tahoma\", \"Segoe UI\", sans-serif"},
  tui: "win95",
};
