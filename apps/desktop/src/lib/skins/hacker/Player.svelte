<script lang="ts">
 import {player,prev,next,togglePause,seek,fmtTime} from '$lib/player.svelte';
 import {compact} from '$lib/compact.svelte';
 const ratio=$derived(player.status.duration>0?Math.max(0,Math.min(1,player.status.position/player.status.duration)):0);
 const blocks=$derived(Math.round(ratio*28));
 const playing=$derived(player.status.index!==null&&!player.status.paused);
</script>
<div class="terminal" data-tauri-drag-region>
 <header data-tauri-drag-region>terminal / audio</header>
 <p data-tauri-drag-region>&gt; now_playing --artist</p><h2 data-tauri-drag-region title={player.track?.artist}>{player.track?.artist||'Sin artista'}</h2>
 <p data-tauri-drag-region>--title <span class="cursor">▌</span></p><h1 data-tauri-drag-region title={player.track?.title}>{player.track?.title||'Esperando una canción'}</h1>
 <div class="spectrum" aria-hidden="true">{#each Array(19) as _,i}<span>{'▮'.repeat(Math.max(1,Math.round((player.spectrum[i]??0)*5)))}</span>{/each}</div>
 <label class="seek"><span aria-hidden="true">[{'#'.repeat(blocks)}{'–'.repeat(28-blocks)}]</span><input aria-label="Posición de reproducción" type="range" min="0" max={player.status.duration||1} step="0.1" value={player.status.position} disabled={!player.status.duration} oninput={e=>seek(+e.currentTarget.value)}/></label>
 <div class="time" data-tauri-drag-region>{fmtTime(player.status.position)} / {fmtTime(player.status.duration)} · {playing?'REPRODUCIENDO':'EN ESPERA'}</div>
 <nav aria-label="Controles"><button aria-label="Canción anterior" onclick={prev}>|&lt;</button><button onclick={togglePause}>{playing?'[ PAUSAR ]':'[ REPRODUCIR ]'}</button><button aria-label="Canción siguiente" onclick={next}>&gt;|</button><button aria-label="Ecualizador" aria-pressed={compact.eq} onclick={()=>compact.eq=!compact.eq}>EQ</button><button aria-label="Lista de reproducción" aria-pressed={compact.playlist} onclick={()=>compact.playlist=!compact.playlist}>PL</button></nav>
</div>
<style>
.terminal{box-sizing:border-box;width:100%;height:100%;padding:12px 20px;background:repeating-linear-gradient(#0000 0 3px,#0003 3px 4px),#030b06;color:#83ffa1;border:2px solid #366a43;font:12px 'Courier New',monospace;text-shadow:0 0 6px #50ff7050;overflow:hidden}header{height:25px;padding-right:132px;color:#72b886;font-size:10px}p{margin:3px 0;color:#72b886;font-size:11px}h1,h2{margin:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:17px;line-height:24px}h2{font-size:14px}.cursor{animation:blink 1s steps(1) infinite}.spectrum{height:48px;display:flex;gap:8px;align-items:flex-end;overflow:hidden;margin:7px 0}.spectrum span{writing-mode:vertical-rl;font-size:9px;line-height:12px}.seek{display:block;position:relative;font-size:20px;letter-spacing:0;height:28px;white-space:nowrap}.seek input{position:absolute;inset:0;width:100%;height:100%;opacity:0;cursor:pointer}.seek:focus-within{outline:2px solid #83ffa1}.time{font-size:10px;color:#72b886;margin:3px 0 12px}nav{display:flex;gap:6px}button{background:#0c2012;border:1px solid #366a43;color:inherit;font:11px 'Courier New',monospace;padding:7px;cursor:pointer}button:hover,button[aria-pressed=true]{background:#83ffa1;color:#030b06}button:focus-visible{outline:2px solid #83ffa1;outline-offset:2px}@keyframes blink{50%{opacity:0}}@media(prefers-reduced-motion:reduce){.cursor{animation:none}}
</style>
