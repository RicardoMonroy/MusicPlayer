<script lang="ts">
 import { player } from '$lib/player.svelte';
 const glyphs = ['01:Σ/1010','λ010>_01','101::010','0xA7/001','Σ01λ1010','1001>010','λ:010110','010/Σ001','101001λ0','0xFF:010','101>0010','λ001:101'];
</script>
<div class="crt" aria-label="Lluvia de caracteres del terminal" data-tauri-drag-region>
 <div class="rain" aria-hidden="true">{#each glyphs as text,i}<span style:left={`${i*8.2}%`} style:animation-delay={`${-i*1.7}s`} style:animation-duration={`${9+i%4}s`}>{text.repeat(4)}</span>{/each}</div>
 <div class="readout" data-tauri-drag-region><small data-tauri-drag-region>SEÑAL / {player.status.paused ? 'PAUSA' : player.track ? 'ACTIVA' : 'ESPERA'}</small><strong data-tauri-drag-region>SONIDO<span class="cursor">▌</span></strong><div class="spectrum" aria-hidden="true">{#each Array(19) as _, i}<i style:height={`${5+(player.spectrum[i]??0)*45}px`}></i>{/each}</div><small data-tauri-drag-region>19 CANALES · SALIDA ESTÉREO</small></div>
</div>
<style>
.crt{width:100%;height:100%;min-height:0;position:relative;overflow:hidden;background:radial-gradient(ellipse,#12351c,#020904 80%);color:#83ffa1;font-family:'Courier New',monospace;box-shadow:inset 0 0 30px #000;border:1px solid #366a43;box-sizing:border-box}.crt::after{content:'';position:absolute;inset:0;pointer-events:none;background:repeating-linear-gradient(transparent 0 3px,#0005 3px 4px)}.rain{position:absolute;inset:0;opacity:.3}.rain span{position:absolute;top:-120%;width:12px;word-break:break-all;font-size:14px;line-height:19px;animation:fall linear infinite}.readout{position:absolute;inset:0;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:18px;text-shadow:0 0 8px #42ff77}small{font-size:10px;letter-spacing:1px}strong{font-size:32px}.spectrum{height:50px;display:flex;align-items:flex-end;gap:4px}.spectrum i{width:6px;background:repeating-linear-gradient(to top,#83ffa1 0 3px,transparent 3px 5px)}.cursor{animation:blink 1.2s steps(1) infinite}@keyframes fall{to{transform:translateY(620px)}}@keyframes blink{50%{opacity:0}}@media(prefers-reduced-motion:reduce){.rain span{animation:none;top:0}.cursor{animation:none}.rain{opacity:.16}}
</style>
