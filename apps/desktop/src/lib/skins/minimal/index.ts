import Hero from "./Hero.svelte";
import { type Skin } from "..";
import Player from "./Player.svelte";
import Equalizer from "./Equalizer.svelte";

export const skin: Skin = {
  id: "minimal", name: "Minimalista", width: 320, height: 120,
  component: Player,
  hero: Hero,
  equalizer: { component: Equalizer, height: 210 },
  playlist: {
  "bg": "#f7f6f2",
  "fg": "#202421",
  "dim": "#595e59",
  "border": "#cbc9c0",
  "font": "\"Helvetica Neue\", \"Inter\", system-ui, sans-serif",
  "current": "rgb(71, 99, 81)",
  "hover": "#e7e6e0"
},
  theme: {
  "bg": "#f7f6f2",
  "sidebar": "#efeee9",
  "panel": "#f2f1ec",
  "surface": "#ffffff",
  "surface2": "#e7e6e0",
  "border": "#cbc9c0",
  "fg": "#202421",
  "dim": "#595e59",
  "artBg": "#e3e4dc",
  "accent": "71, 99, 81",
  "onAccent": "#ffffff",
  "font": "\"Helvetica Neue\", \"Inter\", system-ui, sans-serif",
  "radius": "2px",
  "radiusLg": "4px",
  "radiusPill": "3px",
  "scheme": "light",
  "raised": "none",
  "sunken": "none"
},
  tui: "minimal",
};
