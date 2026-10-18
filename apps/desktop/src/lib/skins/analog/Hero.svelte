<script lang="ts">
 import {onMount} from 'svelte';
 import {player} from '$lib/player.svelte';
 let level=$state(0);
 let reduced=$state(false);
 const energy=$derived(player.status.paused?0:player.spectrum.reduce((a,b)=>a+b,0)/Math.max(1,player.spectrum.length));
 onMount(()=>{
  const media=matchMedia('(prefers-reduced-motion: reduce)');
  const update=()=>{reduced=media.matches;};update();media.addEventListener('change',update);
  let frame=0;let previous=0;
  const tick=(time:number)=>{const dt=Math.min(100,time-previous);previous=time;if(!document.hidden&&!reduced)level+=(energy-level)*(1-Math.exp(-dt/(energy>level?65:360)));frame=requestAnimationFrame(tick);};
  frame=requestAnimationFrame(tick);
  return()=>{cancelAnimationFrame(frame);media.removeEventListener('change',update);};
 });
</script>
<div class="amplifier" data-tauri-drag-region>
 <div class="plate" data-tauri-drag-region>AMPLIFICADOR · VÁLVULAS</div>
 <div class="meter" aria-label="Medidor de nivel de audio"><div class="scale" aria-hidden="true">−20　−10　−5　0　+3</div><div class="arc"></div><span class="needle" style:transform={`rotate(${-48+(reduced?0:level)*105}deg)`}></span><b>VU</b></div>
 <div class="tubes" aria-hidden="true">{#each [0,6,12] as band}<div class="socket"><div class="tube" style:--glow={reduced?.35:.2+(player.status.paused?0:(player.spectrum[band]??0))*.8}><i></i><span>▥</span></div></div>{/each}</div>
 <footer data-tauri-drag-region>ESTÉREO · ALTA FIDELIDAD</footer>
</div>
<style>
.amplifier{box-sizing:border-box;width:100%;height:100%;padding:18px;background:repeating-linear-gradient(4deg,#352117 0 4px,#40291a 5px,#2c1b12 8px);border:5px solid #21150f;border-radius:8px;box-shadow:inset 0 0 15px #000;color:#ffedc7;font:10px Georgia,serif;overflow:hidden}.plate{text-align:center;letter-spacing:1.5px;margin-bottom:12px}.meter{height:92px;background:radial-gradient(ellipse at bottom,#fff1c5,#c4a66b);border:5px solid #17120d;border-radius:48% 48% 6px 6px;position:relative;overflow:hidden;color:#342319;box-shadow:inset 0 0 9px #694821}.scale{position:absolute;top:20px;width:100%;text-align:center;font:11px Georgia,serif}.arc{position:absolute;inset:12px 13px 20px;border-top:2px solid #6f3924;border-radius:50%}.needle{position:absolute;bottom:-8px;left:50%;width:2px;height:84px;background:#863818;transform-origin:bottom;box-shadow:1px 0 2px #0005}.meter b{position:absolute;bottom:8px;left:44%;font-size:17px}.tubes{display:flex;justify-content:space-around;margin-top:12px;height:76px}.socket{width:48px;border-bottom:9px solid #16110d;display:flex;justify-content:center}.tube{position:relative;width:34px;border:1px solid #d0b58e80;border-radius:18px 18px 7px 7px;background:linear-gradient(90deg,#ffffff05,#ffffff20 24%,#120a04a0 40%,#ffffff15);box-shadow:inset 0 0 9px #ff8a1233}.tube i{position:absolute;inset:14px 8px 3px;background:#ff941b;opacity:var(--glow);box-shadow:0 0 18px 5px #ff7200;border-radius:50%}.tube span{position:absolute;bottom:4px;left:7px;font-size:28px;color:#ffb96e;text-shadow:0 0 6px #f80}footer{text-align:center;font-size:8px;letter-spacing:2px;margin-top:9px}
@media(prefers-reduced-motion:reduce){.needle{transform:rotate(-48deg)!important}.tube i{opacity:.35}}
</style>
