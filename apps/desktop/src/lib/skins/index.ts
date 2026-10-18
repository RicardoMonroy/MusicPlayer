/*
 * Compact-view skins.
 *
 * Contract: a skin is a folder `src/lib/skins/<id>/` whose `index.ts` exports
 * `skin: Skin`. The component takes no props. It reads state from `player`
 * ($lib/player.svelte: status, track, coverUrl, accent) and calls the actions
 * exported there (togglePause, next, prev, seek, setVolume, showView).
 * The component fills the whole window (width x height, transparent outside
 * its own shape) and marks drag areas with `data-tauri-drag-region`.
 * The compact page draws the shared chrome (skin switcher, expand, hide) on hover
 * in the top-right corner, so keep that 132x24 px area free of controls.
 * `playlist` themes the queue panel the compact page can unfold below the skin.
 * `equalizer.component` is the skin's EQ panel (no props): it reads `player.eq` /
 * `player.eqPresets` and calls `setEqualizer` / `applyPreset` ($lib/player.svelte).
 * Panels can be toggled from inside a skin through `compact` ($lib/compact.svelte).
 */
import type { Component } from "svelte";
import { emit } from "@tauri-apps/api/event";
import { skin as cassette } from "./cassette";
import { skin as vinyl } from "./vinyl";
import { skin as classic } from "./classic";

/** Colors and font for the shared playlist panel shown under the skin. */
export type PlaylistTheme = {
  bg: string;
  fg: string;
  dim: string;
  /** Current track text color. */
  current: string;
  /** Hover/selection background. */
  hover: string;
  border: string;
  font: string;
};

export type Skin = {
  id: string;
  name: string;
  width: number;
  height: number;
  component: Component;
  playlist: PlaylistTheme;
  /** Equalizer panel drawn in the skin's own style, unfolded under the skin. */
  equalizer: { component: Component; height: number };
};

/** Extra window height while the playlist is open. */
export const PLAYLIST_HEIGHT = 260;

export const skins: Skin[] = [cassette, vinyl, classic];

const KEY = "compact-skin";

export function savedSkin(): Skin {
  return skins.find((s) => s.id === localStorage.getItem(KEY)) ?? skins[0];
}

export const skinById = (id: string) => skins.find((s) => s.id === id) ?? skins[0];

/** Choose the mini player skin; every window hears about it through the `skin` event. */
export function chooseSkin(id: string) {
  localStorage.setItem(KEY, id);
  emit("skin", id);
}
