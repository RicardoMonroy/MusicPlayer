import type { Track } from "$lib/player.svelte";
import { albumGenre, compareAlbums, compareValues, groupAlbums, sortTracks, buildLibrary, shuffled } from "./library";

function assert(value: unknown, message: string): asserts value {
  if (!value) throw new Error(message);
}
const track = (title: string, album: string, uri: string, overrides: Partial<Track> = {}): Track => ({
  title, album, uri, artist: "Grupo", album_artist: "Grupo", year: 2000,
  disc_no: 1, track_no: 1, duration: 60, cover: null, ...overrides,
});
const tracks = [
  track("Segundo", "Doble", "/music/Grupo/Doble/CD2/02.flac", { artist: "Invitado", disc_no: 2 }),
  track("Primero", "Doble", "/music/Grupo/Doble/CD1/01.flac"),
  track("Antes", "Debut", "/music/Grupo/Debut/01.flac", { album_artist: null, year: 1990 }),
];
const library = buildLibrary(tracks, "/music");
const album = [...library.albums.values()][0];
assert(library.albums.size === 2, "Guest artists must not split an album");
assert(album.tracks[0].title === "Primero", "Album tracks follow disc order");
assert(album.discs[1].offset === 1, "Disc offsets address the whole album queue");
assert(album.duration === 120, "Album duration includes every disc");
assert(library.artists.get("Grupo")?.albums[0].title === "Debut", "Artist albums follow chronological order");
assert(library.root.tracks.every((t, i) => t === tracks[i]), "Folder queues preserve library order");
const group = library.root.children.get("Grupo");
assert(group?.tracks.length === 3, "Folder counts include descendants");
assert(group.children.get("Doble")?.album === album, "Album folders expose their cover album");
assert(group.children.get("Doble")?.direct.length === 0, "Direct tracks exclude child folders");
const windows = buildLibrary([track("Windows", "A", "C:\\Music\\Grupo\\A\\1.flac")], "C:\\Music");
assert(windows.root.children.get("Grupo")?.children.get("A")?.direct.length === 1, "Windows separators work");
assert(buildLibrary([], "").folders.size === 1, "Empty libraries retain a root folder");
const inferred = buildLibrary(tracks, "");
assert(inferred.root.children.has("Doble"), "Without a saved root, infer the common directory");
const copy = shuffled(tracks);
assert(copy !== tracks && copy.length === tracks.length && new Set(copy).size === tracks.length, "Shuffle copies every track once");
assert(tracks[0].title === "Segundo", "Shuffle and album sorting leave the library unchanged");
console.log("Library assertions passed");

const tagged = [
  track("Árbol", "Época", "/a", { genre: "Heavy Metal", year: 2020 }),
  track("Zorro", "Época", "/b", { genre: "Goth" }),
  track("Eco", "Época", "/c", { genre: "Heavy Metal" }),
  track("Sin etiqueta", "Época", "/d", { genre: null }),
];
assert(albumGenre(tagged) === "Heavy Metal", "Most frequent tagged genre wins");
assert(albumGenre([track("A", "A", "/empty", { genre: "  " })]) === null, "Blank genres are missing");
assert(albumGenre(tagged.slice(0, 2)) === "Goth", "Genre ties use alphabetical order");
const genreLibrary = buildLibrary([
  ...tagged,
  track("Otro", "Álbum", "/e", { genre: "Goth", album_artist: "Águila", year: null }),
  track("Nada", "Zeta", "/f", { genre: null, year: 1990 }),
], "");
const albums = [...genreLibrary.albums.values()];
assert(albums[0].genre === "Heavy Metal", "Built albums expose their dominant genre");
const byArtist = [...albums].sort((a, b) => compareAlbums(a, b, "artist"));
const artistGroups = groupAlbums(byArtist, "artist");
assert(artistGroups[0].name === "Águila" && artistGroups[1].albums.length === 2, "Artist grouping uses album artist and Spanish ordering");
assert(artistGroups[1].albums[0].year === 1990, "Artist albums sort by year ascending");
const genreGroups = groupAlbums([...albums].sort((a, b) => compareAlbums(a, b, "genre")), "genre");
assert(genreGroups.map(g => g.name).join(",") === "Goth,Heavy Metal,Sin género", "Genre sections sort alphabetically with missing last");
assert([...albums].sort((a, b) => compareAlbums(a, b, "title"))[0].title === "Álbum", "Album title sorting ignores accents");
assert([...albums].sort((a, b) => compareAlbums(a, b, "year"))[0].year === 2020, "Year defaults to newest first");
assert([...albums].sort((a, b) => compareAlbums(a, b, "year", true)).at(-1)?.year === null, "Missing years remain last ascending");
assert([...albums].sort((a, b) => compareAlbums(a, b, "year")).at(-1)?.year === null, "Missing years remain last descending");
assert(compareValues("Árbol", "arbol") === 0 && compareValues("Época", "Zeta") < 0, "Spanish string comparison ignores case and accents");
for (const descending of [false, true]) {
  assert(compareValues(null, "A", descending) > 0, "Missing strings stay last");
  assert(compareValues(" ", "A", descending) > 0, "Blank strings stay last");
  assert(compareValues(undefined, 1, descending) > 0, "Missing numbers stay last");
}
const sortableTracks = [
  track("Zeta", "B", "/1", { track_no: 10, duration: 100, genre: null, artist: "Zorro" }),
  track("Árbol", "A", "/2", { track_no: 2, duration: 20, genre: "Goth", artist: "Águila" }),
  track("Eco", "C", "/3", { track_no: null, duration: null, genre: "Heavy Metal", artist: "" }),
];
assert(sortTracks(sortableTracks, null) === sortableTracks, "Clearing sort restores library order");
assert(sortTracks(sortableTracks, { column: "title", descending: false })[0].title === "Árbol", "Track titles use Spanish order");
for (const column of ["track_no", "duration"] as const) {
  assert(sortTracks(sortableTracks, { column, descending: false })[0].uri === "/2", "Track numbers sort numerically");
  const descending = sortTracks(sortableTracks, { column, descending: true });
  assert(descending[0].uri === "/1" && descending.at(-1)?.uri === "/3", "Descending numeric sort leaves missing last");
}
for (const column of ["artist", "album", "genre"] as const) {
  const sorted = sortTracks(sortableTracks, { column, descending: false });
  assert(sorted[0].uri === "/2", "Artist, album and genre columns sort alphabetically");
  assert(sortTracks(sortableTracks, { column, descending: true })[0].uri === (column === "genre" ? "/3" : column === "artist" ? "/1" : "/3"), "String columns reverse order");
}
assert(sortTracks(sortableTracks, { column: "genre", descending: true }).at(-1)?.uri === "/1", "Missing track genres stay last descending");
assert(sortableTracks[0].uri === "/1", "Sorting never mutates library order");
console.log("Sorting assertions passed");
