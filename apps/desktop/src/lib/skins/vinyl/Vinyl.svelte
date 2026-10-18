<script lang="ts">
  import Spectrum from "./Spectrum.svelte";
  import { player, togglePause, next, prev } from "$lib/player.svelte";

  const playing = $derived(player.status.index != null && !player.status.paused);
  const progress = $derived(
    player.status.duration > 0 ? player.status.position / player.status.duration : 0,
  );
  // Arm rests off the record when stopped, then sweeps inward as the track plays.
  const armAngle = $derived(player.track ? 12 + 12 * progress : 0);
</script>

<div class="plinth" style="--accent: {player.accent}" data-tauri-drag-region>
  <div class="platter" data-tauri-drag-region>
    <div class="record" class:spin={playing}>
      <div class="label">
        {#if player.coverUrl}
          <img src={player.coverUrl} alt="" />
        {/if}
      </div>
    </div>
  </div>

  <Spectrum />

  <div class="arm" style="rotate: {armAngle}deg">
    <div class="pivot"></div>
    <div class="rod"></div>
    <div class="head"></div>
  </div>

  <div class="info">
    <div class="text">
      <span class="title">{player.track?.title ?? "Baja la aguja"}</span>
      <span class="artist">{player.track?.artist ?? ""}</span>
    </div>
    <div class="buttons">
      <button onclick={prev} aria-label="Anterior">
        <svg viewBox="0 0 24 24"><path d="M6 6h2v12H6zm3.5 6 8.5 6V6z" /></svg>
      </button>
      <button class="play" onclick={togglePause} aria-label="Reproducir/Pausar">
        {#if playing}
          <svg viewBox="0 0 24 24"><path d="M6 5h4v14H6zm8 0h4v14h-4z" /></svg>
        {:else}
          <svg viewBox="0 0 24 24"><path d="M8 5v14l11-7z" /></svg>
        {/if}
      </button>
      <button onclick={next} aria-label="Siguiente">
        <svg viewBox="0 0 24 24"><path d="M16 6h2v12h-2zM6 18l8.5-6L6 6z" /></svg>
      </button>
    </div>
  </div>
</div>

<style>
  .plinth {
    position: relative;
    width: 380px;
    height: 440px;
    box-sizing: border-box;
    border-radius: 20px;
    background:
      repeating-linear-gradient(92deg, rgba(0, 0, 0, 0.06) 0 3px, transparent 3px 9px),
      linear-gradient(160deg, #6b4a32, #3e2a1c);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.15);
    font-family: "Inter", system-ui, sans-serif;
    color: #f3ece4;
  }
  .platter {
    position: absolute;
    left: 24px;
    top: 28px;
    width: 300px;
    height: 300px;
    border-radius: 50%;
    background: #1a1a1a;
    box-shadow: 0 10px 24px rgba(0, 0, 0, 0.6);
    display: grid;
    place-items: center;
  }
  .record {
    width: 290px;
    height: 290px;
    border-radius: 50%;
    background:
      conic-gradient(
        from 30deg,
        transparent 0 40deg,
        rgba(255, 255, 255, 0.09) 50deg,
        transparent 60deg 220deg,
        rgba(255, 255, 255, 0.07) 230deg,
        transparent 240deg
      ),
      repeating-radial-gradient(#111 0 1px, #161616 1px 3px);
    display: grid;
    place-items: center;
  }
  .label {
    width: 100px;
    height: 100px;
    border-radius: 50%;
    overflow: hidden;
    background: rgb(var(--accent));
    box-shadow: 0 0 0 2px #0a0a0a;
    position: relative;
  }
  .label::after {
    content: "";
    position: absolute;
    inset: 46px;
    border-radius: 50%;
    background: #ddd;
  }
  .label img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .spin {
    animation: spin 1.8s linear infinite; /* 33⅓ rpm */
  }
  @keyframes spin {
    to {
      rotate: 360deg;
    }
  }
  .arm {
    z-index: 1;
    position: absolute;
    left: 322px;
    top: 34px;
    width: 0;
    height: 0;
    transition: rotate 0.8s ease;
  }
  .pivot {
    position: absolute;
    left: -16px;
    top: -16px;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: radial-gradient(#d8d8d8, #8a8a8a);
    box-shadow: 0 3px 6px rgba(0, 0, 0, 0.5);
  }
  .rod {
    position: absolute;
    left: -3px;
    top: 0;
    width: 6px;
    height: 230px;
    border-radius: 3px;
    background: linear-gradient(90deg, #9a9a9a, #eee, #9a9a9a);
    transform-origin: top center;
    rotate: 8deg;
  }
  .head {
    position: absolute;
    left: -42px;
    top: 220px;
    width: 22px;
    height: 34px;
    border-radius: 3px;
    background: #2a2a2a;
    rotate: 8deg;
  }
  .info {
    position: absolute;
    left: 20px;
    right: 20px;
    bottom: 18px;
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .title,
  .artist {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .title {
    font-weight: 700;
    font-size: 15px;
  }
  .artist {
    font-size: 12px;
    opacity: 0.75;
  }
  .buttons {
    display: flex;
    gap: 4px;
  }
  button {
    width: 36px;
    height: 36px;
    border: 0;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.3);
    color: inherit;
    display: grid;
    place-items: center;
    cursor: pointer;
  }
  .play {
    width: 44px;
    height: 44px;
    background: #f3ece4;
    color: #3e2a1c;
  }
  svg {
    width: 20px;
    fill: currentColor;
  }
  @media (prefers-reduced-motion: reduce) {
    .spin { animation: none; }
    .arm { transition: none; }
  }
</style>
