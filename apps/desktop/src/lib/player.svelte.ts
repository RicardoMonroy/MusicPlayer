import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type Track = {
  title: string;
  artist: string;
  album: string;
  uri: string;
  track_no: number | null;
  duration: number | null;
  cover: string | null;
  album_artist?: string | null;
  year?: number | null;
  disc_no?: number | null;
  genre?: string | null;
  /** YouTube Music album (OLAK5uy or MPREb id); null for local files. */
  album_id?: string | null;
};

export type Status = {
  index: number | null;
  paused: boolean;
  position: number;
  duration: number;
  volume: number;
  shuffle: boolean;
  repeat: "off" | "all" | "one";
};

export type Equalizer = { enabled: boolean; preamp: number; gains: number[] };
/** Band centre frequencies (Hz), as in Winamp. */
export const EQ_FREQS = [60, 170, 310, 600, 1000, 3000, 6000, 12000, 14000, 16000];
export const EQ_RANGE = 12;

export type Now = { status: Status; track: Track | null };

export const player = $state({
  tracks: [] as Track[],
  track: null as Track | null,
  status: {
    index: null,
    paused: false,
    position: 0,
    duration: 0,
    volume: 100,
    shuffle: false,
    repeat: "off",
  } as Status,
  coverUrl: "",
  accent: "120, 120, 160",
  /** Short-lived warning, e.g. a track mpv could not load and skipped. */
  notice: "",
  /** 19 band levels (0..1, low to high), pushed ~25×/s while the compact window is visible. */
  spectrum: [] as number[],
  /** What is playing, in order; `status.index` points into it. */
  queue: [] as Track[],
  eq: { enabled: false, preamp: 0, gains: Array(10).fill(0) } as Equalizer,
  eqPresets: [] as [string, number[]][],
  /** Library scan progress; null when no scan is running. */
  scan: null as { done: number; total: number } | null,
});

let noticeTimer: ReturnType<typeof setTimeout> | undefined;

/** Where the folder lived before it was shared with the TUI. */
const LEGACY_FOLDER_KEY = "library-folder";
let coverFor = "";

/** Scan `folder` and remember it (shared with the TUI). Progress lands in `player.scan`. */
export async function scan(folder: string) {
  player.scan = { done: 0, total: 0 };
  const off = await listen<[number, number]>("scan-progress", (e) => {
    player.scan = { done: e.payload[0], total: e.payload[1] };
  });
  try {
    player.tracks = await invoke<Track[]>("scan_folder", { path: folder });
  } finally {
    off();
    player.scan = null;
  }
}

/**
 * Bytes from a command that returns a raw `Response`. Dev builds get an
 * ArrayBuffer, release builds a plain number array.
 */
const bytesOf = (b: ArrayBuffer | number[]) => (b instanceof ArrayBuffer ? b : new Uint8Array(b).buffer);

const thumbs = new Map<string, Promise<string>>();
/** Object URL of a small cached cover for the album `uri` belongs to ("" if none). */
export function thumbnail(uri: string): Promise<string> {
  let p = thumbs.get(uri);
  if (!p) {
    p = invoke<ArrayBuffer | number[]>("thumbnail", { uri }).then(bytesOf).then((b) =>
      b.byteLength ? URL.createObjectURL(new Blob([b], { type: "image/jpeg" })) : "",
    );
    thumbs.set(uri, p);
  }
  return p;
}

export async function savedFolder(): Promise<string | null> {
  return (await invoke<string | null>("music_folder")) ?? localStorage.getItem(LEGACY_FOLDER_KEY);
}

/** Replace the queue with `tracks` and start at `index`. */
export const play = (tracks: Track[], index: number) =>
  invoke("play", { tracks: $state.snapshot(tracks), index });
export const togglePause = () => invoke("toggle_pause");
export const next = () => invoke("next");
export const prev = () => invoke("prev");
export const seek = (seconds: number) => invoke("seek", { seconds });
export const setVolume = (volume: number) => invoke("set_volume", { volume });

export const jump = (index: number) => invoke("jump", { index });
export const stop = () => invoke("stop");
export const setShuffle = (on: boolean) => invoke("set_shuffle", { on });
export const setRepeat = (mode: Status["repeat"]) => invoke("set_repeat", { mode });
/** Cycle off → all → one, like Winamp's repeat button. */
export const cycleRepeat = () =>
  setRepeat(({ off: "all", all: "one", one: "off" } as const)[player.status.repeat]);

let eqFrame = 0;
/** Update the equalizer now in the UI; send to mpv at most once per frame (sliders fire fast). */
export function setEqualizer(eq: Equalizer) {
  player.eq = eq;
  cancelAnimationFrame(eqFrame);
  eqFrame = requestAnimationFrame(() => invoke("set_equalizer", { eq: $state.snapshot(player.eq) }));
}

export function applyPreset(name: string) {
  const p = player.eqPresets.find(([n]) => n === name);
  if (p) setEqualizer({ ...player.eq, enabled: true, gains: [...p[1]] });
}
export const showView = (label: "main" | "compact") => invoke("show_view", { label });

async function loadCover(track: Track | null) {
  if (player.coverUrl) URL.revokeObjectURL(player.coverUrl);
  player.coverUrl = "";
  if (!track) return;
  const bytes = bytesOf(await invoke<ArrayBuffer | number[]>("cover", { track }));
  if (bytes.byteLength === 0) return;
  player.coverUrl = URL.createObjectURL(new Blob([bytes]));
  player.accent = await averageColor(player.coverUrl);
}

/** Average cover color, nudged brighter so it works as an accent on dark UI. */
async function averageColor(url: string): Promise<string> {
  const img = new Image();
  img.src = url;
  await img.decode();
  const c = document.createElement("canvas");
  c.width = c.height = 1;
  const ctx = c.getContext("2d")!;
  ctx.drawImage(img, 0, 0, 1, 1);
  const [r, g, b] = ctx.getImageData(0, 0, 1, 1).data;
  const lift = Math.max(0, 140 - Math.max(r, g, b));
  return [r, g, b].map((v) => Math.min(255, v + lift)).join(", ");
}

function apply(now: Now) {
  player.status = now.status;
  // Status arrives ~4x/s; keep the same track object unless the song changed, so
  // anything keyed on the track (lyrics, cover, heroes) does not restart.
  if (now.track?.uri !== player.track?.uri) player.track = now.track;
  if ((now.track?.uri ?? "") !== coverFor) {
    coverFor = now.track?.uri ?? "";
    loadCover(now.track).catch(() => {});
  }
}

/** Follow player state pushed from Rust. Returns the unsubscribe function. */
export function connect() {
  invoke<Now>("now").then(apply);
  invoke<Track[]>("queue").then((q) => (player.queue = q));
  invoke<Equalizer | null>("equalizer").then((e) => e && (player.eq = e));
  invoke<[string, number[]][]>("eq_presets").then((p) => (player.eqPresets = p));
  const offEq = listen<Equalizer>("equalizer", (e) => (player.eq = e.payload));
  const offQueue = listen<Track[]>("queue", (e) => (player.queue = e.payload));
  const off = listen<Now>("now", (e) => apply(e.payload));
  const offFailed = listen<Track>("track-failed", (e) => {
    player.notice = `No se pudo reproducir “${e.payload.title}”; se saltó. ¿Requiere iniciar sesión?`;
    clearTimeout(noticeTimer);
    noticeTimer = setTimeout(() => (player.notice = ""), 7000);
  });
  const offSpectrum = listen<number[]>("spectrum", (e) => (player.spectrum = e.payload));
  return () => {
    off.then((f) => f());
    offFailed.then((f) => f());
    offSpectrum.then((f) => f());
    offQueue.then((f) => f());
    offEq.then((f) => f());
  };
}

export function fmtTime(s: number | null | undefined) {
  if (!s || !isFinite(s)) return "0:00";
  const m = Math.floor(s / 60);
  return `${m}:${String(Math.floor(s % 60)).padStart(2, "0")}`;
}
