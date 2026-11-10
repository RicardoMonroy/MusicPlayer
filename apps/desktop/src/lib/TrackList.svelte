<script lang="ts">
  import { player, fmtTime, type Track } from "$lib/player.svelte";
  import { openArtist, openAlbum } from "$lib/nav.svelte";

  import { sortTracks, type TrackSort, type TrackColumn } from "$lib/local/library";

  let {
    tracks,
    onplay,
    offset = 0,
    sortable = false,
    sort = $bindable(null),
    onplaysorted,
  }: { tracks: Track[]; onplay: (index: number) => void; offset?: number; sortable?: boolean; sort?: TrackSort; onplaysorted?: (tracks: Track[], index: number) => void } = $props();

  /** Fixed row height, so long lists can render only the rows on screen. */
  const ROW = 48;
  const VIRTUAL_AFTER = 200;
  const OVERSCAN = 12;

  const columns: { key: TrackColumn; label: string }[] = [
    { key: "track_no", label: "#" }, { key: "title", label: "Título" },
    { key: "artist", label: "Artista" }, { key: "album", label: "Álbum" },
    { key: "genre", label: "Género" }, { key: "duration", label: "Duración" },
  ];
  const ordered = $derived(sortable ? sortTracks(tracks, sort) : tracks);
  function toggleSort(column: TrackColumn) {
    sort = sort?.column !== column ? { column, descending: false }
      : !sort.descending ? { column, descending: true } : null;
  }
  function activate(index: number) {
    if (onplaysorted) onplaysorted(ordered, index);
    else onplay(tracks.indexOf(ordered[index]));
  }

  let list: HTMLElement | undefined = $state();
  let scrollTop = $state(0);
  let viewHeight = $state(800);

  // The list scrolls inside the nearest `.scroll` container of the view.
  $effect(() => {
    const scroller = list?.closest<HTMLElement>(".scroll");
    if (!scroller || tracks.length <= VIRTUAL_AFTER) return;
    const update = () => {
      scrollTop = scroller.getBoundingClientRect().top - list!.getBoundingClientRect().top + scroller.clientTop + (sortable ? ROW : 0);
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
    const start = Math.min(Math.max(0, tracks.length - 1), Math.max(0, Math.floor(scrollTop / ROW) - OVERSCAN));
    return { start, end: Math.min(tracks.length, start + Math.ceil(viewHeight / ROW) + 2 * OVERSCAN) };
  });
  const visible = $derived(ordered.slice(range.start, range.end));
</script>

{#if sortable}
  <table class="sortable">
    <thead><tr>
      {#each columns as column}
        <th scope="col" aria-sort={sort?.column === column.key ? (sort.descending ? "descending" : "ascending") : "none"}>
          <button onclick={() => toggleSort(column.key)}>{column.label}{#if sort?.column === column.key}<span aria-hidden="true"> {sort.descending ? "▼" : "▲"}</span>{/if}</button>
        </th>
      {/each}
    </tr></thead>
    <tbody bind:this={list}>
      {#if range.start}<tr aria-hidden="true"><td colspan="6" class="spacer" style:height="{range.start * ROW}px"></td></tr>{/if}
      {#each visible as t, j (t.uri + (range.start + j))}
        {@const i = range.start + j}
        <tr class:active={t.uri === player.track?.uri} ondblclick={() => activate(i)}>
          <td class="no">{t.track_no ?? "—"}</td>
          <td><button class="track-title" onclick={event => { if (event.detail === 0) activate(i); }}>{#if t.cover}<img src={t.cover} alt="" loading="lazy" />{/if}{t.title}</button></td>
          <td><button class="nav-link" disabled={!t.artist_id} onclick={() => openArtist(t)}>{t.artist || "—"}</button></td>
          <td><button class="nav-link" disabled={!t.album_id} onclick={() => openAlbum(t)}>{t.album || "—"}</button></td><td>{t.genre || "—"}</td>
          <td class="dur">{t.duration != null ? fmtTime(t.duration) : "—"}</td>
        </tr>
      {/each}
      {#if range.end < ordered.length}<tr aria-hidden="true"><td colspan="6" class="spacer" style:height="{(ordered.length - range.end) * ROW}px"></td></tr>{/if}
    </tbody>
  </table>
{:else}
<ol
  class="tracks"
  bind:this={list}
  style:padding-top="{range.start * ROW}px"
  style:padding-bottom="{(tracks.length - range.end) * ROW}px"
>
  {#each visible as t, j (t.uri + (range.start + j))}
    {@const i = range.start + j}
    <li class:active={t.uri === player.track?.uri}>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="row" ondblclick={() => onplay(i)}>
        <span class="no">{t.track_no ?? offset + i + 1}</span>
        <span class="title">
          {#if t.cover}<img src={t.cover} alt="" loading="lazy" />{/if}
          {t.title}
        </span>
        <span class="artist"><button class="nav-link" disabled={!t.artist_id} onclick={() => openArtist(t)}>{t.artist}</button></span>
        <span class="album"><button class="nav-link" disabled={!t.album_id} onclick={() => openAlbum(t)}>{t.album}</button></span>
        <span class="dur">{t.duration ? fmtTime(t.duration) : "—"}</span>
      </div>
    </li>
  {/each}
</ol>
{/if}

<style>
  .sortable { width: 100%; table-layout: fixed; border-collapse: separate; border-spacing: 0; color: var(--fg); font-family: var(--font); }
  .sortable th { position: sticky; top: 0; z-index: 1; height: 48px; background: var(--panel); text-align: left; }
  .sortable th:first-child { width: 44px; }
  .sortable th:nth-child(2) { width: 30%; }
  .sortable th:last-child { width: 90px; }
  .sortable th button { width: 100%; height: 100%; padding: 8px; border: 0; background: none; color: var(--fg); font: inherit; text-align: inherit; cursor: pointer; }
  .sortable td { height: 48px; box-sizing: border-box; padding: 8px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--fg-dim); font-size: 14px; }
  .sortable .spacer { padding: 0; border: 0; }
  .sortable tbody tr:hover { background: var(--surface-2); }
  .sortable .active { background: rgba(var(--accent), 0.16); }
  .track-title { display: flex; align-items: center; gap: 10px; width: 100%; padding: 0; border: 0; background: none; color: var(--fg); font: inherit; text-align: left; overflow: hidden; white-space: nowrap; }
  .track-title img { width: 32px; height: 32px; object-fit: cover; border-radius: var(--radius); flex: none; }
  .sortable .active .track-title { color: rgb(var(--accent)); }
  .sortable button:focus-visible { outline: 2px solid rgb(var(--accent)); outline-offset: -2px; }
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
    border-radius: var(--radius);
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
    background: var(--surface-2);
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
    color: rgb(var(--accent));
    font-weight: 600;
  }
</style>
