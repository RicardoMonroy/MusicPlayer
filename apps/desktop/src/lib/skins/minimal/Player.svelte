<script lang="ts">
  import { player, togglePause, prev, next, seek, fmtTime } from "$lib/player.svelte";
  import { compact } from "$lib/compact.svelte";
  const playing = $derived(player.status.index !== null && !player.status.paused);
  const duration = $derived(Math.max(0, player.status.duration || 0));
  let pending = $state<number | null>(null);
  const position = $derived(pending ?? Math.max(0, Math.min(duration, player.status.position)));
  const progress = $derived(duration > 0 ? position / duration * 100 : 0);
</script>

<div class="player" data-tauri-drag-region><header data-tauri-drag-region>Minimalista</header><div class="info" data-tauri-drag-region><h1 title={player.track?.title} data-tauri-drag-region>{player.track?.title || "Nada en reproducción"}</h1><p data-tauri-drag-region>{player.track?.artist || "Elige una canción"}</p></div><div class="progress">
    <input aria-label="Posición de reproducción" type="range" min="0" max={duration || 1} step="0.1" value={position} disabled={duration === 0} style:--progress={`${progress}%`}
      oninput={(event) => pending = +event.currentTarget.value}
      onchange={(event) => { void seek(+event.currentTarget.value); pending = null; }} onpointercancel={() => pending = null} />
    <div class="times" data-tauri-drag-region><span data-tauri-drag-region>{fmtTime(position)}</span><span data-tauri-drag-region>{fmtTime(duration)}</span></div>
  </div><div class="transport">
    <button aria-label="Canción anterior" onclick={prev}><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 5h3v14H5zm4 7 10-7v14z"/></svg></button>
    <button class="play" aria-label={playing ? "Pausar" : "Reproducir"} onclick={togglePause}><svg viewBox="0 0 24 24" aria-hidden="true">{#if playing}<path d="M6 5h4v14H6zm8 0h4v14h-4z"/>{:else}<path d="M8 4v16l12-8z"/>{/if}</svg></button>
    <button aria-label="Canción siguiente" onclick={next}><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M16 5h3v14h-3zM5 5l10 7-10 7z"/></svg></button>
  </div><div class="toggles">
    <button class:on={compact.eq} aria-pressed={compact.eq} aria-label="Ecualizador" onclick={() => compact.eq = !compact.eq}>EQ</button>
    <button class:on={compact.playlist} aria-pressed={compact.playlist} aria-label="Lista de reproducción" onclick={() => compact.playlist = !compact.playlist}>PL</button>
  </div></div>
<style>

.player { position:relative; width:100%; height:100%; box-sizing:border-box; overflow:hidden; user-select:none; font:12px "Inter",system-ui,sans-serif; }
.player * { box-sizing:border-box; } header { height:24px; padding-right:132px; font-size:9px; letter-spacing:1.4px; text-transform:uppercase; color:var(--dim); }
.info { min-width:0; } h1,p { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; margin:0; } h1 { font-size:16px; line-height:22px; letter-spacing:-.4px; font-weight:650; } p { color:var(--dim); font-size:12px; line-height:20px; }
button { display:grid; place-items:center; border:0; background:transparent; color:inherit; cursor:pointer; font:inherit; padding:0; } button:hover { background:var(--hover); } button:focus-visible,input:focus-visible { outline:2px solid rgb(var(--local-accent)); outline-offset:2px; }
.transport { display:flex; align-items:center; gap:9px; } .transport button { width:30px; height:30px; border-radius:50%; } .transport svg { width:19px; height:19px; fill:currentColor; } .transport .play { width:36px; height:36px; background:rgb(var(--local-accent)); color:var(--on-accent-local); }
.toggles { display:flex; gap:4px; } .toggles button { height:24px; width:27px; font-size:9px; border-radius:4px; color:var(--dim); } .toggles .on { background:var(--hover); color:var(--fg-local); box-shadow:inset 0 -2px rgb(var(--local-accent)); }
.progress { min-width:0; } input { display:block; appearance:none; width:100%; margin:0; height:14px; background:transparent; cursor:pointer; } input::-webkit-slider-runnable-track { height:3px; border-radius:5px; background:linear-gradient(90deg,rgb(var(--local-accent)) var(--progress),var(--hover) var(--progress)); } input::-moz-range-track { height:3px; background:var(--hover); } input::-moz-range-progress { height:3px; background:rgb(var(--local-accent)); } input::-webkit-slider-thumb { appearance:none; height:9px; width:9px; margin-top:-3px; border-radius:50%; background:rgb(var(--local-accent)); } input::-moz-range-thumb { width:9px; height:9px; border:0; border-radius:50%; background:rgb(var(--local-accent)); } input:disabled { cursor:default; opacity:.5; }
.times { display:flex; justify-content:space-between; color:var(--dim); font-size:9px; font-variant-numeric:tabular-nums; }

.player { --local-accent:71,99,81; --dim:#595e59; --fg-local:#202421; --hover:#dcded5; --on-accent-local:#fff; padding:8px 18px; color:#202421; background:#f7f6f2; border:1px solid #d3d2ca; border-radius:4px; font-family:"Helvetica Neue","Inter",system-ui,sans-serif; }
header { height:22px; font-size:8px; letter-spacing:1.7px; } .info { display:flex; align-items:baseline; gap:10px; } h1 { font-size:19px; line-height:24px; font-weight:500; flex:1; } p { max-width:94px; font-size:10px; } .progress { margin-top:3px; } .times { font-size:8px; } .transport { position:absolute; left:12px; bottom:4px; gap:1px; } .transport button { height:25px; width:30px; border-radius:2px; } .transport .play { height:25px; width:30px; background:transparent; color:#202421; } .transport .play:hover { background:var(--hover); } .transport svg { width:16px; height:16px; } .toggles { position:absolute; right:13px; bottom:5px; } input::-webkit-slider-runnable-track { height:1px; } input::-webkit-slider-thumb { width:5px; height:5px; margin-top:-2px; } input::-moz-range-track,input::-moz-range-progress { height:1px; } input::-moz-range-thumb { width:5px; height:5px; }
</style>
