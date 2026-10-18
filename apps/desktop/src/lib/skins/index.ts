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
 * in the top-right corner, so keep that 80x24 px area free of controls.
 */
import type { Component } from "svelte";
import { skin as cassette } from "./cassette";
import { skin as vinyl } from "./vinyl";
import { skin as classic } from "./classic";

export type Skin = {
  id: string;
  name: string;
  width: number;
  height: number;
  component: Component;
};

export const skins: Skin[] = [cassette, vinyl, classic];

const KEY = "compact-skin";

export function savedSkin(): Skin {
  return skins.find((s) => s.id === localStorage.getItem(KEY)) ?? skins[0];
}

export function nextSkin(current: Skin): Skin {
  const s = skins[(skins.indexOf(current) + 1) % skins.length];
  localStorage.setItem(KEY, s.id);
  return s;
}
