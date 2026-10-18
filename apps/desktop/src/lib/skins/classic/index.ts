import type { Skin } from "../index";
import Classic from "./Classic.svelte";

export const skin: Skin = {
  id: "classic",
  name: "Clásica",
  width: 550,
  height: 232,
  component: Classic,
  playlist: {
    bg: "#000000",
    fg: "#00e000",
    dim: "#2f8f2f",
    current: "#ffffff",
    hover: "#0000c6",
    border: "#2b2d35",
    font: 'ui-monospace, "DejaVu Sans Mono", monospace',
  },
};
