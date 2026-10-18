<script lang="ts">
  import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
  import { connect, showView, player } from "$lib/player.svelte";
  import { savedSkin, nextSkin } from "$lib/skins";

  let skin = $state.raw(savedSkin());
  const win = getCurrentWindow();

  $effect(() => connect());
  $effect(() => {
    win.setSize(new LogicalSize(skin.width, skin.height));
  });
</script>

<div class="compact">
  <skin.component />
  {#if player.notice}<div class="toast">{player.notice}</div>{/if}
  <nav>
    <button title="Siguiente skin" onclick={() => (skin = nextSkin(skin))}>
      <svg viewBox="0 0 24 24"><path d="M12 3a9 9 0 1 0 9 9h-2a7 7 0 1 1-2.05-4.95L14 10h7V3l-2.64 2.64A8.96 8.96 0 0 0 12 3" /></svg>
    </button>
    <button title="Reproductor completo" onclick={() => showView("main")}>
      <svg viewBox="0 0 24 24"><path d="M4 4h7v2H6v5H4zm16 16h-7v-2h5v-5h2z" /></svg>
    </button>
    <button title="Ocultar en la bandeja" onclick={() => win.hide()}>
      <svg viewBox="0 0 24 24"><path d="M6 11h12v2H6z" /></svg>
    </button>
  </nav>
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
