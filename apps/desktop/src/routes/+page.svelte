<script lang="ts">
  import "$lib/app.css";
  import { invoke } from "@tauri-apps/api/core";
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
  import Queue from "$lib/views/Queue.svelte";
  import { listen } from "@tauri-apps/api/event";
  import { savedSkin, skinById, applyTheme, DEFAULT_SKIN, HERO_SIZE } from "$lib/skins";
  import Settings, { type Section } from "$lib/Settings.svelte";
  import { typing } from "$lib/shortcuts";

  type View = "local" | "youtube" | "queue";
  const VIEW_KEY = "view";
  let view = $state<View>((localStorage.getItem(VIEW_KEY) as View) ?? "local");
  let beforeQueue: View = "local";
  let dragPos = $state<number | null>(null);

  const track = $derived(player.track);
  let showLyrics = $state(false);
  let translation = $state<string | null | undefined>(undefined);
  let translate = $state(localStorage.getItem("lyrics-translate") === "1");
  let translationError = $state("");

  function toggleTranslate() {
    translate = !translate;
    localStorage.setItem("lyrics-translate", translate ? "1" : "0");
  }

  // Translate once per song while translation is on (results are cached on disk).
  $effect(() => {
    const t = player.track;
    const text = lyrics;
    translationError = "";
    if (!translate || !t || !text) return;
    let stale = false;
    invoke<string>("translate_lyrics", { uri: t.uri, text })
      .then((tr) => !stale && (translation = tr))
      .catch((e) => {
        if (!stale) {
          translation = null;
          translationError = String(e);
        }
      });
    return () => (stale = true);
  });

  /** Width of the now-playing panel; dragged with the handle on its left edge. */
  let sideCollapsed = $state(localStorage.getItem("side-collapsed") === "1");
  const sidebarWidth = () => (sideCollapsed ? 64 : 210);
  function toggleSide() {
    sideCollapsed = !sideCollapsed;
    localStorage.setItem("side-collapsed", sideCollapsed ? "1" : "0");
    asideWidth = Math.min(asideWidth, asideMax());
  }
  const ASIDE_MIN = 280;
  let asideWidth = $state(Number(localStorage.getItem("aside-width")) || 340);
  const asideMax = () => Math.max(ASIDE_MIN, (window.innerWidth - sidebarWidth()) / 2);

  function resizeStart(e: PointerEvent) {
    const el = e.currentTarget as HTMLElement;
    el.setPointerCapture(e.pointerId);
    const move = (ev: PointerEvent) => {
      asideWidth = Math.round(Math.min(asideMax(), Math.max(ASIDE_MIN, window.innerWidth - ev.clientX)));
    };
    const up = () => {
      el.removeEventListener("pointermove", move);
      localStorage.setItem("aside-width", String(asideWidth));
    };
    el.addEventListener("pointermove", move);
    el.addEventListener("pointerup", up, { once: true });
  }
  let lyrics = $state<string | null | undefined>(undefined);

  // Fetch lyrics for the current track only while the panel is open.
  $effect(() => {
    const t = player.track;
    lyrics = undefined;
    translation = undefined;
    if (!showLyrics || !t) return;
    let stale = false;
    invoke<string | null>("lyrics", { track: $state.snapshot(t) })
      .then((l) => !stale && (lyrics = l))
      .catch(() => !stale && (lyrics = null));
    return () => (stale = true);
  });

  $effect(() => connect());

  let skinId = $state(DEFAULT_SKIN);
  savedSkin().then((s) => (skinId = s.id));
  // The skin is global: the full player takes its colors, fonts and shapes.
  $effect(() => applyTheme(skinById(skinId).theme, player.accent));
  let settingsOpen = $state(false);
  let settingsSection = $state<Section>("apariencia");
  const openSettings = (section: Section) => {
    settingsSection = section;
    settingsOpen = true;
  };
  const skin = $derived(skinById(skinId));
  $effect(() => {
    const off = listen<string>("skin", (e) => (skinId = e.payload));
    return () => {
      off.then((f) => f());
    };
  });
  $effect(() => localStorage.setItem(VIEW_KEY, view));

  // Shortcuts are documented in $lib/shortcuts.ts (shown in Ajustes › Atajos).
  function onKey(e: KeyboardEvent) {
    if (typing(e)) return;
    const ctrl = e.ctrlKey || e.metaKey;
    if (ctrl && e.key === ",") return (e.preventDefault(), openSettings("apariencia"));
    if (e.key === "?") return (e.preventDefault(), openSettings("atajos"));
    if (settingsOpen) return;
    if (ctrl && e.key.toLowerCase() === "f") {
      e.preventDefault();
      document.querySelector<HTMLInputElement>(".view input")?.focus();
      return;
    }
    if (ctrl && (e.key === "ArrowUp" || e.key === "ArrowDown")) {
      e.preventDefault();
      setVolume(Math.max(0, Math.min(100, player.status.volume + (e.key === "ArrowUp" ? 5 : -5))));
      return;
    }
    if (ctrl || e.altKey) return;
    const s = player.status;
    switch (e.key) {
      case " ":
        e.preventDefault();
        togglePause();
        break;
      case "ArrowLeft":
      case "ArrowRight":
        if (s.duration > 0) {
          e.preventDefault();
          seek(Math.max(0, Math.min(s.duration, s.position + (e.key === "ArrowRight" ? 5 : -5))));
        }
        break;
      case "n":
      case "N":
        next();
        break;
      case "p":
      case "P":
        prev();
        break;
      case "l":
      case "L":
        showLyrics = !showLyrics;
        break;
      case "t":
      case "T":
        showLyrics = true;
        toggleTranslate();
        break;
      case "e":
      case "E":
        openSettings("eq");
        break;
      case "q":
      case "Q":
        if (view !== "queue") beforeQueue = view;
        view = view === "queue" ? beforeQueue : "queue";
        break;
      case "m":
      case "M":
        showView("compact");
        break;
    }
  }
</script>

<svelte:window onkeydown={onKey} onresize={() => (asideWidth = Math.min(asideWidth, asideMax()))} />

<div class="app" style:--aside-w="{asideWidth}px" style:--side-w="{sidebarWidth()}px">
  <nav class="side" class:collapsed={sideCollapsed}>
    <div class="brand">
      <span class="label">♫ MusicPlayer</span>
      <button
        class="collapse"
        onclick={toggleSide}
        title={sideCollapsed ? "Expandir barra lateral" : "Contraer barra lateral"}
        aria-label={sideCollapsed ? "Expandir barra lateral" : "Contraer barra lateral"}
        aria-expanded={!sideCollapsed}
      >
        <svg viewBox="0 0 24 24"><path d={sideCollapsed ? "M9 6l6 6-6 6" : "M15 6l-6 6 6 6"} /></svg>
      </button>
    </div>
    <button title="Biblioteca" class:on={view === "local"} onclick={() => (view = "local")}>
      <svg viewBox="0 0 24 24"><path d="M4 6h2v14H4zm4-2h2v16H8zm4 2h2v14h-2zm4.5-1.5 1.9-.6 4.2 14.3-1.9.6z" /></svg>
      <span class="label">Biblioteca</span>
    </button>
    <button title="YouTube Music" class:on={view === "youtube"} onclick={() => (view = "youtube")}>
      <svg viewBox="0 0 24 24"><path d="M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20m0 18a8 8 0 1 1 0-16 8 8 0 0 1 0 16m-2-12.5v9l6-4.5z" /></svg>
      <span class="label">YouTube Music</span>
    </button>
    <button title="Cola de reproducción" class:on={view === "queue"} onclick={() => (view = "queue")}>
      <svg viewBox="0 0 24 24"><path d="M3 6h12v2H3zm0 5h12v2H3zm0 5h8v2H3zm14-5v6.3a3 3 0 1 0 2 2.7V13h3v-2z" /></svg>
      <span class="label">Cola</span>
    </button>
    <button title="Ajustes" class="gear" onclick={() => openSettings("apariencia")}>
      <svg viewBox="0 0 24 24"><path d="M19.4 13a7.5 7.5 0 0 0 0-2l2.1-1.6-2-3.5-2.5 1a7.6 7.6 0 0 0-1.7-1L15 3h-4l-.4 2.9a7.6 7.6 0 0 0-1.7 1l-2.5-1-2 3.5L6.6 11a7.5 7.5 0 0 0 0 2l-2.1 1.6 2 3.5 2.5-1c.5.4 1.1.7 1.7 1L11 21h4l.4-2.9c.6-.3 1.2-.6 1.7-1l2.5 1 2-3.5zM13 15.5a3.5 3.5 0 1 1 0-7 3.5 3.5 0 0 1 0 7" /></svg>
      <span class="label">Ajustes</span>
    </button>
    <button title="Mini reproductor" class="mini" onclick={() => showView("compact")}>
      <svg viewBox="0 0 24 24"><path d="M19 11h-8v6h8zm4 8V5a2 2 0 0 0-2-2H3a2 2 0 0 0-2 2v14c0 1.1.9 2 2 2h18a2 2 0 0 0 2-2m-2 0H3V5h18z" /></svg>
      <span class="label">Mini reproductor</span>
    </button>
  </nav>

  {#if view === "local"}
    <Local />
  {:else if view === "youtube"}
    <YouTube />
  {:else}
    <Queue />
  {/if}

  <aside class="now">
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="resizer"
      onpointerdown={resizeStart}
      ondblclick={() => {
        asideWidth = 340;
        localStorage.setItem("aside-width", "340");
      }}
      title="Arrastra para cambiar el ancho (doble clic: restablecer)"
    ></div>
    {#if player.coverUrl}
      <div class="backdrop" style="background-image: url({player.coverUrl})"></div>
    {/if}
    {#if showLyrics}
      <div class="lyrics">
        {#if !track}
          <p class="dim">Nada en reproducción.</p>
        {:else if lyrics === undefined}
          <p class="dim">Buscando la letra…</p>
        {:else if lyrics === null}
          <p class="dim">No hay letra para esta canción.</p>
        {:else}
          <button class="translate" class:on={translate} onclick={toggleTranslate}>
            {translate ? "Solo original" : "Traducir al español"}
          </button>
          {#if translate && translation === undefined}
            <p class="dim">Traduciendo…</p>
          {:else if translate && translationError}
            <p class="dim">{translationError}</p>
          {:else if translate && translation === lyrics}
            <p class="dim">La letra ya está en español.</p>
            <p>{lyrics}</p>
          {:else if translate && translation}
            {@const original = lyrics.split("\n")}
            {@const spanish = translation.split("\n")}
            <!-- Line by line so each verse sits next to its translation. -->
            <div class="dual">
              {#each Array(Math.max(original.length, spanish.length)) as _, i (i)}
                <span>{original[i] ?? ""}</span>
                <span class="es">{spanish[i] ?? ""}</span>
              {/each}
            </div>
          {:else}
            <p>{lyrics}</p>
          {/if}
        {/if}
      </div>
    {:else if skin.hero}
      <div class="hero" style:width="{HERO_SIZE}px" style:height="{HERO_SIZE}px">
        <skin.hero />
      </div>
    {:else}
      <div class="cover">
        {#if player.coverUrl}
          <img src={player.coverUrl} alt="" />
        {:else}
          <svg viewBox="0 0 24 24"><path d="M12 3v10.55A4 4 0 1 0 14 17V7h4V3z" /></svg>
        {/if}
      </div>
    {/if}
    <h2>{track?.title ?? "Nada en reproducción"}</h2>
    <p>{track ? `${track.artist} · ${track.album}` : "Haz doble clic en una canción"}</p>
    <button class="lyrics-toggle" class:on={showLyrics} onclick={() => (showLyrics = !showLyrics)}>
      {showLyrics ? "Portada" : "Letra"}
    </button>
  </aside>

  {#if player.notice}<div class="toast" role="status">{player.notice}</div>{/if}

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

    <button class="icon eq-btn" class:on={player.eq.enabled} onclick={() => openSettings("eq")} title="Ecualizador" aria-label="Ecualizador">
      <svg viewBox="0 0 24 24"><path d="M4 18h2v-6H4zm5 0h2V6H9zm5 0h2v-9h-2zm5 0h2v-4h-2z" /></svg>
    </button>
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

<Settings bind:open={settingsOpen} bind:section={settingsSection} {skinId} />

<style>
  :global(body) {
    margin: 0;
    font-family: "Inter", system-ui, sans-serif;
    background: var(--bg, #0f0f12);
    color: var(--fg, #e8e8ee);
    font-family: var(--font, "Inter", system-ui, sans-serif);
    overflow: hidden;
  }
  :global(button) {
    font: inherit;
    color: inherit;
  }

  .app {
    --accent-c: rgb(var(--accent));
    height: 100vh;
    display: grid;
    grid-template-columns: var(--side-w, 210px) minmax(0, 1fr) var(--aside-w, 340px);
    grid-template-rows: 1fr 88px;
  }

  .side {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 20px 12px;
    background: var(--sidebar);
    border-right: 1px solid var(--border);
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
    border-radius: var(--radius);
    padding: 10px 12px;
    text-align: left;
    color: var(--fg-dim);
    font-weight: 600;
    cursor: pointer;
  }
  .side button:hover {
    color: var(--fg);
    background: var(--surface-2);
  }
  .side button.on {
    color: var(--fg);
    background: rgba(var(--accent), 0.18);
  }
  .side svg {
    width: 20px;
  }

  .resizer {
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 6px;
    cursor: col-resize;
    z-index: 5;
    touch-action: none;
  }
  .resizer:hover,
  .resizer:active {
    background: rgba(var(--accent), 0.35);
  }
  .now {
    box-shadow: var(--sunken);
    position: relative;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 24px;
    text-align: center;
    background: var(--panel);
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
    border-radius: var(--radius-lg);
    overflow: hidden;
    background: var(--art-bg);
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
    fill: var(--fg-dim);
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

  .brand {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
  }
  .side .collapse {
    flex: none;
    width: 28px;
    height: 28px;
    padding: 0;
    justify-content: center;
    border-radius: var(--radius);
    color: var(--fg-dim);
  }
  .side .collapse svg {
    width: 18px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2.4;
  }
  .side.collapsed {
    padding-left: 8px;
    padding-right: 8px;
  }
  .side.collapsed .label {
    display: none;
  }
  .side.collapsed .brand {
    justify-content: center;
    padding: 4px 0 18px;
  }
  .side.collapsed button {
    justify-content: center;
    padding: 10px 0;
  }
  .side .gear {
    margin-top: auto;
  }
  .hero {
    position: relative;
    flex: none;
  }
  .eq-btn.on {
    color: rgb(var(--accent));
  }
  .lyrics {
    position: relative;
    width: 100%;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    text-align: center;
    mask-image: linear-gradient(transparent, #000 8%, #000 92%, transparent);
  }
  .lyrics p {
    white-space: pre-line;
    font-size: 15px;
    line-height: 1.7;
    margin: 24px 0;
  }
  .lyrics .dim {
    color: var(--fg-dim);
  }
  .translate {
    position: sticky;
    top: 0;
    z-index: 1;
    margin: 12px auto 0;
    display: block;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-pill);
    padding: 5px 14px;
    font-size: 12px;
    cursor: pointer;
  }
  .translate.on {
    background: rgba(var(--accent), 0.25);
  }
  .dual {
    display: grid;
    grid-template-columns: 1fr 1fr;
    column-gap: 18px;
    row-gap: 2px;
    margin: 20px 0;
    text-align: left;
    font-size: 14px;
    line-height: 1.6;
  }
  .dual span {
    min-height: 1.6em;
  }
  .dual .es {
    color: rgb(var(--accent));
  }
  .lyrics-toggle {
    position: relative;
    margin-top: 14px;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-pill);
    padding: 6px 16px;
    font-size: 13px;
    cursor: pointer;
  }
  .lyrics-toggle.on {
    background: var(--accent-c);
    color: var(--on-accent);
  }

  .bar {
    grid-column: 1 / -1;
    display: grid;
    grid-template-columns: auto 1fr auto 180px;
    align-items: center;
    gap: 28px;
    padding: 0 28px;
    background: var(--sidebar);
    border-top: 1px solid var(--border);
  }
  .buttons {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .icon {
    box-shadow: var(--raised);
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
    background: var(--surface-2);
  }
  svg {
    width: 22px;
    fill: currentColor;
  }
  .play {
    width: 52px;
    height: 52px;
    background: var(--accent-c);
    color: var(--on-accent);
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
  .toast {
    position: fixed;
    left: 50%;
    bottom: 100px;
    translate: -50% 0;
    max-width: 560px;
    padding: 10px 16px;
    border-radius: var(--radius);
    background: #3a1d22;
    color: #ffd7d7;
    border: 1px solid #6b2a33;
    font: 13px/1.4 system-ui, sans-serif;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
    z-index: 10;
  }
</style>
