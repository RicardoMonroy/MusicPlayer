<script lang="ts">
  import { player, togglePause, prev, next, seek, fmtTime } from "$lib/player.svelte";
  import { compact } from "$lib/compact.svelte";
  const playing = $derived(player.status.index !== null && !player.status.paused);
  const duration = $derived(Math.max(0, player.status.duration || 0));
  let pending = $state<number | null>(null);
  const position = $derived(pending ?? Math.max(0, Math.min(duration, player.status.position)));
  const progress = $derived(duration > 0 ? position / duration * 100 : 0);
</script>

<div class="player" style:--local-accent={player.accent} data-tauri-drag-region>
  {#if player.coverUrl}<div class="backdrop" style:background-image={`url("${player.coverUrl}")`}></div>{/if}
  <div class="content" data-tauri-drag-region><header data-tauri-drag-region>MusicPlayer</header><div class="cover" data-tauri-drag-region>{#if player.coverUrl}<img src={player.coverUrl} alt="Portada del álbum" draggable="false" data-tauri-drag-region />{:else}<svg viewBox="0 0 100 100" aria-hidden="true" data-tauri-drag-region><circle cx="50" cy="50" r="32"/><circle cx="50" cy="50" r="12"/><path d="M50 8v12M50 80v12M8 50h12M80 50h12"/></svg>{/if}</div><div class="info" data-tauri-drag-region><h1 title={player.track?.title} data-tauri-drag-region>{player.track?.title || "Nada en reproducción"}</h1><p data-tauri-drag-region>{player.track?.artist || "Elige una canción"}</p></div><div class="progress">
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
  </div><div class="spectrum" aria-hidden="true">{#each Array(19) as _, index}<i style:height={`${2 + Math.max(0, Math.min(1, playing ? player.spectrum[index] ?? 0 : 0)) * 14}px`}></i>{/each}</div></div></div>
<style>

.player { position:relative; width:100%; height:100%; box-sizing:border-box; overflow:hidden; user-select:none; font:12px "Inter",system-ui,sans-serif; }
.player * { box-sizing:border-box; } header { height:24px; padding-right:132px; font-size:9px; letter-spacing:1.4px; text-transform:uppercase; color:var(--dim); }
.info { min-width:0; } h1,p { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; margin:0; } h1 { font-size:16px; line-height:22px; letter-spacing:-.4px; font-weight:650; } p { color:var(--dim); font-size:12px; line-height:20px; }
.cover { display:grid; place-items:center; overflow:hidden; background:var(--art); } .cover img { width:100%; height:100%; object-fit:cover; } .cover > svg { width:70%; height:70%; fill:none; stroke:rgb(var(--local-accent)); stroke-width:1.5; }
button { display:grid; place-items:center; border:0; background:transparent; color:inherit; cursor:pointer; font:inherit; padding:0; } button:hover { background:var(--hover); } button:focus-visible,input:focus-visible { outline:2px solid rgb(var(--local-accent)); outline-offset:2px; }
.transport { display:flex; align-items:center; gap:9px; } .transport button { width:30px; height:30px; border-radius:50%; } .transport svg { width:19px; height:19px; fill:currentColor; } .transport .play { width:36px; height:36px; background:rgb(var(--local-accent)); color:var(--on-accent-local); }
.toggles { display:flex; gap:4px; } .toggles button { height:24px; width:27px; font-size:9px; border-radius:4px; color:var(--dim); } .toggles .on { background:var(--hover); color:var(--fg-local); box-shadow:inset 0 -2px rgb(var(--local-accent)); }
.progress { min-width:0; } input { display:block; appearance:none; width:100%; margin:0; height:14px; background:transparent; cursor:pointer; } input::-webkit-slider-runnable-track { height:3px; border-radius:5px; background:linear-gradient(90deg,rgb(var(--local-accent)) var(--progress),var(--hover) var(--progress)); } input::-moz-range-track { height:3px; background:var(--hover); } input::-moz-range-progress { height:3px; background:rgb(var(--local-accent)); } input::-webkit-slider-thumb { appearance:none; height:9px; width:9px; margin-top:-3px; border-radius:50%; background:rgb(var(--local-accent)); } input::-moz-range-thumb { width:9px; height:9px; border:0; border-radius:50%; background:rgb(var(--local-accent)); } input:disabled { cursor:default; opacity:.5; }
.times { display:flex; justify-content:space-between; color:var(--dim); font-size:9px; font-variant-numeric:tabular-nums; }

.player { --dim:#b4b4c2; --fg-local:#e8e8ee; --art:#23232b; --hover:#ffffff22; --on-accent-local:#111; color:#e8e8ee; border:1px solid #ffffff18; border-radius:14px; background:#15151a; }
.backdrop { position:absolute; inset:-30px; background-size:cover; background-position:center; filter:blur(25px); opacity:.2; pointer-events:none; }
.content { position:relative; height:100%; padding:8px 14px; } .cover { position:absolute; left:14px; top:34px; width:78px; height:78px; border-radius:9px; box-shadow:0 5px 14px #0006; } .info { position:absolute; left:106px; right:14px; top:33px; } .progress { position:absolute; left:106px; right:14px; top:76px; } .transport { position:absolute; left:102px; bottom:7px; } .toggles { position:absolute; right:12px; bottom:12px; } .spectrum { position:absolute; display:flex; gap:2px; align-items:end; left:15px; bottom:13px; height:17px; width:76px; } .spectrum i { flex:1; background:rgb(var(--local-accent)); opacity:.65; border-radius:1px; }
@media (prefers-reduced-motion:reduce) { .spectrum { visibility:hidden; } }
</style>
