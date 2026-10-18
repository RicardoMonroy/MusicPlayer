import type { Skin } from "..";
import Cassette from "./Cassette.svelte";

import Equalizer from "./Equalizer.svelte";

export const skin: Skin = {
  id: "cassette",
  name: "Casete",
  width: 420,
  height: 300,
  component: Cassette,
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
};
