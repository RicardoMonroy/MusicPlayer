import Hero from "./Hero.svelte";
import type { Skin } from "../index";
import Ipod from "./Ipod.svelte";
import Equalizer from "./Equalizer.svelte";
export const skin: Skin = {
  id: "ipod", name: "iPod", width: 260, height: 430, component: Ipod,
  hero: Hero,
  equalizer: { component: Equalizer, height: 290 },
  playlist: {"bg": "#c5d2df", "fg": "#202b36", "dim": "#4d5966", "current": "#163e68", "hover": "#dce8f2", "border": "#a5adb6", "font": "\"Charcoal\", \"Geneva\", system-ui, sans-serif"},
  theme: {"scheme": "light", "bg": "#f4f5f6", "sidebar": "#e5e8ec", "panel": "#ffffff", "surface": "#ffffff", "surface2": "#dce8f2", "border": "#a5adb6", "fg": "#202b36", "dim": "#4d5966", "artBg": "#d4dce4", "accent": "45, 96, 143", "onAccent": "#ffffff", "radius": "8px", "radiusLg": "16px", "radiusPill": "999px", "raised": "inset 0 1px 0 #fff, 0 1px 2px #0002", "sunken": "inset 0 1px 3px #0002", "font": "\"Charcoal\", \"Geneva\", system-ui, sans-serif"},
  tui: "ipod",
};
