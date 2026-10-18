import Hero from "./Hero.svelte";
import { type Skin } from "..";
import Player from "./Player.svelte";
import Equalizer from "./Equalizer.svelte";

export const skin: Skin = {
  id: "modern", name: "Moderno", width: 340, height: 340,
  component: Player,
  hero: Hero,
  equalizer: { component: Equalizer, height: 210 },
  playlist: {
  "bg": "#0c1020",
  "fg": "#f3f4ff",
  "dim": "#b5bed7",
  "border": "#354262",
  "font": "\"Inter\", system-ui, sans-serif",
  "current": "rgb(174, 151, 255)",
  "hover": "#25304d"
},
  theme: {
  "bg": "#0c1020",
  "sidebar": "#090d1a",
  "panel": "#12182c",
  "surface": "#19213a",
  "surface2": "#25304d",
  "border": "#354262",
  "fg": "#f3f4ff",
  "dim": "#b5bed7",
  "artBg": "#222b48",
  "accent": "174, 151, 255",
  "onAccent": "#141026",
  "font": "\"Inter\", system-ui, sans-serif",
  "radius": "14px",
  "radiusLg": "24px",
  "radiusPill": "999px",
  "scheme": "dark",
  "raised": "0 8px 24px #00000030",
  "sunken": "none"
},
  tui: "modern",
};
