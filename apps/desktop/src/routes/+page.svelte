<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    player,
    scan,
    savedFolder,
    play,
    togglePause,
    next,
    prev,
    seek,
    setVolume,
    connect,
    showView,
    fmtTime,
  } from "$lib/player.svelte";

  let query = $state("");
  let scanning = $state(false);
  let dragPos = $state<number | null>(null);

  const track = $derived(player.track);
  const shown = $derived.by(() => {
    const q = query.trim().toLowerCase();
    const all = player.tracks.map((t, i) => ({ t, i }));
    if (!q) return all;
    return all.filter(({ t }) =>
      `${t.title} ${t.artist} ${t.album}`.toLowerCase().includes(q),
    );
  });

  $effect(() => connect());

  $effect(() => {
    const folder = savedFolder();
    if (folder) rescan(folder);
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

  function onKey(e: KeyboardEvent) {
    if (e.target instanceof HTMLInputElement) return;
    if (e.code === "Space") {
      e.preventDefault();
      togglePause();
    }
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="app" style="--accent: {player.accent}">
  <section class="library">
    <header>
      <h1>Library</h1>
      <span class="count">{player.tracks.length} tracks</span>
      <input placeholder="Search" bind:value={query} />
      <button class="pill ghost" onclick={() => showView("compact")}>Mini player</button>
      <button class="pill" onclick={pickFolder} disabled={scanning}>
        {scanning ? "Scanning…" : "Choose folder"}
      </button>
    </header>

    {#if player.tracks.length === 0}
      <div class="empty">
        <p>No music yet.</p>
        <button class="pill" onclick={pickFolder}>Choose your music folder</button>
      </div>
    {:else}
      <!-- shortcut: renders every row; add virtualization past ~10k tracks. -->
      <ol class="tracks">
        {#each shown as { t, i } (t.uri)}
          <li class:active={i === player.status.index}>
            <button class="row" ondblclick={() => play(i)}>
              <span class="no">{t.track_no ?? ""}</span>
              <span class="title">{t.title}</span>
              <span class="artist">{t.artist}</span>
              <span class="album">{t.album}</span>
              <span class="dur">{fmtTime(t.duration)}</span>
            </button>
          </li>
        {/each}
      </ol>
    {/if}
  </section>

  <aside class="now">
    {#if player.coverUrl}
      <div class="backdrop" style="background-image: url({player.coverUrl})"></div>
    {/if}
    <div class="cover">
      {#if player.coverUrl}
        <img src={player.coverUrl} alt="" />
      {:else}
        <svg viewBox="0 0 24 24"><path d="M12 3v10.55A4 4 0 1 0 14 17V7h4V3z" /></svg>
      {/if}
    </div>
    <h2>{track?.title ?? "Nothing playing"}</h2>
    <p>{track ? `${track.artist} · ${track.album}` : "Double-click a track"}</p>
  </aside>

  <footer class="bar">
    <div class="buttons">
      <button class="icon" onclick={prev} aria-label="Previous">
        <svg viewBox="0 0 24 24"><path d="M6 6h2v12H6zm3.5 6 8.5 6V6z" /></svg>
      </button>
      <button class="icon play" onclick={togglePause} aria-label="Play/Pause">
        {#if player.status.paused || player.status.index == null}
          <svg viewBox="0 0 24 24"><path d="M8 5v14l11-7z" /></svg>
        {:else}
          <svg viewBox="0 0 24 24"><path d="M6 5h4v14H6zm8 0h4v14h-4z" /></svg>
        {/if}
      </button>
      <button class="icon" onclick={next} aria-label="Next">
        <svg viewBox="0 0 24 24"><path d="M16 6h2v12h-2zM6 18l8.5-6L6 6z" /></svg>
      </button>
    </div>

    <div class="seek">
      <span>{fmtTime(dragPos ?? player.status.position)}</span>
      <input
        type="range"
        min="0"
        max={player.status.duration || 1}
        step="0.1"
        value={dragPos ?? player.status.position}
        oninput={(e) => (dragPos = +e.currentTarget.value)}
        onchange={(e) => {
          seek(+e.currentTarget.value);
          dragPos = null;
        }}
      />
      <span>{fmtTime(player.status.duration)}</span>
    </div>

    <label class="volume">
      <svg viewBox="0 0 24 24"><path d="M3 9v6h4l5 5V4L7 9zm13.5 3A4.5 4.5 0 0 0 14 8v8a4.5 4.5 0 0 0 2.5-4" /></svg>
      <input
        type="range"
        min="0"
        max="100"
        value={player.status.volume}
        oninput={(e) => setVolume(+e.currentTarget.value)}
      />
    </label>
  </footer>
</div>

<style>
  :global(body) {
    margin: 0;
    font-family: "Inter", system-ui, sans-serif;
    background: #0f0f12;
    color: #e8e8ee;
    overflow: hidden;
  }
  :global(button) {
    font: inherit;
    color: inherit;
  }

  .app {
    --fg-dim: #9a9aa8;
    --accent-c: rgb(var(--accent));
    height: 100vh;
    display: grid;
    grid-template-columns: 1fr 340px;
    grid-template-rows: 1fr 88px;
  }

  .library {
    overflow: hidden;
    display: flex;
    flex-direction: column;
    padding: 20px 8px 0 24px;
  }
  header {
    display: flex;
    align-items: center;
    gap: 14px;
    padding-bottom: 14px;
  }
  h1 {
    margin: 0;
    font-size: 26px;
    font-weight: 700;
  }
  .count {
    color: var(--fg-dim);
    font-size: 13px;
    margin-right: auto;
  }
  header input {
    background: #1c1c22;
    border: 1px solid #2a2a33;
    border-radius: 999px;
    padding: 7px 14px;
    color: inherit;
    width: 200px;
    outline: none;
  }
  header input:focus {
    border-color: var(--accent-c);
  }
  .pill {
    background: var(--accent-c);
    color: #111;
    border: 0;
    border-radius: 999px;
    padding: 8px 16px;
    font-weight: 600;
    cursor: pointer;
  }
  .ghost {
    background: none;
    color: inherit;
    border: 1px solid #2f2f3a;
  }
  .empty {
    flex: 1;
    display: grid;
    place-content: center;
    justify-items: center;
    color: var(--fg-dim);
  }

  .tracks {
    list-style: none;
    margin: 0;
    padding: 0 12px 12px 0;
    overflow-y: auto;
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
    padding: 9px 10px;
    cursor: default;
    font-size: 14px;
  }
  .row span {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
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

  .now {
    position: relative;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 24px;
    text-align: center;
    background: #15151a;
  }
  .backdrop {
    position: absolute;
    inset: -40px;
    background-size: cover;
    background-position: center;
    filter: blur(50px) saturate(1.4);
    opacity: 0.35;
  }
  .cover {
    position: relative;
    width: 260px;
    aspect-ratio: 1;
    border-radius: 14px;
    overflow: hidden;
    background: #23232b;
    display: grid;
    place-items: center;
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.55);
  }
  .cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .cover svg {
    width: 90px;
    fill: #3a3a46;
  }
  .now h2,
  .now p {
    position: relative;
    margin: 18px 0 0;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .now h2 {
    font-size: 20px;
  }
  .now p {
    margin-top: 6px;
    color: var(--fg-dim);
    font-size: 14px;
  }

  .bar {
    grid-column: 1 / -1;
    display: grid;
    grid-template-columns: auto 1fr 180px;
    align-items: center;
    gap: 28px;
    padding: 0 28px;
    background: #121216;
    border-top: 1px solid #22222a;
  }
  .buttons {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .icon {
    width: 42px;
    height: 42px;
    border-radius: 50%;
    border: 0;
    background: none;
    display: grid;
    place-items: center;
    cursor: pointer;
  }
  .icon:hover {
    background: #22222a;
  }
  svg {
    width: 22px;
    fill: currentColor;
  }
  .play {
    width: 52px;
    height: 52px;
    background: var(--accent-c);
    color: #111;
  }
  .play:hover {
    background: var(--accent-c);
    filter: brightness(1.1);
  }
  .play svg {
    width: 26px;
  }
  .seek,
  .volume {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 12px;
    color: var(--fg-dim);
    font-variant-numeric: tabular-nums;
  }
  input[type="range"] {
    flex: 1;
    accent-color: var(--accent-c);
  }
</style>
