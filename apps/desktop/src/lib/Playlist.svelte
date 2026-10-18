<script lang="ts">
  import { player, jump, fmtTime } from "$lib/player.svelte";
  import type { PlaylistTheme } from "$lib/skins";

  let { theme, height }: { theme: PlaylistTheme; height: number } = $props();

  let list: HTMLOListElement | undefined = $state();
  const total = $derived(player.queue.reduce((s, t) => s + (t.duration ?? 0), 0));

  // Keep the current track in view when it changes.
  $effect(() => {
    const i = player.status.index;
    if (i == null || !list) return;
    list.children[i]?.scrollIntoView({ block: "nearest" });
  });
</script>

<section
  class="playlist"
  style:height="{height}px"
  style:--bg={theme.bg}
  style:--fg={theme.fg}
  style:--dim={theme.dim}
  style:--current={theme.current}
  style:--hover={theme.hover}
  style:--border={theme.border}
  style:--font={theme.font}
>
  <header data-tauri-drag-region>
    <span data-tauri-drag-region>Lista de reproducción</span>
    <span data-tauri-drag-region>{player.queue.length} · {fmtTime(total)}</span>
  </header>
  {#if player.queue.length === 0}
    <p class="empty">La cola está vacía. Elige algo en el reproductor completo.</p>
  {:else}
    <ol bind:this={list}>
      {#each player.queue as t, i (t.uri + i)}
        <li class:current={i === player.status.index}>
          <button ondblclick={() => jump(i)} title="Doble clic para reproducir">
            <span class="n">{i + 1}.</span>
            <span class="t">{t.artist ? `${t.artist} — ${t.title}` : t.title}</span>
            <span class="d">{fmtTime(t.duration)}</span>
          </button>
        </li>
      {/each}
    </ol>
  {/if}
</section>

<style>
  .playlist {
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    background: var(--bg);
    color: var(--fg);
    font-family: var(--font);
    font-size: 12px;
    border: 2px solid var(--border);
    border-top: 0;
    border-radius: 0 0 10px 10px;
    overflow: hidden;
  }
  header {
    display: flex;
    justify-content: space-between;
    padding: 6px 10px;
    color: var(--dim);
    font-size: 10px;
    letter-spacing: 1px;
    text-transform: uppercase;
    border-bottom: 1px solid var(--border);
  }
  ol {
    list-style: none;
    margin: 0;
    padding: 4px 0;
    overflow-y: auto;
    flex: 1;
  }
  button {
    width: 100%;
    display: grid;
    grid-template-columns: 28px 1fr auto;
    gap: 6px;
    padding: 3px 10px;
    border: 0;
    background: none;
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: default;
  }
  button:hover {
    background: var(--hover);
  }
  .n,
  .d {
    color: var(--dim);
    font-variant-numeric: tabular-nums;
  }
  .n {
    text-align: right;
  }
  .t {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .current button,
  .current .n,
  .current .d {
    color: var(--current);
    font-weight: 700;
  }
  .empty {
    margin: auto;
    padding: 16px;
    color: var(--dim);
    text-align: center;
  }
</style>
