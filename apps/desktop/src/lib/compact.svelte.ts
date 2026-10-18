/** Panels the mini player can unfold under the skin; skins may toggle them too. */
export const compact = $state({
  playlist: localStorage.getItem("compact-playlist") === "1",
  eq: localStorage.getItem("compact-eq") === "1",
});
