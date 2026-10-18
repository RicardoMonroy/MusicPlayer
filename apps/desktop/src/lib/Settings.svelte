<script lang="ts" module>
  export type Section = "apariencia" | "audio" | "eq";
</script>

<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { skins, skinById, chooseSkin } from "$lib/skins";

  let {
    open = $bindable(false),
    section = $bindable<Section>("apariencia"),
    skinId,
  }: { open: boolean; section: Section; skinId: string } = $props();

  let replaygain = $state("track");
  invoke<string>("replaygain").then((m) => (replaygain = m));

  const skin = $derived(skinById(skinId));
  const sections: [Section, string][] = [
    ["apariencia", "Apariencia"],
    ["audio", "Audio"],
    ["eq", "Ecualizador"],
  ];

  function onKey(e: KeyboardEvent) {
    if (open && e.key === "Escape") open = false;
  }
</script>

<svelte:window onkeydown={onKey} />

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="overlay" onclick={(e) => e.target === e.currentTarget && (open = false)}>
    <div class="dialog" role="dialog" aria-modal="true" aria-label="Ajustes">
      <nav>
        <h2>Ajustes</h2>
        {#each sections as [id, label] (id)}
          <button class:on={section === id} onclick={() => (section = id)}>{label}</button>
        {/each}
      </nav>

      <section>
        <button class="close" onclick={() => (open = false)} aria-label="Cerrar">✕</button>

        {#if section === "apariencia"}
          <h3>Skin</h3>
          <p class="hint">Cambia el mini reproductor, el reproductor completo y la TUI.</p>
          <div class="skins">
            {#each skins as s (s.id)}
              {@const t = s.theme}
              <button
                class="skin"
                class:on={s.id === skinId}
                style:background={t.bg}
                style:color={t.fg}
                style:font-family={t.font}
                style:border-radius={t.radiusLg}
                onclick={() => chooseSkin(s.id)}
              >
                <span class="swatches">
                  <i style:background={t.panel}></i>
                  <i style:background={t.surface2}></i>
                  <i style:background={t.accent ? `rgb(${t.accent})` : "rgb(255, 110, 150)"}></i>
                </span>
                <strong>{s.name}</strong>
                <small style:color={t.dim}>{t.scheme === "light" ? "Claro" : "Oscuro"}</small>
              </button>
            {/each}
          </div>
        {:else if section === "audio"}
          <h3>Volumen normalizado</h3>
          <p class="hint">
            ReplayGain iguala el volumen entre canciones que traen esas etiquetas (la mayoría de los
            FLAC y MP3 bien etiquetados).
          </p>
          <select
            bind:value={replaygain}
            onchange={() => invoke("set_replaygain", { mode: replaygain })}
          >
            <option value="no">Desactivado</option>
            <option value="track">Por canción</option>
            <option value="album">Por álbum (respeta la dinámica del disco)</option>
          </select>
        {:else}
          <h3>Ecualizador</h3>
          <p class="hint">Con el estilo de la skin {skin.name}. Se guarda y lo usa también la TUI.</p>
          <div class="eq" style:height="{skin.equalizer.height}px">
            <skin.equalizer.component />
          </div>
        {/if}
      </section>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.62);
  }
  .dialog {
    display: grid;
    grid-template-columns: 180px minmax(0, 1fr);
    width: min(860px, calc(100vw - 48px));
    height: min(600px, calc(100vh - 48px));
    background: var(--bg);
    color: var(--fg);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: 0 30px 80px rgba(0, 0, 0, 0.6), var(--raised);
    overflow: hidden;
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 20px 12px;
    background: var(--sidebar);
    border-right: 1px solid var(--border);
  }
  nav h2 {
    margin: 0 0 14px 10px;
    font-size: 18px;
  }
  nav button {
    text-align: left;
    padding: 9px 12px;
    border: 0;
    border-radius: var(--radius);
    background: none;
    color: var(--fg-dim);
    font-weight: 600;
    cursor: pointer;
  }
  nav button:hover,
  nav button.on {
    color: var(--fg);
    background: var(--surface-2);
  }
  section {
    position: relative;
    padding: 24px 28px;
    overflow-y: auto;
  }
  .close {
    position: absolute;
    top: 14px;
    right: 14px;
    width: 32px;
    height: 32px;
    border: 0;
    border-radius: var(--radius);
    background: none;
    color: var(--fg-dim);
    cursor: pointer;
  }
  .close:hover {
    background: var(--surface-2);
    color: var(--fg);
  }
  h3 {
    margin: 0;
    font-size: 20px;
  }
  .hint {
    margin: 6px 0 18px;
    color: var(--fg-dim);
    font-size: 13px;
    line-height: 1.5;
  }
  .skins {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 14px;
  }
  .skin {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
    padding: 14px;
    border: 2px solid transparent;
    box-shadow: 0 4px 14px rgba(0, 0, 0, 0.35);
    cursor: pointer;
    text-align: left;
  }
  .skin.on {
    border-color: rgb(var(--accent));
  }
  .swatches {
    display: flex;
    gap: 5px;
    margin-bottom: 6px;
  }
  .swatches i {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    box-shadow: inset 0 0 0 1px rgba(127, 127, 127, 0.4);
  }
  .skin strong {
    font-size: 15px;
  }
  .skin small {
    font-size: 11px;
  }
  select {
    background: var(--surface);
    color: inherit;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: var(--sunken);
    padding: 8px 10px;
    font: inherit;
    min-width: 320px;
  }
  .eq {
    max-width: 640px;
    border-radius: var(--radius);
    overflow: hidden;
  }
</style>
