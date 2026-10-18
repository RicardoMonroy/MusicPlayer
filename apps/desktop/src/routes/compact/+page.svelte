<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { connect, showView, player } from "$lib/player.svelte";
  import { savedSkin, skins, skinById, chooseSkin, DEFAULT_SKIN, PLAYLIST_HEIGHT } from "$lib/skins";
  import Playlist from "$lib/Playlist.svelte";
  import { compact } from "$lib/compact.svelte";

  let skin = $state.raw(skinById(DEFAULT_SKIN));
  savedSkin().then((s) => (skin = s));

  let picking = $state(false);
  const win = getCurrentWindow();

  $effect(() => {
    const off = listen<string>("skin", (e) => (skin = skinById(e.payload)));
    return () => {
      off.then((f) => f());
    };
  });

  $effect(() => connect());
  $effect(() => {
    localStorage.setItem("compact-playlist", compact.playlist ? "1" : "0");
    localStorage.setItem("compact-eq", compact.eq ? "1" : "0");
    invoke("resize_compact", {
      width: skin.width,
      height:
        skin.height +
        (compact.eq ? skin.equalizer.height : 0) +
        (compact.playlist ? PLAYLIST_HEIGHT : 0),
    });
  });
</script>

<div class="compact">
  <div class="skin" style:height="{skin.height}px">
    <skin.component />
  </div>
  {#if compact.eq}
    <div style:height="{skin.equalizer.height}px">
      <skin.equalizer.component />
    </div>
  {/if}
  {#if compact.playlist}
    <Playlist theme={skin.playlist} height={PLAYLIST_HEIGHT} />
  {/if}
  {#if player.notice}<div class="toast">{player.notice}</div>{/if}
  <nav>
    <button title="Ecualizador" class:on={compact.eq} onclick={() => (compact.eq = !compact.eq)}>
      <svg viewBox="0 0 24 24"><path d="M4 18h2v-6H4zm5 0h2V6H9zm5 0h2v-9h-2zm5 0h2v-4h-2z" /></svg>
    </button>
    <button title="Lista de reproducción" class:on={compact.playlist} onclick={() => (compact.playlist = !compact.playlist)}>
      <svg viewBox="0 0 24 24"><path d="M3 5h12v2H3zm0 4h12v2H3zm0 4h8v2H3zm14 0v6l5-3z" /></svg>
    </button>
    <button title="Elegir skin" class:on={picking} onclick={() => (picking = !picking)}>
      <svg viewBox="0 0 24 24"><path d="M12 3a9 9 0 1 0 9 9h-2a7 7 0 1 1-2.05-4.95L14 10h7V3l-2.64 2.64A8.96 8.96 0 0 0 12 3" /></svg>
    </button>
    <button title="Reproductor completo" onclick={() => showView("main")}>
      <svg viewBox="0 0 24 24"><path d="M4 4h7v2H6v5H4zm16 16h-7v-2h5v-5h2z" /></svg>
    </button>
    <button title="Ocultar en la bandeja" onclick={() => win.hide()}>
      <svg viewBox="0 0 24 24"><path d="M6 11h12v2H6z" /></svg>
    </button>
  </nav>
  {#if picking}
    <ul class="picker" role="menu">
      {#each skins as s (s.id)}
        <li>
          <button
            role="menuitemradio"
            aria-checked={s.id === skin.id}
            onclick={() => {
              chooseSkin(s.id);
              picking = false;
            }}
          >
            <i style:background={s.playlist.bg} style:color={s.playlist.current}>♪</i>
            {s.name}
            {#if s.id === skin.id}<b>✓</b>{/if}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  :global(html),
  :global(body) {
    margin: 0;
    background: transparent;
    overflow: hidden;
    user-select: none;
  }
  .compact {
    position: relative;
    width: 100vw;
    height: 100vh;
  }
  nav {
    position: absolute;
    top: 4px;
    right: 4px;
    display: flex;
    gap: 2px;
    opacity: 0;
    transition: opacity 0.2s;
  }
  .compact:hover nav {
    opacity: 1;
  }
  .skin {
    position: relative;
  }
  .picker {
    position: absolute;
    top: 32px;
    right: 4px;
    margin: 0;
    padding: 4px;
    list-style: none;
    background: rgba(20, 20, 24, 0.94);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 8px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
    font: 13px system-ui, sans-serif;
    z-index: 20;
  }
  .picker button {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 150px;
    padding: 6px 8px;
    border: 0;
    border-radius: 6px;
    background: none;
    color: #eee;
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  .picker button:hover {
    background: rgba(255, 255, 255, 0.1);
  }
  .picker i {
    width: 18px;
    height: 18px;
    border-radius: 4px;
    display: grid;
    place-items: center;
    font-style: normal;
    font-size: 11px;
  }
  .picker b {
    margin-left: auto;
  }
  nav button.on {
    background: rgba(255, 255, 255, 0.35);
  }
  nav button {
    width: 24px;
    height: 24px;
    padding: 4px;
    border: 0;
    border-radius: 6px;
    background: rgba(0, 0, 0, 0.55);
    color: #fff;
    cursor: pointer;
  }
  nav svg {
    width: 100%;
    fill: currentColor;
  }
  .toast {
    position: fixed;
    left: 50%;
    bottom: 8px;
    translate: -50% 0;
    max-width: calc(100% - 24px);
    padding: 10px 16px;
    border-radius: 10px;
    background: #3a1d22;
    color: #ffd7d7;
    border: 1px solid #6b2a33;
    font: 13px/1.4 system-ui, sans-serif;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
    z-index: 10;
  }
</style>
