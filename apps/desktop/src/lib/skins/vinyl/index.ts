import type { Skin } from "..";
import Vinyl from "./Vinyl.svelte";

export const skin: Skin = {
  id: "vinyl",
  name: "Vinilo",
  width: 380,
  height: 440,
  component: Vinyl,
  playlist: {
    bg: "#3e2a1c",
    fg: "#f3ece4",
    dim: "#c2ad98",
    current: "#f2b366",
    hover: "#56402e",
    border: "#2a1c12",
    font: '"Inter", system-ui, sans-serif',
  },
};
