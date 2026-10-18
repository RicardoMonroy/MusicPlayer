<script lang="ts">
  import { player, fmtTime } from "$lib/player.svelte";
  const playing = $derived(player.status.index != null && !player.status.paused);
  const duration = $derived(Math.max(0, player.status.duration || 0));
  const position = $derived(Math.max(0, Math.min(duration, player.status.position || 0)));
  const progress = $derived(duration > 0 ? position / duration : 0);
</script>

<div class="hero"><div class="unit"><header>REPRODUCTOR · ESTÉREO</header><div class="lcd"><div class="state">{playing ? '▶ REPRODUCIENDO' : player.track ? 'Ⅱ EN PAUSA' : '■ DETENIDO'}</div><div class="time">{fmtTime(position)}</div><div class="spectrum" aria-hidden="true">{#each Array(19) as _, i}<i style:height={`${playing ? Math.max(2, (player.spectrum[i] ?? 0)*100) : 2}%`}></i>{/each}</div><div class="scale"><span>60 Hz</span><span>ESPECTRO</span><span>16 kHz</span></div><div class="marquee"><span style:animation-play-state={playing ? 'running' : 'paused'}>{player.track ? `${player.track.artist} - ${player.track.title}` : 'Elige una canción'}</span></div><div class="progress"><i style:width={`${progress*100}%`}></i></div><footer><span>PCM · ESTÉREO</span><span>{fmtTime(duration)}</span></footer></div></div></div>

<style>

  .hero { width:100%; height:100%; position:relative; box-sizing:border-box; overflow:hidden; }
  .hero :global(*) { box-sizing:border-box; }
  .hero { display:grid; place-items:center; font:10px monospace; }
  .unit { width:274px; padding:8px; border:2px solid; border-color:#858891 #16171d #15161b #737680; background:linear-gradient(120deg,#484b55,#292b34); box-shadow:inset 0 0 0 2px #24252e,0 6px 10px #0004; }
  header { text-align:center; color:#d0d2db; font-size:9px; letter-spacing:2px; margin-bottom:9px; }
  .lcd { padding:10px; background:repeating-linear-gradient(0deg,#0002 0 1px,transparent 1px 3px),#061009; color:#80f47b; border:2px inset #686c71; }
  .state { font-size:8px; letter-spacing:1px; } .time { font:52px/1.3 monospace; font-variant-numeric:tabular-nums; color:#a3ff83; letter-spacing:-3px; text-shadow:0 0 8px #87fa6544; }
  .spectrum { display:flex; align-items:end; gap:3px; height:62px; border-bottom:1px solid #376833; }
  .spectrum i { flex:1; min-height:2px; background:repeating-linear-gradient(to top,transparent 0 3px,#061009 3px 4px),linear-gradient(to top,#2aaf41,#c8f54e 80%,#ffbd57); transition:height .16s linear; }
  .scale,footer { display:flex; justify-content:space-between; font-size:7px; color:#70a469; margin-top:5px; }
  .marquee { overflow:hidden; margin-top:12px; border-top:1px solid #274526; padding:7px 0; } .marquee span { display:block; width:max-content; min-width:100%; white-space:nowrap; animation:scroll 14s linear infinite alternate; }
  .progress { height:3px; background:#18351a; } .progress i { display:block; height:100%; background:#a3ff83; }
  @keyframes scroll { 0%,15% { transform:translateX(0); } 85%,100% { transform:translateX(min(0px, calc(230px - 100%))); } }
  @media (prefers-reduced-motion:reduce) { .marquee span { animation:none; } .spectrum i { height:2px !important; transition:none; } }

</style>
