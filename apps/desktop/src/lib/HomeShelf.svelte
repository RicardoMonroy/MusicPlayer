<script lang="ts">
  import type { Track } from "$lib/player.svelte";
  import type { Collection, Shelf } from "$lib/yt";

  let {
    shelf,
    onplay,
    onopen,
  }: {
    shelf: Shelf;
    onplay: (tracks: Track[], index: number) => void;
    onopen: (c: Collection) => void;
  } = $props();

  let row: HTMLDivElement | undefined = $state();
  // A song plays the shelf's songs from there, like the web app.
  const tracks = $derived(shelf.items.flatMap((i) => (i.type === "track" ? [i.item] : [])));
  const scroll = (dir: number) => row?.scrollBy({ left: dir * row.clientWidth * 0.8, behavior: "smooth" });
</script>

<section class="shelf">
  <header>
    <h3>{shelf.title}</h3>
    <div class="arrows">
      <button onclick={() => scroll(-1)} aria-label="Anteriores">‹</button>
      <button onclick={() => scroll(1)} aria-label="Siguientes">›</button>
    </div>
  </header>
  <div class="row" bind:this={row}>
    {#each shelf.items as it, i (i)}
      {#if it.type === "track"}
        {@const t = it.item}
        <button class="card" onclick={() => onplay(tracks, tracks.indexOf(t))} title="Reproducir">
          <div class="art">
            {#if t.cover}<img src={t.cover} alt="" loading="lazy" />{/if}
            <span class="play">▶</span>
          </div>
          <strong>{t.title}</strong>
          <span>{t.artist}</span>
        </button>
      {:else}
        {@const c = it.item}
        <button class="card" onclick={() => onopen(c)}>
          <div class="art" class:round={c.kind === "artist"}>
            {#if c.cover}<img src={c.cover} alt="" loading="lazy" />{/if}
          </div>
          <strong>{c.title}</strong>
          <span>{c.subtitle}</span>
        </button>
      {/if}
    {/each}
  </div>
</section>

<style>
  .shelf {
    margin-bottom: 26px;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: 0 0 10px;
  }
  h3 {
    margin: 0;
    font-size: 19px;
  }
  .arrows {
    display: flex;
    gap: 6px;
  }
  .arrows button {
    width: 30px;
    height: 30px;
    border-radius: var(--radius-pill);
    border: 1px solid var(--border);
    background: var(--surface);
    box-shadow: var(--raised);
    color: var(--fg);
    font-size: 18px;
    line-height: 1;
    cursor: pointer;
  }
  .arrows button:hover {
    background: var(--surface-2);
  }
  .row {
    display: grid;
    grid-auto-flow: column;
    grid-auto-columns: 168px;
    gap: 16px;
    overflow-x: auto;
    scroll-snap-type: x proximity;
    padding-bottom: 6px;
    scrollbar-width: thin;
  }
  .card {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 0;
    border: 0;
    background: none;
    color: inherit;
    text-align: left;
    cursor: pointer;
    scroll-snap-align: start;
    min-width: 0;
  }
  .art {
    position: relative;
    aspect-ratio: 1;
    border-radius: var(--radius);
    overflow: hidden;
    background: var(--art-bg);
    margin-bottom: 6px;
  }
  .art.round {
    border-radius: 50%;
  }
  .art img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 0.2s;
  }
  .card:hover img {
    transform: scale(1.04);
  }
  .play {
    position: absolute;
    right: 8px;
    bottom: 8px;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: rgb(var(--accent));
    color: var(--on-accent);
    opacity: 0;
    transition: opacity 0.15s;
  }
  .card:hover .play,
  .card:focus-visible .play {
    opacity: 1;
  }
  strong,
  span {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  strong {
    font-size: 14px;
  }
  span {
    font-size: 12px;
    color: var(--fg-dim);
  }
  @media (prefers-reduced-motion: reduce) {
    .card:hover img {
      transform: none;
    }
  }
</style>
