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
import { invoke } from "@tauri-apps/api/core";
import { skin as cassette } from "./cassette";
import { skin as vinyl } from "./vinyl";
import { skin as classic } from "./classic";
import { skin as musicplayer } from "./musicplayer";
import { skin as modern } from "./modern";
import { skin as minimal } from "./minimal";
import { skin as ipod } from "./ipod";
import { skin as win95 } from "./win95";
import { skin as hacker } from "./hacker";
import { skin as analog } from "./analog";

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

/**
 * Look of the full player while this skin is active (the skin is global).
 * Colors are CSS values; `accent` is an "r, g, b" triplet, or null to follow
 * the cover art like the default look.
 */
export type FullTheme = {
  scheme: "dark" | "light";
  bg: string;
  sidebar: string;
  /** Now-playing panel. */
  panel: string;
  /** Inputs, cards. */
  surface: string;
  /** Hover / selected rows. */
  surface2: string;
  border: string;
  fg: string;
  dim: string;
  artBg: string;
  accent: string | null;
  onAccent: string;
  font: string;
  radius: string;
  radiusLg: string;
  radiusPill: string;
  /** box-shadow for raised controls (buttons) and sunken ones (fields, panels). */
  raised: string;
  sunken: string;
};

/** The original MusicPlayer look. */
export const DEFAULT_THEME: FullTheme = {
  scheme: "dark",
  bg: "#0f0f12",
  sidebar: "#0b0b0e",
  panel: "#15151a",
  surface: "#17171d",
  surface2: "#1f1f26",
  border: "#2a2a33",
  fg: "#e8e8ee",
  dim: "#9a9aa8",
  artBg: "#23232b",
  accent: null,
  onAccent: "#111111",
  font: '"Inter", system-ui, sans-serif',
  radius: "8px",
  radiusLg: "14px",
  radiusPill: "999px",
  raised: "none",
  sunken: "none",
};

/** Write a theme as CSS variables on the document root. */
export function applyTheme(t: FullTheme, coverAccent: string) {
  const vars: Record<string, string> = {
    "--bg": t.bg,
    "--sidebar": t.sidebar,
    "--panel": t.panel,
    "--surface": t.surface,
    "--surface-2": t.surface2,
    "--border": t.border,
    "--fg": t.fg,
    "--fg-dim": t.dim,
    "--art-bg": t.artBg,
    "--accent": t.accent ?? coverAccent,
    "--on-accent": t.onAccent,
    "--font": t.font,
    "--radius": t.radius,
    "--radius-lg": t.radiusLg,
    "--radius-pill": t.radiusPill,
    "--raised": t.raised,
    "--sunken": t.sunken,
  };
  const root = document.documentElement;
  for (const [k, v] of Object.entries(vars)) root.style.setProperty(k, v);
  root.style.colorScheme = t.scheme;
}

export type Skin = {
  id: string;
  name: string;
  width: number;
  height: number;
  component: Component;
  playlist: PlaylistTheme;
  /** Equalizer panel drawn in the skin's own style, unfolded under the skin. */
  equalizer: { component: Component; height: number };
  /** Look of the full player when this skin is chosen. */
  theme: FullTheme;
  /**
   * Optional signature piece shown big in the full player's now-playing panel
   * (spinning record, cassette reels, LCD...). No props; it fills a HERO_SIZE
   * square and reads `player` like the mini skin does.
   */
  hero?: Component;
  /** Palette name the TUI uses for this skin (see apps/tui). */
  tui: string;
};

/** Side of the square the full player gives to a skin's `hero`. */
export const HERO_SIZE = 280;

/** Extra window height while the playlist is open. */
export const PLAYLIST_HEIGHT = 260;

export const skins: Skin[] = [musicplayer, modern, minimal, classic, cassette, vinyl, ipod, win95, hacker, analog];

/** Default skin id, also the fallback for unknown ids. */
export const DEFAULT_SKIN = "musicplayer";

export const skinById = (id: string) =>
  skins.find((s) => s.id === id) ?? skins.find((s) => s.id === DEFAULT_SKIN) ?? skins[0];

/** The chosen skin, shared with the TUI through the config dir. */
export async function savedSkin(): Promise<Skin> {
  const legacy = localStorage.getItem("compact-skin");
  const id = await invoke<string | null>("skin");
  if (id == null && legacy) await chooseSkin(legacy);
  return skinById(id ?? legacy ?? DEFAULT_SKIN);
}

/** Choose the global skin; every window hears about it through the `skin` event. */
export const chooseSkin = (id: string) => invoke("set_skin", { id });
