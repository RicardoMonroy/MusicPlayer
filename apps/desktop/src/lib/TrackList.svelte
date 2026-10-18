<script lang="ts">
  import { player, fmtTime, type Track } from "$lib/player.svelte";

  let { tracks, onplay }: { tracks: Track[]; onplay: (index: number) => void } = $props();
</script>

<!-- shortcut: renders every row; add virtualization past ~10k tracks. -->
<ol class="tracks">
  {#each tracks as t, i (t.uri + i)}
    <li class:active={t.uri === player.track?.uri}>
      <button class="row" ondblclick={() => onplay(i)}>
        <span class="no">{t.track_no ?? i + 1}</span>
        <span class="title">
          {#if t.cover}<img src={t.cover} alt="" loading="lazy" />{/if}
          {t.title}
        </span>
        <span class="artist">{t.artist}</span>
        <span class="album">{t.album}</span>
        <span class="dur">{fmtTime(t.duration)}</span>
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
