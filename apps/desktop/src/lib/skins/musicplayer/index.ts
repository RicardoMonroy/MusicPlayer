import Hero from "./Hero.svelte";
import { DEFAULT_THEME, type Skin } from "..";
import Player from "./Player.svelte";
import Equalizer from "./Equalizer.svelte";

export const skin: Skin = {
  id: "musicplayer", name: "MusicPlayer", width: 360, height: 150,
  component: Player,
  hero: Hero,
  equalizer: { component: Equalizer, height: 210 },
  playlist: {
  "bg": "#0f0f12",
  "fg": "#e8e8ee",
  "dim": "#9a9aa8",
  "border": "#2a2a33",
  "font": "\"Inter\", system-ui, sans-serif",
  "current": "rgb(120, 120, 160)",
  "hover": "#1f1f26"
},
  // Resolve lazily because the registry imports this skin.
  get theme() { return DEFAULT_THEME; },
  tui: "musicplayer",
};
