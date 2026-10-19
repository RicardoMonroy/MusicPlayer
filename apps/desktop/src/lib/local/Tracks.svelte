<script lang="ts">
  import TrackList from "$lib/TrackList.svelte";
  import { play } from "$lib/player.svelte";
  import type { Track } from "$lib/player.svelte";
  import type { TrackSort } from "./library";
  let { tracks, onplay, offset = 0, sortable = false, sort = $bindable(null) }: { tracks: Track[]; onplay: (index: number) => void; offset?: number; sortable?: boolean; sort?: TrackSort } = $props();

  function keyboard(node: HTMLElement) {
    // TrackList uses double-click; give its existing buttons keyboard activation too.
    const activate = (event: KeyboardEvent) => {
      if (event.key !== "Enter" && event.key !== " ") return;
      const target = event.target;
      if (!(target instanceof HTMLButtonElement) || !target.classList.contains("row")) return;
      event.preventDefault();
      target.dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));
    };
    node.addEventListener("keydown", activate);
    return { destroy() { node.removeEventListener("keydown", activate); } };
  }
</script>
<div use:keyboard><TrackList {tracks} {onplay} {offset} {sortable} bind:sort onplaysorted={sortable ? (queue, index) => { void play(queue, index); } : undefined} /></div>
