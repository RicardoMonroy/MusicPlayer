<script lang="ts">
  import { player, togglePause, next, prev } from "$lib/player.svelte";

  const playing = $derived(player.status.index != null && !player.status.paused);
  const progress = $derived(
    player.status.duration > 0 ? player.status.position / player.status.duration : 0,
  );
  // Tape moves from the left reel to the right one as the track plays.
  const leftTape = $derived(78 - 40 * progress);
  const rightTape = $derived(38 + 40 * progress);
</script>

<div class="deck" data-tauri-drag-region>
  <div class="shell" data-tauri-drag-region>
    <div class="label" style="--accent: {player.accent}" data-tauri-drag-region>
      <div class="lines" data-tauri-drag-region>
        <span class="title">{player.track?.title ?? "Inserta un casete"}</span>
        <span class="artist">{player.track?.artist ?? "—"}</span>
      </div>
      <div class="window" data-tauri-drag-region>
        {#each [leftTape, rightTape] as tape}
          <div class="reel-slot">
            <div class="tape" style="width: {tape}px; height: {tape}px"></div>
            <div class="reel" class:spin={playing}>
              {#each [0, 60, 120] as a}
                <i style="transform: rotate({a}deg)"></i>
              {/each}
            </div>
          </div>
        {/each}
      </div>
      <span class="side">A</span>
    </div>
    <div class="bottom" data-tauri-drag-region>
      {#each [0, 1, 2, 3] as _}<b></b>{/each}
    </div>
  </div>

  <div class="keys">
    <button onclick={prev} aria-label="Anterior">◀◀</button>
    <button class:down={playing} onclick={togglePause} aria-label="Reproducir/Pausar">
      {playing ? "❚❚" : "▶"}
    </button>
    <button onclick={next} aria-label="Siguiente">▶▶</button>
  </div>
</div>

<style>
  .deck {
    width: 420px;
    height: 300px;
    box-sizing: border-box;
    padding: 14px 16px 0;
    border-radius: 18px;
    background: linear-gradient(160deg, #2b2b30, #18181c);
    font-family: "Courier New", ui-monospace, monospace;
  }
  .shell {
    position: relative;
    height: 220px;
    border-radius: 12px;
    background: linear-gradient(180deg, #3a3a40, #26262b);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.12),
      0 6px 16px rgba(0, 0, 0, 0.5);
    padding: 12px 16px;
    box-sizing: border-box;
  }
  .label {
    position: relative;
    height: 150px;
    border-radius: 6px;
    background:
      linear-gradient(rgb(var(--accent)) 0 0) 0 30px / 100% 6px no-repeat,
      #efe8d8;
    color: #2a2622;
    padding: 6px 12px;
    box-sizing: border-box;
  }
  .lines {
    display: flex;
    gap: 10px;
    align-items: baseline;
    height: 24px;
    overflow: hidden;
    white-space: nowrap;
  }
  .title {
    font-weight: 700;
    font-size: 15px;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .artist {
    font-size: 12px;
    opacity: 0.7;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .side {
    position: absolute;
    left: 12px;
    bottom: 8px;
    font-weight: 700;
    font-size: 22px;
  }
  .window {
    margin: 18px auto 0;
    width: 250px;
    height: 84px;
    border-radius: 42px;
    background: #15130f;
    box-shadow: inset 0 2px 6px #000;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 6px;
    box-sizing: border-box;
  }
  .reel-slot {
    position: relative;
    width: 78px;
    height: 78px;
    display: grid;
    place-items: center;
  }
  .tape {
    position: absolute;
    border-radius: 50%;
    background: radial-gradient(#4a3426 55%, #2c1d14);
    transition:
      width 0.3s,
      height 0.3s;
  }
  .reel {
    position: relative;
    width: 34px;
    height: 34px;
    border-radius: 50%;
    background: #ece6d6;
    box-shadow: 0 0 0 3px #bdb6a4;
  }
  .reel i {
    position: absolute;
    left: 15px;
    top: 2px;
    width: 4px;
    height: 30px;
    background: #15130f;
    border-radius: 2px;
  }
  .spin {
    animation: spin 2s linear infinite;
  }
  @keyframes spin {
    to {
      rotate: -360deg;
    }
  }
  .bottom {
    display: flex;
    justify-content: center;
    gap: 40px;
    margin-top: 12px;
  }
  .bottom b {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: #111;
    box-shadow: inset 0 1px 2px #000;
  }
  .keys {
    display: flex;
    justify-content: center;
    gap: 8px;
    margin-top: 12px;
  }
  .keys button {
    width: 80px;
    height: 36px;
    border: 0;
    border-radius: 4px 4px 8px 8px;
    background: linear-gradient(#d9d9dc, #a9a9ae);
    box-shadow:
      0 5px 0 #6e6e74,
      0 6px 8px rgba(0, 0, 0, 0.6);
    color: #2a2a2e;
    font-size: 14px;
    cursor: pointer;
    transition:
      transform 0.08s,
      box-shadow 0.08s;
  }
  .keys button:active,
  .keys .down {
    transform: translateY(4px);
    box-shadow:
      0 1px 0 #6e6e74,
      0 2px 4px rgba(0, 0, 0, 0.6);
  }
</style>
