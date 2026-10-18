<script lang="ts">
  import { player, fmtTime, type Track } from "$lib/player.svelte";

  let {
    tracks,
    onplay,
    offset = 0,
  }: { tracks: Track[]; onplay: (index: number) => void; offset?: number } = $props();

  /** Fixed row height, so long lists can render only the rows on screen. */
  const ROW = 48;
  const VIRTUAL_AFTER = 200;
  const OVERSCAN = 12;

  let list: HTMLOListElement | undefined = $state();
  let scrollTop = $state(0);
  let viewHeight = $state(800);

  // The list scrolls inside the nearest `.scroll` container of the view.
  $effect(() => {
    const scroller = list?.closest<HTMLElement>(".scroll");
    if (!scroller || tracks.length <= VIRTUAL_AFTER) return;
    const update = () => {
      scrollTop = scroller.scrollTop - list!.offsetTop;
      viewHeight = scroller.clientHeight;
    };
    update();
    const ro = new ResizeObserver(update);
    ro.observe(scroller);
    scroller.addEventListener("scroll", update, { passive: true });
    return () => {
      ro.disconnect();
      scroller.removeEventListener("scroll", update);
    };
  });

  const range = $derived.by(() => {
    if (tracks.length <= VIRTUAL_AFTER) return { start: 0, end: tracks.length };
    const start = Math.max(0, Math.floor(scrollTop / ROW) - OVERSCAN);
    return { start, end: Math.min(tracks.length, start + Math.ceil(viewHeight / ROW) + 2 * OVERSCAN) };
  });
  const visible = $derived(tracks.slice(range.start, range.end));
</script>

<ol
  class="tracks"
  bind:this={list}
  style:padding-top="{range.start * ROW}px"
  style:padding-bottom="{(tracks.length - range.end) * ROW}px"
>
  {#each visible as t, j (t.uri + (range.start + j))}
    {@const i = range.start + j}
    <li class:active={t.uri === player.track?.uri}>
      <button class="row" ondblclick={() => onplay(i)}>
        <span class="no">{t.track_no ?? offset + i + 1}</span>
        <span class="title">
          {#if t.cover}<img src={t.cover} alt="" loading="lazy" />{/if}
          {t.title}
        </span>
        <span class="artist">{t.artist}</span>
        <span class="album">{t.album}</span>
        <span class="dur">{t.duration ? fmtTime(t.duration) : "—"}</span>
      </button>
    </li>
  {/each}
</ol>

<style>
  .tracks {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .row {
    width: 100%;
    height: 48px;
    box-sizing: border-box;
    display: grid;
    grid-template-columns: 36px 2fr 1.3fr 1.3fr 52px;
    gap: 12px;
    align-items: center;
    text-align: left;
    background: none;
    border: 0;
    border-radius: 8px;
    padding: 8px 10px;
    cursor: default;
    font-size: 14px;
  }
  .row span {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .title {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .title img {
    width: 32px;
    height: 32px;
    border-radius: 4px;
    object-fit: cover;
    flex: none;
  }
  .row:hover {
    background: #1b1b21;
  }
  .no,
  .dur,
  .artist,
  .album {
    color: var(--fg-dim);
  }
  .no,
  .dur {
    font-variant-numeric: tabular-nums;
    text-align: right;
  }
  .active .row {
    background: rgba(var(--accent), 0.16);
  }
  .active .title {
    color: var(--accent-c);
    font-weight: 600;
  }
</style>
