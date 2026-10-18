import { invoke } from "@tauri-apps/api/core";

export type Track = {
  title: string;
  artist: string;
  album: string;
  uri: string;
  track_no: number | null;
  duration: number | null;
};

export type Status = {
  index: number | null;
  paused: boolean;
  position: number;
  duration: number;
  volume: number;
};

export const player = $state({
  tracks: [] as Track[],
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

export const play = (index: number) => invoke("play", { index });
export const togglePause = () => invoke("toggle_pause");
export const next = () => invoke("next");
export const prev = () => invoke("prev");
export const seek = (seconds: number) => invoke("seek", { seconds });
export const setVolume = (volume: number) => invoke("set_volume", { volume });

export function current(): Track | undefined {
  const i = player.status.index;
  return i == null ? undefined : player.tracks[i];
}

async function loadCover(track: Track | undefined) {
  if (player.coverUrl) URL.revokeObjectURL(player.coverUrl);
  player.coverUrl = "";
  if (!track) return;
  const bytes = await invoke<ArrayBuffer>("cover", { uri: track.uri });
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

export function startPolling() {
  const tick = async () => {
    player.status = await invoke<Status>("status");
    const t = current();
    if ((t?.uri ?? "") !== coverFor) {
      coverFor = t?.uri ?? "";
      loadCover(t).catch(() => {});
    }
  };
  tick();
  const id = setInterval(tick, 250);
  return () => clearInterval(id);
}

export function fmtTime(s: number | null | undefined) {
  if (!s || !isFinite(s)) return "0:00";
  const m = Math.floor(s / 60);
  return `${m}:${String(Math.floor(s % 60)).padStart(2, "0")}`;
}
