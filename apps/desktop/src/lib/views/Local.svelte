<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { player, scan, savedFolder, play } from "$lib/player.svelte";
  import TrackList from "$lib/TrackList.svelte";

  let query = $state("");
  let scanning = $state(false);

  const shown = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (!q) return player.tracks;
    return player.tracks.filter((t) =>
      `${t.title} ${t.artist} ${t.album}`.toLowerCase().includes(q),
    );
  });

  $effect(() => {
    const folder = savedFolder();
    if (folder && player.tracks.length === 0) rescan(folder);
  });

  async function rescan(folder: string) {
    scanning = true;
    try {
      await scan(folder);
    } finally {
      scanning = false;
    }
  }

  async function pickFolder() {
    const folder = await open({ directory: true });
    if (typeof folder === "string") rescan(folder);
  }
</script>

<section class="view">
  <header>
    <h1>Biblioteca</h1>
    <span class="count">{player.tracks.length} canciones</span>
    <input class="field" placeholder="Buscar" bind:value={query} />
    <button class="pill" onclick={pickFolder} disabled={scanning}>
      {scanning ? "Escaneando…" : "Elegir carpeta"}
    </button>
  </header>

  {#if player.tracks.length === 0}
    <div class="empty">
      <p>Aún no hay música.</p>
      <button class="pill" onclick={pickFolder}>Elige tu carpeta de música</button>
    </div>
  {:else}
    <div class="scroll">
      <TrackList tracks={shown} onplay={(i) => play(shown, i)} />
    </div>
  {/if}
</section>
