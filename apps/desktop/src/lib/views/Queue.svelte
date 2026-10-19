<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { player, jump, fmtTime, type Track } from "$lib/player.svelte";
  import Cover from "$lib/local/Cover.svelte";

  type Group = { album: string; artist: string; first: Track; start: number; tracks: Track[] };

  /** Consecutive tracks of the same album share a header: the queue is mostly whole albums. */
  const groups = $derived.by(() => {
    const out: Group[] = [];
    player.queue.forEach((t, i) => {
      const last = out.at(-1);
      if (last && last.album === t.album && last.first.album_id == t.album_id) last.tracks.push(t);
      else out.push({ album: t.album, artist: t.album_artist || t.artist, first: t, start: i, tracks: [t] });
    });
    return out;
  });
  const total = $derived(player.queue.reduce((s, t) => s + (t.duration ?? 0), 0));

  let copied = $state<string | null>(null);
  let copyError = $state<string | null>(null);
  async function copyAlbumUrl(albumId: string) {
    copyError = null;
    try {
      await invoke("copy_album_url", { albumId });
      copied = albumId;
      setTimeout(() => copied === albumId && (copied = null), 2000);
    } catch (e) {
      copyError = `No se pudo copiar la URL: ${e}`;
    }
  }

  let list: HTMLElement | undefined = $state();
  $effect(() => {
    const i = player.status.index;
    if (i == null || !list) return;
    list.querySelector(`[data-i="${i}"]`)?.scrollIntoView({ block: "nearest" });
  });
</script>

<section class="view queue">
  <header>
    <h1>Cola</h1>
    <span class="count">{player.queue.length} canciones · {fmtTime(total)}</span>
  </header>
  {#if copyError}<p class="error">{copyError}</p>{/if}
  {#if player.queue.length === 0}
    <div class="empty"><p>La cola está vacía.</p><p>Reproduce un álbum o una lista desde la Biblioteca o YouTube Music.</p></div>
  {:else}
    <div class="scroll" bind:this={list}>
      {#each groups as g (g.start)}
        <article>
          <div class="head">
            <div class="art">
              {#if g.first.cover}<img src={g.first.cover} alt="" />{:else}<Cover uri={g.first.uri} title={g.album} />{/if}
            </div>
            <div class="meta">
              <strong>{g.album || "Sin álbum"}</strong>
              <span>{g.artist}{g.first.year ? ` · ${g.first.year}` : ""} · {g.tracks.length} {g.tracks.length === 1 ? "canción" : "canciones"}</span>
            </div>
            {#if g.first.album_id}
              {@const id = g.first.album_id}
              <button class="pill ghost" onclick={() => copyAlbumUrl(id)} title="Copia el enlace del álbum de YouTube Music (para descargarlo)">
                {copied === id ? "¡Copiada!" : "Copiar URL del álbum"}
              </button>
            {/if}
          </div>
          <ol>
            {#each g.tracks as t, j (g.start + j)}
              {@const i = g.start + j}
              <li data-i={i} class:current={i === player.status.index}>
                <button ondblclick={() => jump(i)} title="Doble clic para reproducir">
                  <span class="n">{i === player.status.index ? (player.status.paused ? "❚❚" : "▶") : (t.track_no ?? j + 1)}</span>
                  <span class="t">{t.title}{#if t.artist !== g.artist}<small>{t.artist}</small>{/if}</span>
                  <span class="d">{fmtTime(t.duration)}</span>
                </button>
              </li>
            {/each}
          </ol>
        </article>
      {/each}
    </div>
  {/if}
</section>

<style>
  .scroll { flex: 1; min-height: 0; }
  .error { color: #f87171; font-size: 13px; margin: 0 0 12px; }
  article { margin-bottom: 22px; }
  .head { display: flex; flex-wrap: wrap; align-items: center; gap: 14px; padding: 10px; background: var(--surface); border-radius: var(--radius); }
  .art { width: 56px; height: 56px; flex: none; overflow: hidden; border-radius: var(--radius); font-size: 18px; }
  .art img { width: 100%; height: 100%; object-fit: cover; }
  .meta { flex: 1; min-width: 140px; display: grid; gap: 3px; }
  .meta strong, .meta span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .meta span { color: var(--fg-dim); font-size: 13px; }
  .head .pill { font-size: 13px; padding: 6px 14px; margin-left: auto; }
  ol { list-style: none; margin: 4px 0 0; padding: 0; }
  li button {
    width: 100%; display: grid; grid-template-columns: 40px 1fr auto; align-items: center; gap: 10px;
    padding: 8px 10px; background: none; border: 0; border-radius: var(--radius); color: inherit; font: inherit; text-align: left; cursor: default;
  }
  li button:hover { background: var(--surface); }
  li.current button { color: var(--accent-c); font-weight: 600; }
  .n, .d { color: var(--fg-dim); font-variant-numeric: tabular-nums; font-size: 13px; }
  .n { text-align: right; }
  li.current .n { color: var(--accent-c); }
  .t { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .t small { color: var(--fg-dim); margin-left: 8px; font-weight: 400; }
</style>
