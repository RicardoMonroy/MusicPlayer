<script lang="ts">
  import { player, togglePause, prev, next, seek, setVolume, fmtTime } from '$lib/player.svelte';
  import { compact } from '$lib/compact.svelte';
  let turning = $state(false);
  let volume = $state(100);
  let angle = 0;
  let pointer: number | null = null;
  const playing = $derived(player.status.index !== null && !player.status.paused);
  function radians(e: PointerEvent) {
    const r = e.currentTarget instanceof HTMLElement ? e.currentTarget.getBoundingClientRect() : null;
    return r ? Math.atan2(e.clientY-r.top-r.height/2,e.clientX-r.left-r.width/2) : 0;
  }
  function start(e: PointerEvent) {
    if (e.button !== 0 || (e.target instanceof Element && e.target.closest('button'))) return;
    pointer=e.pointerId; e.currentTarget instanceof HTMLElement && e.currentTarget.setPointerCapture(pointer);
    angle=radians(e); volume=player.status.volume; turning=true;
  }
  function move(e: PointerEvent) {
    if (pointer!==e.pointerId) return;
    const a=radians(e); const delta=Math.atan2(Math.sin(a-angle),Math.cos(a-angle)); angle=a;
    volume=Math.max(0,Math.min(100,volume+delta*100/(Math.PI*2))); setVolume(Math.round(volume));
  }
  function end() { pointer=null; turning=false; }
  function key(e: KeyboardEvent) {
    const d= e.key==='ArrowUp'||e.key==='ArrowRight'?5:e.key==='ArrowDown'||e.key==='ArrowLeft'?-5:0;
    if(d || e.key==='Home'||e.key==='End') { e.preventDefault(); setVolume(e.key==='Home'?0:e.key==='End'?100:Math.max(0,Math.min(100,player.status.volume+d))); }
  }
</script>
<div class="device" data-tauri-drag-region>
  <div class="top" data-tauri-drag-region></div>
  <section class="lcd" aria-label="Reproducción">
    <header data-tauri-drag-region>Reproduciendo <span data-tauri-drag-region>{playing?'▶︎':'Ⅱ'} ▰</span></header>
    <div class="song" data-tauri-drag-region>
      <div class="art" data-tauri-drag-region>{#if player.coverUrl}<img src={player.coverUrl} alt="Portada del álbum" draggable="false" />{:else}<span>♫</span>{/if}</div>
      <div class="text" data-tauri-drag-region><strong title={player.track?.title}>{player.track?.title || 'Sin reproducción'}</strong><span title={player.track?.artist}>{player.track?.artist || 'Elige una canción'}</span><small title={player.track?.album}>{player.track?.album || 'Sin álbum'}</small></div>
    </div>
    {#if turning}<div class="volume">Volumen · {Math.round(volume)}%<progress max="100" value={volume}></progress></div>{:else}
    <input aria-label="Posición de reproducción" type="range" min="0" max={player.status.duration||1} step="0.1" value={player.status.position} disabled={!player.status.duration} oninput={(e)=>seek(+e.currentTarget.value)} />
    <div class="times"><span>{fmtTime(player.status.position)}</span><span>−{fmtTime(Math.max(0,player.status.duration-player.status.position))}</span></div>{/if}
  </section>
  <div class="wheel-wrap">
    <div class="wheel" role="slider" tabindex="0" aria-label="Volumen: gira la rueda" aria-valuemin="0" aria-valuemax="100" aria-valuenow={turning?Math.round(volume):player.status.volume} onkeydown={key} onpointerdown={start} onpointermove={move} onpointerup={end} onpointercancel={end} onlostpointercapture={end}>
      <button class="menu" aria-label="Mostrar lista de reproducción" aria-pressed={compact.playlist} onclick={()=>compact.playlist=!compact.playlist}>MENU</button>
      <button class="prev" aria-label="Anterior" onclick={prev}>⏮︎</button><button class="next" aria-label="Siguiente" onclick={next}>⏭︎</button>
      <button class="bottom" aria-label={playing?'Pausar':'Reproducir'} onclick={togglePause}>⏯︎</button>
      <button class="center" aria-label={playing?'Pausar':'Reproducir'} onclick={togglePause}></button>
    </div>
  </div>
  <div class="switches"><button aria-label="Ecualizador" aria-pressed={compact.eq} onclick={()=>compact.eq=!compact.eq}>EQ</button><span data-tauri-drag-region>Gira para ajustar el volumen</span><button aria-label="Lista de reproducción" aria-pressed={compact.playlist} onclick={()=>compact.playlist=!compact.playlist}>PL</button></div>
</div>
<style>
.device{box-sizing:border-box;width:260px;height:430px;padding:0 20px 14px;border:1px solid #aeb2b7;border-radius:28px;background:linear-gradient(115deg,#fff 10%,#f7f7f8 60%,#d7dadd);box-shadow:inset 2px 2px 3px #fff,inset -2px -2px 4px #a1a6ab;color:#253344;font:12px "Charcoal","Geneva",system-ui,sans-serif}.top{height:36px}.lcd{height:170px;box-sizing:border-box;border:3px solid #7b8790;border-radius:6px;background:linear-gradient(#d1deea,#b5c5d5);box-shadow:0 1px 0 #fff,inset 0 1px 4px #53617080;color:#162b40;overflow:hidden}header{display:flex;justify-content:space-between;padding:6px;border-bottom:1px solid #667e93;font-size:11px;font-weight:bold;background:linear-gradient(#e5edf3,#a8b9ca)}.song{display:flex;gap:9px;padding:12px 9px 9px}.art{flex-shrink:0;width:56px;height:60px;display:grid;place-items:center;background:#a1b4c6;border:1px solid #647e95;font-size:30px}.art img{width:100%;height:100%;object-fit:cover}.text{min-width:0;display:flex;flex-direction:column;gap:5px}.text>*{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.text strong{font-size:12px}.text span,.text small{font-size:10px}.lcd input{display:block;width:calc(100% - 18px);height:12px;margin:0 9px;accent-color:#264f76}.times{display:flex;justify-content:space-between;padding:3px 10px;font-size:10px;font-variant-numeric:tabular-nums}.volume{padding:0 10px;font-size:11px}.volume progress{display:block;width:100%;height:12px;margin-top:6px;accent-color:#264f76}.wheel-wrap{display:grid;place-items:center;height:188px}.wheel{position:relative;width:174px;height:174px;border-radius:50%;background:linear-gradient(135deg,#fff,#eeeef0);box-shadow:0 0 0 1px #e1e2e4,inset 0 1px 3px #fff;touch-action:none}.wheel button{position:absolute;border:0;background:none;color:#606b77;font:bold 13px system-ui;cursor:pointer;padding:6px}.wheel .menu{top:8px;left:58px;font-size:10px}.wheel .prev{left:8px;top:70px}.wheel .next{right:8px;top:70px}.wheel .bottom{bottom:7px;left:68px}.wheel .center{width:64px;height:64px;top:55px;left:55px;border-radius:50%;background:linear-gradient(130deg,#eff0f2,#fff);box-shadow:0 1px 3px #9ca3ab,inset 0 1px 2px #fff}.switches{display:flex;align-items:center;justify-content:space-between;gap:5px}.switches span{font-size:8px;color:#4b5663}.switches button{border:1px solid #aeb7bf;border-radius:4px;background:#f5f7f9;color:#344b61;font:bold 9px system-ui;padding:3px 5px;cursor:pointer}.switches button[aria-pressed=true]{background:#c5d8e9}button:focus-visible,.wheel:focus-visible,input:focus-visible{outline:2px solid #285f90;outline-offset:2px}
</style>
