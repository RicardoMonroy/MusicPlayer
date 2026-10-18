<script lang="ts">
  import { player, EQ_FREQS, EQ_RANGE, setEqualizer, applyPreset } from '$lib/player.svelte';
  let pointer: number | null = null;
  const values = $derived([player.eq.preamp, ...player.eq.gains]);
  function change(index: number, value: number) {
    value=Math.max(-EQ_RANGE,Math.min(EQ_RANGE,Math.round(value*2)/2));
    const eq={...player.eq,enabled:true,gains:[...player.eq.gains]};
    if(index===0) eq.preamp=value; else eq.gains[index-1]=value;
    setEqualizer(eq);
  }
  function move(e: PointerEvent,index: number) {
    if(!(e.currentTarget instanceof HTMLElement)) return;
    const r=e.currentTarget.getBoundingClientRect();
    change(index,EQ_RANGE-(e.clientY-r.top-8)/(r.height-16)*EQ_RANGE*2);
  }
  function key(e: KeyboardEvent,index: number) {
    const d:Record<string,number>={ArrowUp:1,ArrowRight:1,ArrowDown:-1,ArrowLeft:-1,PageUp:3,PageDown:-3};
    if(d[e.key]!==undefined||e.key==='Home'||e.key==='End') {
      e.preventDefault(); change(index,e.key==='Home'?-EQ_RANGE:e.key==='End'?EQ_RANGE:values[index]+d[e.key]);
    }
  }
</script>
<section class="equalizer" aria-label="Ecualizador gráfico">
  <header data-tauri-drag-region>Ecualizador</header>
  <div class="toolbar">
    <button aria-label="Activar ecualizador" aria-pressed={player.eq.enabled} onclick={()=>setEqualizer({...player.eq,enabled:!player.eq.enabled})}>ON {player.eq.enabled?'●':'○'}</button>
    <button onclick={()=>setEqualizer({...player.eq,preamp:0,gains:EQ_FREQS.map(()=>0)})}>Restablecer</button>
    <select aria-label="Preajustes del ecualizador" value="" onchange={(e)=>{applyPreset(e.currentTarget.value);e.currentTarget.value='';}}><option value="" disabled>Preajustes…</option>{#each player.eqPresets as [name]}<option value={name}>{name}</option>{/each}</select>
  </div>
  <div class="graph" aria-hidden="true">{#each player.eq.gains as gain}<span style:height={`${24+(gain+EQ_RANGE)/(2*EQ_RANGE)*70}%`}></span>{/each}</div>
  <div class="scale" aria-hidden="true"><span>+12 dB</span><span>0</span><span>−12 dB</span></div>
  <div class="sliders">
    {#each values as value,index}
      <div class="channel" class:preamp={index===0}>
        <div class="slider" role="slider" tabindex="0" aria-label={index===0?'Preamplificación':`${EQ_FREQS[index-1]} Hz`} aria-orientation="vertical" aria-valuemin={-EQ_RANGE} aria-valuemax={EQ_RANGE} aria-valuenow={value} aria-valuetext={`${value} dB`}
          onpointerdown={(e)=>{if(e.button!==0)return;pointer=e.pointerId;e.currentTarget.setPointerCapture(e.pointerId);move(e,index);}}
          onpointermove={(e)=>{if(pointer===e.pointerId&&e.currentTarget.hasPointerCapture(e.pointerId))move(e,index);}}
          onpointerup={()=>pointer=null} onpointercancel={()=>pointer=null} onlostpointercapture={()=>pointer=null}
          onwheel={(e)=>{e.preventDefault();if(e.deltaY)change(index,value+(e.deltaY<0?1:-1));}}
          ondblclick={()=>change(index,0)} onkeydown={(e)=>key(e,index)} title={`${value} dB`}>
          <i></i><b style:top={`${8+(EQ_RANGE-value)/(2*EQ_RANGE)*72}px`}></b>
        </div>
        <span class="freq">{index===0?'PRE':EQ_FREQS[index-1]>=1000?`${EQ_FREQS[index-1]/1000}k`:EQ_FREQS[index-1]}</span>
        <span class="gain">{value>0?'+':''}{value}</span>
      </div>
    {/each}
  </div>
  <p data-tauri-drag-region>Arrastra · rueda ±1 dB · doble clic: 0</p>
</section>
<style>
.equalizer{box-sizing:border-box;width:100%;height:290px;padding:10px 12px;color:#162b40;background:linear-gradient(120deg,#fafbfc,#dfe3e7);font:10px "Charcoal","Geneva",system-ui,sans-serif;border:1px solid #a5adb6;border-radius:14px;}.equalizer *{box-sizing:border-box}header{height:24px;font-weight:bold;border-bottom:1px solid #8298ac;}.toolbar{display:flex;gap:5px;align-items:center;height:33px}button,select{font:10px "Charcoal","Geneva",system-ui,sans-serif;color:#162b40;background:#e3ecf4;border:1px solid #8298ac;padding:4px;min-width:0;box-shadow:inset 0 1px 0 #fff;}button{cursor:pointer;white-space:nowrap}button[aria-pressed=true]{background:#264f76;color:#fff}select{flex:1;width:55px}.graph{height:44px;display:flex;align-items:end;gap:4px;padding:5px 8px;background:#c5d2df;border:1px solid #8298ac;margin:4px 0 6px}.graph span{flex:1;background:#264f76}.scale{display:flex;justify-content:space-between;font-size:8px;margin-bottom:4px}.sliders{display:flex;gap:3px;padding:4px;background:#c5d2df;border:1px solid #8298ac}.channel{flex:1;min-width:0;text-align:center}.preamp{margin-right:5px;border-right:1px solid #8298ac;padding-right:4px}.slider{height:88px;position:relative;touch-action:none;cursor:ns-resize;background:repeating-linear-gradient(to bottom,transparent 0 21px,#647e9525 21px 22px)}.slider i{position:absolute;top:7px;bottom:7px;left:calc(50% - 2px);width:4px;background:#899eb2;box-shadow:inset 1px 1px #0005}.slider b{position:absolute;left:50%;transform:translate(-50%,-50%);width:13px;height:12px;background:#e3ecf4;border:1px solid #8298ac;box-shadow:inset 0 1px 0 #fff;pointer-events:none}.freq{display:block;margin-top:5px;font-size:8px}.gain{display:block;font-size:8px;margin-top:3px}p{margin:8px 0 0;font-size:8px;text-align:center}button:focus-visible,select:focus-visible,.slider:focus-visible{outline:2px solid #264f76;outline-offset:1px}
</style>
