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
};

export type Status = {
  index: number | null;
  paused: boolean;
  position: number;
  duration: number;
  volume: number;
};

export type Now = { status: Status; track: Track | null };

export const player = $state({
  tracks: [] as Track[],
  track: null as Track | null,
  status: { index: null, paused: false, position: 0, duration: 0, volume: 100 } as Status,
  coverUrl: "",
  accent: "120, 120, 160",
});

const FOLDER_KEY = "library-folder";
let coverFor = "";

export async function scan(folder: string) {
  localStorage.setItem(FOLDER_KEY, folder);
  player.tracks = await invoke<Track[]>("scan_folder", { path: folder });
}

export const savedFolder = () => localStorage.getItem(FOLDER_KEY);

/** Replace the queue with `tracks` and start at `index`. */
export const play = (tracks: Track[], index: number) =>
  invoke("play", { tracks: $state.snapshot(tracks), index });
export const togglePause = () => invoke("toggle_pause");
export const next = () => invoke("next");
export const prev = () => invoke("prev");
export const seek = (seconds: number) => invoke("seek", { seconds });
export const setVolume = (volume: number) => invoke("set_volume", { volume });

export const showView = (label: "main" | "compact") => invoke("show_view", { label });

async function loadCover(track: Track | null) {
  if (player.coverUrl) URL.revokeObjectURL(player.coverUrl);
  player.coverUrl = "";
  if (!track) return;
  const bytes = await invoke<ArrayBuffer>("cover", { track });
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
  player.track = now.track;
  if ((now.track?.uri ?? "") !== coverFor) {
    coverFor = now.track?.uri ?? "";
    loadCover(now.track).catch(() => {});
  }
}

/** Follow player state pushed from Rust. Returns the unsubscribe function. */
export function connect() {
  invoke<Now>("now").then(apply);
  const off = listen<Now>("now", (e) => apply(e.payload));
  return () => {
    off.then((f) => f());
  };
}

export function fmtTime(s: number | null | undefined) {
  if (!s || !isFinite(s)) return "0:00";
  const m = Math.floor(s / 60);
  return `${m}:${String(Math.floor(s % 60)).padStart(2, "0")}`;
}
