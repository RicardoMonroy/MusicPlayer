import { invoke } from "@tauri-apps/api/core";
import type { Track } from "$lib/player.svelte";

export type Kind = "album" | "playlist";

export type Collection = {
  kind: Kind;
  id: string;
  title: string;
  subtitle: string;
  cover: string | null;
};

export type SearchResults = { tracks: Track[]; albums: Collection[]; playlists: Collection[] };
export type Library = { playlists: Collection[]; albums: Collection[] };

export const search = (query: string) => invoke<SearchResults>("yt_search", { query });
export const openCollection = (c: Collection) =>
  invoke<Track[]>("yt_open", { kind: c.kind, id: c.id });
export const loggedIn = () => invoke<boolean>("yt_logged_in");
export const login = (path: string) => invoke("yt_login", { path });
export const logout = () => invoke("yt_logout");
export const library = () => invoke<Library>("yt_library");
export const liked = () => invoke<Track[]>("yt_liked");
