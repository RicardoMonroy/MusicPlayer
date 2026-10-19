<script lang="ts">
  import { play } from "$lib/player.svelte";
  import type { Album } from "./library";
  import Cover from "./Cover.svelte";
  let { albums, onopen }: { albums: Album[]; onopen: (album: Album) => void } = $props();
</script>
<div class="grid">
  {#each albums as album (album.id)}
    <article>
      <button class="card" onclick={() => onopen(album)} aria-label={`Abrir ${album.title}, ${album.artist}`}>
        <Cover uri={album.tracks[0].uri} title={album.title} />
        <strong>{album.title}</strong><span>{album.artist}</span><span>{album.year ?? "Sin año"}</span>
      </button>
      <button class="play" aria-label={`Reproducir álbum ${album.title}`} onclick={() => play(album.tracks, 0)}>▶</button>
    </article>
  {/each}
</div>
<style>
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(155px, 1fr)); gap: 18px; }
  article { position: relative; min-width: 0; border-radius: var(--radius); }
  article:hover, article:focus-within { background: var(--surface-2); }
  button { color: var(--fg); font: inherit; cursor: pointer; border: 0; }
  .card { display: flex; flex-direction: column; gap: 5px; width: 100%; padding: 8px; background: none; text-align: left; border-radius: var(--radius); }
  strong, span { max-width: 100%; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }
  strong { margin-top: 5px; font-size: 14px; } span { color: var(--fg-dim); font-size: 12px; }
  .play { position: absolute; right: 18px; top: 18px; width: 42px; height: 42px; border-radius: var(--radius-pill); background: rgb(var(--accent)); color: var(--on-accent); box-shadow: var(--raised); opacity: 0; }
  article:hover .play, article:focus-within .play { opacity: 1; }
  button:focus-visible { outline: 2px solid rgb(var(--accent)); outline-offset: 2px; }
  @media (hover: none) { .play { opacity: 1; } }
</style>
