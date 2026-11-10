import type { Track } from "$lib/player.svelte";

export type View = "local" | "youtube" | "queue";
/** A YouTube Music page to open, from anywhere in the app. */
export type YtTarget = { kind: "artist" | "album"; id: string; title: string };

export const nav = $state({
  view: (localStorage.getItem("view") as View | null) ?? "local",
  /** Picked up (and cleared) by the YouTube view. */
  yt: null as YtTarget | null,
});

export function openYt(target: YtTarget) {
  nav.yt = target;
  nav.view = "youtube";
}

export const openArtist = (t: Track) =>
  t.artist_id && openYt({ kind: "artist", id: t.artist_id, title: t.artist });
export const openAlbum = (t: Track) =>
  t.album_id && openYt({ kind: "album", id: t.album_id, title: t.album });
