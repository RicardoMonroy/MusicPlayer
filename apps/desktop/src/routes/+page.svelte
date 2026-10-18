<script lang="ts">
  import "$lib/app.css";
  import {
    player,
    togglePause,
    next,
    prev,
    seek,
    setVolume,
    connect,
    showView,
    fmtTime,
  } from "$lib/player.svelte";
  import Local from "$lib/views/Local.svelte";
  import YouTube from "$lib/views/YouTube.svelte";

  type View = "local" | "youtube";
  const VIEW_KEY = "view";
  let view = $state<View>((localStorage.getItem(VIEW_KEY) as View) ?? "local");
  let dragPos = $state<number | null>(null);

  const track = $derived(player.track);

  $effect(() => connect());
  $effect(() => localStorage.setItem(VIEW_KEY, view));

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
  <nav class="side">
    <div class="brand">♫ MusicPlayer</div>
    <button class:on={view === "local"} onclick={() => (view = "local")}>
      <svg viewBox="0 0 24 24"><path d="M4 6h2v14H4zm4-2h2v16H8zm4 2h2v14h-2zm4.5-1.5 1.9-.6 4.2 14.3-1.9.6z" /></svg>
      Biblioteca
    </button>
    <button class:on={view === "youtube"} onclick={() => (view = "youtube")}>
      <svg viewBox="0 0 24 24"><path d="M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20m0 18a8 8 0 1 1 0-16 8 8 0 0 1 0 16m-2-12.5v9l6-4.5z" /></svg>
      YouTube Music
    </button>
    <button class="mini" onclick={() => showView("compact")}>
      <svg viewBox="0 0 24 24"><path d="M19 11h-8v6h8zm4 8V5a2 2 0 0 0-2-2H3a2 2 0 0 0-2 2v14c0 1.1.9 2 2 2h18a2 2 0 0 0 2-2m-2 0H3V5h18z" /></svg>
      Mini reproductor
    </button>
  </nav>

  {#if view === "local"}
    <Local />
  {:else}
    <YouTube />
  {/if}

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
    <h2>{track?.title ?? "Nada en reproducción"}</h2>
    <p>{track ? `${track.artist} · ${track.album}` : "Haz doble clic en una canción"}</p>
  </aside>

  <footer class="bar">
    <div class="buttons">
      <button class="icon" onclick={prev} aria-label="Anterior">
        <svg viewBox="0 0 24 24"><path d="M6 6h2v12H6zm3.5 6 8.5 6V6z" /></svg>
      </button>
      <button class="icon play" onclick={togglePause} aria-label="Reproducir/Pausar">
        {#if player.status.paused || player.status.index == null}
          <svg viewBox="0 0 24 24"><path d="M8 5v14l11-7z" /></svg>
        {:else}
          <svg viewBox="0 0 24 24"><path d="M6 5h4v14H6zm8 0h4v14h-4z" /></svg>
        {/if}
      </button>
      <button class="icon" onclick={next} aria-label="Siguiente">
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
    grid-template-columns: 210px minmax(0, 1fr) 340px;
    grid-template-rows: 1fr 88px;
  }

  .side {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 20px 12px;
    background: #0b0b0e;
    border-right: 1px solid #1d1d24;
  }
  .brand {
    font-weight: 800;
    font-size: 17px;
    color: var(--accent-c);
    padding: 4px 12px 18px;
  }
  .side button {
    display: flex;
    align-items: center;
    gap: 12px;
    background: none;
    border: 0;
    border-radius: 8px;
    padding: 10px 12px;
    text-align: left;
    color: var(--fg-dim);
    font-weight: 600;
    cursor: pointer;
  }
  .side button:hover {
    color: #fff;
    background: #17171d;
  }
  .side button.on {
    color: #fff;
    background: rgba(var(--accent), 0.18);
  }
  .side svg {
    width: 20px;
  }
  .side .mini {
    margin-top: auto;
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
