<script lang="ts">
  import { onMount } from "svelte";
  import { player, fmtTime, togglePause, prev, next, seek, setVolume, stop, setShuffle, cycleRepeat } from "$lib/player.svelte";

  import { compact } from "$lib/compact.svelte";

  const playing = $derived(player.status.index !== null && !player.status.paused);
  const title = $derived(player.track
    ? [player.track.artist, player.track.title].filter(Boolean).join(" - ") || "Canción sin título"
    : "Listo para la música");
  let marquee: HTMLDivElement;
  let titleText: HTMLSpanElement;
  let overflow = $state(0);
  let levels = $state(Array<number>(19).fill(0));
  let dragging = $state(false);
  let localPosition = $state(0);
  const duration = $derived(Math.max(0, player.status.duration || 0));
  const position = $derived(dragging ? localPosition : Math.min(duration, Math.max(0, player.status.position)));

  $effect(() => {
    title;
    if (marquee && titleText) overflow = Math.max(0, titleText.scrollWidth - marquee.clientWidth);
  });

  onMount(() => {
    const observer = new ResizeObserver(() => {
      overflow = Math.max(0, titleText.scrollWidth - marquee.clientWidth);
    });
    observer.observe(marquee);
    observer.observe(titleText);
    const motion = window.matchMedia("(prefers-reduced-motion: reduce)");
    let frame = 0;
    let previous = 0;
    // Real bands from mpv (player.spectrum); fast attack, slower fall like a hardware analyser.
    function animate(time: number) {
      const delta = Math.min((time - previous) / 1000 || 0, 0.05);
      previous = time;
      levels = levels.map((level, i) => {
        const target = playing && !motion.matches ? (player.spectrum[i] ?? 0) : 0;
        const rate = target > level ? 25 : 5;
        return level + (target - level) * Math.min(1, delta * rate);
      });
      frame = requestAnimationFrame(animate);
    }
    frame = requestAnimationFrame(animate);
    return () => { cancelAnimationFrame(frame); observer.disconnect(); };
  });

  function commitSeek() {
    void seek(localPosition);
    dragging = false;
  }
</script>

<div class="classic">
  <div class="titlebar" data-tauri-drag-region>
    <span class="title-lines" data-tauri-drag-region></span>
    <span data-tauri-drag-region>CLÁSICO · REPRODUCTOR ESTÉREO</span>
    <span class="title-lines" data-tauri-drag-region></span>
  </div>

  <div class="lcd">
    <div class="meter">
      <div class="elapsed"><span class="state-icon">{playing ? "▶" : "Ⅱ"}</span>{fmtTime(position)}</div>
      <div class="spectrum" aria-hidden="true">
        {#each levels as level, i}
          <div class="bar-track"><div class="bar" style:height={`${level * 100}%`}></div></div>
        {/each}
      </div>
      <div class="meter-label"><span>∞ ESPECTRO</span><span>ESTÉREO</span></div>
    </div>
    <div class="track-panel">
      <div class="track-heading"><span>SONANDO</span><span class:lit={playing}>● SEÑAL</span></div>
      <div class="marquee" bind:this={marquee} style:--travel={`${-overflow}px`} style:--marquee-duration={`${Math.max(8, overflow / 24 + 4)}s`}>
        <span bind:this={titleText} class:scrolling={overflow > 0}>{title}</span>
      </div>
      <div class="album" title={player.track?.album || "Sin álbum"}><b>ÁLBUM</b> {player.track?.album || "—"}</div>
      <div class="format"><span>PCM <b>•••</b> AUDIO</span><span>ESTÉREO DIGITAL</span></div>
    </div>
  </div>

  <div class="seek-row">
    <label for="classic-seek">POSICIÓN</label>
    <input id="classic-seek" aria-label="Posición de reproducción" type="range" min="0" max={duration || 1} step="0.1" value={position} disabled={duration === 0}
      oninput={(event) => { dragging = true; localPosition = +event.currentTarget.value; }} onchange={commitSeek} />
    <span>{fmtTime(duration)}</span>
  </div>

  <div class="controls">
    <div class="transport">
      <button aria-label="Canción anterior" title="Canción anterior" onclick={() => prev()}><svg viewBox="0 0 24 18" aria-hidden="true"><path d="M3 2h3v14H3zM20 2v14L7 9z" /></svg></button>
      <button class="play" aria-label={playing ? "Pausar" : "Reproducir"} title={playing ? "Pausar" : "Reproducir"} onclick={() => togglePause()}><svg viewBox="0 0 24 18" aria-hidden="true">{#if playing}<path d="M5 2h5v14H5zM14 2h5v14h-5z" />{:else}<path d="M6 1v16L21 9z" />{/if}</svg></button>
      <button aria-label="Canción siguiente" title="Canción siguiente" onclick={() => next()}><svg viewBox="0 0 24 18" aria-hidden="true"><path d="M18 2h3v14h-3zM4 2v14l13-7z" /></svg></button>
      <button aria-label="Detener" title="Detener" onclick={() => stop()}><svg viewBox="0 0 24 18" aria-hidden="true"><path d="M5 2h14v14H5z" /></svg></button>
    </div>
    <div class="volume">
      <label for="classic-volume">VOL</label>
      <input id="classic-volume" aria-label="Volumen" type="range" min="0" max="100" step="1" value={player.status.volume} oninput={(event) => setVolume(+event.currentTarget.value)} />
      <span>{Math.round(player.status.volume)}%</span>
    </div>
    <div class="switches">
      <button class:on={player.status.shuffle} aria-pressed={player.status.shuffle} title="Reproducción aleatoria" onclick={() => setShuffle(!player.status.shuffle)}>ALEAT.</button>
      <button class:on={player.status.repeat !== "off"} aria-label={`Repetir: ${player.status.repeat === "off" ? "desactivado" : player.status.repeat === "one" ? "una canción" : "todo"}`} title="Cambiar repetición" onclick={() => cycleRepeat()}>REP.{#if player.status.repeat === "one"}<b class="repeat-one">1</b>{/if}</button>
      <button class:on={compact.eq} aria-pressed={compact.eq} aria-label="Ecualizador" onclick={() => compact.eq = !compact.eq}>EQ</button>
      <button class:on={compact.playlist} aria-pressed={compact.playlist} aria-label="Lista de reproducción" onclick={() => compact.playlist = !compact.playlist}>PL</button>
    </div>
  </div>
</div>

<style>
  .classic { width:550px; height:232px; box-sizing:border-box; overflow:hidden; padding:0 14px 12px; border-radius:4px; border:2px solid; border-color:#92949b #1a1b20 #121317 #747780; background:linear-gradient(110deg,#41434c,#30323b 55%,#454751); box-shadow:inset 0 0 0 2px #25262e; color:#c5c6ce; font:10px monospace; user-select:none; }
  .classic * { box-sizing:border-box; }
  .titlebar { height:29px; padding-right:132px; display:flex; align-items:center; gap:9px; color:#d2d2d9; font-size:10px; letter-spacing:1.3px; white-space:nowrap; }
  .title-lines { flex:1; height:8px; background:repeating-linear-gradient(to bottom,#181a23 0 1px,#72747e 1px 2px,transparent 2px 3px); }
  .lcd { display:grid; grid-template-columns:190px minmax(0,1fr); gap:15px; height:112px; padding:9px 10px; border:2px solid; border-color:#17191f #777a83 #777a83 #17191f; background:#070e0c; box-shadow:inset 0 0 10px #000; color:#77ee74; }
  .meter { min-width:0; }
  .elapsed { height:40px; font:38px/40px monospace; letter-spacing:-2px; color:#98ff83; text-shadow:0 0 5px #77ec623d; font-variant-numeric:tabular-nums; white-space:nowrap; }
  .state-icon { font-size:16px; vertical-align:middle; margin-right:12px; letter-spacing:0; }
  .spectrum { display:flex; align-items:end; gap:3px; height:38px; padding-top:3px; }
  .bar-track { height:100%; flex:1; display:flex; align-items:end; background:#0d1c13; }
  .bar { width:100%; background:linear-gradient(to top,#4ee450 0%,#95f34e 55%,#e4df51 78%,#ed7252 100%); background-size:100% 35px; background-position:bottom; mask-image:repeating-linear-gradient(to top,#000 0 3px,transparent 3px 4px); }
  .meter-label { display:flex; justify-content:space-between; margin-top:4px; font-size:7px; color:#498957; letter-spacing:1px; }
  .track-panel { min-width:0; padding-top:3px; }
  .track-heading { display:flex; justify-content:space-between; font-size:8px; letter-spacing:1px; color:#42764e; }
  .track-heading .lit { color:#95fa72; }
  .marquee { overflow:hidden; margin:9px 0 10px; font:17px/23px monospace; white-space:nowrap; text-shadow:0 0 4px #66de6055; }
  .marquee span { display:inline-block; }
  .scrolling { animation:marquee var(--marquee-duration) linear infinite alternate; }
  @keyframes marquee { 0%,15% { transform:translateX(0); } 85%,100% { transform:translateX(var(--travel)); } }
  .album { overflow:hidden; white-space:nowrap; text-overflow:ellipsis; font-size:10px; color:#83be71; }
  .album b { font-size:8px; color:#42764e; margin-right:8px; }
  .format { display:flex; justify-content:space-between; margin-top:10px; font-size:8px; color:#42764e; }
  .format b { color:#9cce66; }
  .seek-row { height:32px; display:flex; align-items:center; gap:9px; }
  .seek-row label { font-size:8px; letter-spacing:1px; }
  .seek-row span { width:35px; text-align:right; font-variant-numeric:tabular-nums; }
  input[type=range] { appearance:none; min-width:0; height:9px; border:1px solid; border-color:#12141b #737680 #737680 #12141b; background:#191e21; cursor:pointer; margin:0; }
  .seek-row input { flex:1; }
  input[type=range]::-webkit-slider-thumb { appearance:none; width:22px; height:13px; border:2px solid; border-color:#dadbc6 #464b3b #464b3b #dadbc6; background:linear-gradient(90deg,#828a61,#c5c798,#828a61); }
  input[type=range]::-moz-range-thumb { width:18px; height:9px; border:2px outset #c5c798; border-radius:0; background:#a3ad78; }
  input:disabled { opacity:.4; cursor:default; }
  .controls { display:flex; align-items:center; gap:9px; }
  .transport { display:flex; gap:4px; }
  button { display:grid; place-items:center; width:34px; height:32px; padding:5px 5px; border:2px solid; border-color:#afb1bb #181b23 #181b23 #afb1bb; border-radius:2px; background:linear-gradient(#8b8e9b,#595e6c); box-shadow:0 1px 1px #111; cursor:pointer; }
  button:active { border-color:#181b23 #afb1bb #afb1bb #181b23; background:#626674; }
  button svg { width:24px; height:18px; fill:#18212a; filter:drop-shadow(0 1px #b9bdc6); }
  button.play { width:38px; }
  button:focus-visible, input:focus-visible { outline:2px solid #b7f78f; outline-offset:2px; }
  .volume { display:flex; align-items:center; gap:5px; flex:1; min-width:0; }
  .volume label { font-size:9px; }
  .volume input { flex:1; }
  .volume span { width:32px; font-size:9px; text-align:right; }
  .switches { display:grid; grid-template-columns:51px 44px; gap:3px; }
  .switches button { width:100%; height:17px; padding:0 3px; display:flex; gap:3px; align-items:center; font:8px monospace; color:#18212a; }
  .switches button::before { content:""; width:4px; height:4px; background:#263e26; border:1px inset #474c42; }
  .repeat-one { color:#18212a; background:#a7f17a; padding:0 2px; font-size:7px; }
  .switches button.on::before { background:#9bff69; box-shadow:0 0 3px #8cff64; }

  @media (prefers-reduced-motion:reduce) { .scrolling { animation:none; } }
</style>
