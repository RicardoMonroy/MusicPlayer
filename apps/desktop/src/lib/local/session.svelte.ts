import type { AlbumSort, TrackSort } from "./library";
export type Tab = "Álbumes" | "Artistas" | "Carpetas" | "Canciones";
export const librarySession = $state({
  tab: "Álbumes" as Tab,
  queries: { Álbumes: "", Artistas: "", Carpetas: "", Canciones: "" },
  albums: { album: "", artist: "" },
  artists: { album: "", artist: "" },
  albumSort: "artist" as AlbumSort,
  yearAscending: false,
  genre: null as string | null,
  trackSort: null as TrackSort,
  folder: "",
  root: "",
  initialized: false,
  busy: false,
  error: "",
});
