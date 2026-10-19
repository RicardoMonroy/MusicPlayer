import type { Track } from "$lib/player.svelte";

export type Album = { id: string; title: string; artist: string; year: number | null; genre: string | null; tracks: Track[]; duration: number; search: string; discs: { number: number; tracks: Track[]; offset: number }[] };
export type Artist = { name: string; albums: Album[]; tracks: Track[]; search: string };
export type Folder = { id: string; name: string; children: Map<string, Folder>; tracks: Track[]; direct: Track[]; album?: Album; search: string };
export const parts = (path: string) => path.replace(/^file:\/\//, "").split(/[\\/]+/).filter(Boolean);
export const trackSearch = (t: Track) => `${t.title} ${t.artist} ${t.album_artist ?? ""} ${t.album} ${t.genre ?? ""}`.toLocaleLowerCase();
export function buildLibrary(tracks: Track[], rootPath: string) {
  const albums = new Map<string, Album>();
  const albumFor = new Map<Track, Album>();
  for (const t of tracks) {
    const artist = t.album_artist ?? t.artist;
    const id = JSON.stringify([artist, t.album]);
    let a = albums.get(id);
    if (!a) {
      a = { id, title: t.album || "Álbum desconocido", artist, year: t.year ?? null, genre: null, tracks: [], duration: 0, search: "", discs: [] };
      albums.set(id, a);
    }
    a.tracks.push(t);
    a.duration += t.duration ?? 0;
    a.search += ` ${trackSearch(t)}`;
    albumFor.set(t, a);
  }
  const artists = new Map<string, Artist>();
  for (const a of albums.values()) {
    a.genre = albumGenre(a.tracks);
    a.tracks.sort((x, y) => (x.disc_no ?? 1) - (y.disc_no ?? 1) || (x.track_no ?? 0) - (y.track_no ?? 0));
    for (const t of a.tracks) {
      const number = t.disc_no ?? 1;
      let disc = a.discs.at(-1);
      if (!disc || disc.number !== number) {
        disc = { number, tracks: [], offset: a.discs.reduce((n, d) => n + d.tracks.length, 0) };
        a.discs.push(disc);
      }
      disc.tracks.push(t);
    }
    let artist = artists.get(a.artist);
    if (!artist) {
      artist = { name: a.artist, albums: [], tracks: [], search: "" };
      artists.set(a.artist, artist);
    }
    artist.albums.push(a);
    artist.search += a.search;
  }
  for (const artist of artists.values()) {
    artist.albums.sort((a, b) => (a.year ?? Infinity) - (b.year ?? Infinity) || a.title.localeCompare(b.title));
    artist.tracks = artist.albums.flatMap(a => a.tracks);
  }
  const makeFolder = (id: string, name: string): Folder => ({ id, name, children: new Map(), tracks: [], direct: [], search: "" });
  const root = makeFolder("", parts(rootPath).at(-1) || "Biblioteca");
  const folders = new Map<string, Folder>([["", root]]);
  let prefix = parts(rootPath);
  if (!prefix.length && tracks.length) {
    prefix = parts(tracks[0].uri).slice(0, -1);
    for (const t of tracks) {
      const path = parts(t.uri);
      while (prefix.some((p, i) => path[i] !== p)) prefix.pop();
    }
  }
  for (const t of tracks) {
    const path = parts(t.uri).slice(0, -1);
    const relative = prefix.every((p, i) => path[i] === p) ? path.slice(prefix.length) : path;
    let folder = root;
    const add = () => { folder.tracks.push(t); folder.search += ` ${trackSearch(t)}`; };
    add();
    for (const name of relative) {
      const id = JSON.stringify([...partsFromId(folder.id), name]);
      let child = folder.children.get(name);
      if (!child) { child = makeFolder(id, name); folder.children.set(name, child); folders.set(id, child); }
      folder = child;
      add();
    }
    folder.direct.push(t);
  }
  for (const f of folders.values()) {
    const a = albumFor.get(f.tracks[0]);
    if (a && f.tracks.every(t => albumFor.get(t) === a)) f.album = a;
  }
  return { albums, artists, root, folders };
}
export const partsFromId = (id: string): string[] => id ? JSON.parse(id) : [];
export function shuffled(tracks: Track[]) {
  const copy = [...tracks];
  for (let i = copy.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [copy[i], copy[j]] = [copy[j], copy[i]];
  }
  return copy;
}

export type AlbumSort = "artist" | "year" | "title" | "genre";
export type TrackColumn = "track_no" | "title" | "artist" | "album" | "genre" | "duration";
export type TrackSort = { column: TrackColumn; descending: boolean } | null;

/** Missing metadata stays last in either direction. */
export function compareValues(a: string | number | null | undefined, b: string | number | null | undefined, descending = false): number {
  const missing = (v: typeof a) => v == null || (typeof v === "string" && !v.trim()) || (typeof v === "number" && !Number.isFinite(v));
  if (missing(a)) return missing(b) ? 0 : 1;
  if (missing(b)) return -1;
  const result = typeof a === "number" && typeof b === "number" ? a - b : String(a).localeCompare(String(b), "es", { sensitivity: "base" });
  return descending ? -result : result;
}

/** Ignore untagged tracks; tied genres follow Spanish alphabetical order. */
export function albumGenre(tracks: Track[]): string | null {
  const counts = new Map<string, number>();
  for (const t of tracks) {
    const genre = t.genre?.trim();
    if (genre) counts.set(genre, (counts.get(genre) ?? 0) + 1);
  }
  return [...counts].sort((a, b) => b[1] - a[1] || compareValues(a[0], b[0]))[0]?.[0] ?? null;
}

export function compareAlbums(a: Album, b: Album, sort: AlbumSort, yearAscending = false): number {
  if (sort === "title") return compareValues(a.title, b.title);
  if (sort === "year") return compareValues(a.year, b.year, !yearAscending) || compareValues(a.title, b.title);
  if (sort === "genre") return compareValues(a.genre, b.genre) || compareValues(a.artist, b.artist) || compareValues(a.year, b.year) || compareValues(a.title, b.title);
  return compareValues(a.artist, b.artist) || compareValues(a.year, b.year) || compareValues(a.title, b.title);
}

export function groupAlbums(albums: Album[], sort: AlbumSort): { name: string; albums: Album[] }[] {
  const groups = new Map<string, Album[]>();
  for (const album of albums) {
    const name = sort === "artist" ? album.artist || "Artista desconocido" : sort === "genre" ? album.genre ?? "Sin género" : "";
    const group = groups.get(name);
    if (group) group.push(album); else groups.set(name, [album]);
  }
  return [...groups].map(([name, albums]) => ({ name, albums }));
}

export function sortTracks(tracks: Track[], sort: TrackSort): Track[] {
  return sort ? [...tracks].sort((a, b) => compareValues(a[sort.column], b[sort.column], sort.descending)) : tracks;
}
