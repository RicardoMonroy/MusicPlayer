import type { Skin } from "..";
import Cassette from "./Cassette.svelte";

export const skin: Skin = {
  id: "cassette",
  name: "Casete",
  width: 420,
  height: 300,
  component: Cassette,
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
