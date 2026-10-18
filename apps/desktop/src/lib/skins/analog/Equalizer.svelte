<script lang="ts">
  import { player, EQ_FREQS, EQ_RANGE, setEqualizer, applyPreset } from "$lib/player.svelte";
  let presets = $state(false);
  let activePointer: number | null = null;
  const values = $derived([player.eq.preamp, ...player.eq.gains]);
  function change(index: number, value: number) {
    value = Math.max(-EQ_RANGE, Math.min(EQ_RANGE, Math.round(value * 2) / 2));
    if (index === 0) setEqualizer({ ...player.eq, enabled: true, preamp: value });
    else {
      const gains = [...player.eq.gains];
      gains[index - 1] = value;
      setEqualizer({ ...player.eq, enabled: true, gains });
    }
  }
  function move(event: PointerEvent, index: number) {
    const rect = event.currentTarget instanceof HTMLElement ? event.currentTarget.getBoundingClientRect() : null;
    if (rect) change(index, (1 - (event.clientY - rect.top - 7) / (rect.height - 14)) * EQ_RANGE * 2 - EQ_RANGE);
  }
  function key(event: KeyboardEvent, index: number) {
    const delta = { ArrowUp: 1, ArrowRight: 1, ArrowDown: -1, ArrowLeft: -1, PageUp: 3, PageDown: -3 }[event.key];
    if (delta !== undefined || event.key === "Home" || event.key === "End") {
      event.preventDefault();
      change(index, event.key === "Home" ? -EQ_RANGE : event.key === "End" ? EQ_RANGE : values[index] + delta!);
    }
  }
</script>

<section class="equalizer" style:--accent={"244, 175, 79"} aria-label="Ecualizador gráfico">
  <header data-tauri-drag-region>Ecualizador <span data-tauri-drag-region>10 bandas · dB</span></header>
  <div class="toolbar">
    <button class:on={player.eq.enabled} aria-pressed={player.eq.enabled} aria-label="Activar ecualizador" onclick={() => setEqualizer({ ...player.eq, enabled: !player.eq.enabled })}>ON</button>
    <button onclick={() => setEqualizer({ ...player.eq, preamp: 0, gains: EQ_FREQS.map(() => 0) })}>Restablecer</button>
    <div class="preset">
      <button aria-expanded={presets} onclick={() => presets = !presets}>Ajustes ▾</button>
      {#if presets}
        <div class="menu">
          <button onclick={() => presets = false}>Cerrar ×</button>
          {#each player.eqPresets as [name]}
            <button onclick={() => { applyPreset(name); presets = false; }}>{name}</button>
          {:else}<span>Sin ajustes disponibles</span>{/each}
        </div>
      {/if}
    </div>
  </div>
  <div class="sliders">
    {#each values as value, index}
      <div class="channel" class:preamp={index === 0}>
        <div class="slider" role="slider" tabindex="0" aria-label={index === 0 ? "Preamplificación" : `${EQ_FREQS[index - 1]} Hz`} aria-orientation="vertical" aria-valuemin={-EQ_RANGE} aria-valuemax={EQ_RANGE} aria-valuenow={value} aria-valuetext={`${value} dB`}
          onpointerdown={(event) => { if (event.button !== 0) return; activePointer = event.pointerId; event.currentTarget.focus(); event.currentTarget.setPointerCapture(event.pointerId); move(event, index); }}
          onpointermove={(event) => { if (activePointer === event.pointerId && event.currentTarget.hasPointerCapture(event.pointerId)) move(event, index); }}
          onpointerup={() => activePointer = null} onpointercancel={() => activePointer = null} onlostpointercapture={() => activePointer = null}
          onwheel={(event) => { event.preventDefault(); if (event.deltaY) change(index, value + (event.deltaY < 0 ? 1 : -1)); }}
          ondblclick={() => change(index, 0)} onkeydown={(event) => key(event, index)}>
          <span class="rail"></span><span class="cap" style:top={`${7 + (EQ_RANGE - value) / (2 * EQ_RANGE) * 68}px`}></span>
        </div>
        <span class="frequency">{index === 0 ? "PRE" : EQ_FREQS[index - 1] >= 1000 ? `${EQ_FREQS[index - 1] / 1000}k` : EQ_FREQS[index - 1]}</span>
        <span class="value">{value > 0 ? "+" : ""}{value}</span>
      </div>
    {/each}
  </div>
  <footer data-tauri-drag-region>±12 dB · Doble clic para poner a cero</footer>
</section>

<style>
.equalizer { box-sizing:border-box; height:230px; padding:14px; background:repeating-linear-gradient(3deg,#352117 0 3px,#2b1a12 4px 8px); color:#ffedc7; border:2px solid #d0b58e; font:11px Georgia, "Times New Roman", serif; border-radius:6px; }
header { display:flex; justify-content:space-between; height:24px; } header span,footer,.frequency { color:#d0b58e; }
.toolbar { display:flex; gap:8px; margin-bottom:12px; } button { font:inherit; color:#ffedc7; background:#21150f; border:1px solid #d0b58e; padding:5px 8px; cursor:pointer; border-radius:6px; } button:hover,button.on { color:#21150f; background:#ffedc7; }
.preset { margin-left:auto; position:relative; } .menu { position:absolute; right:0; top:29px; z-index:5; width:165px; max-height:150px; overflow:auto; background:#21150f; border:1px solid #d0b58e; padding:5px; } .menu button { display:block; width:100%; margin:3px 0; text-align:left; }
.sliders { display:flex; gap:4px; } .channel { flex:1; min-width:0; text-align:center; } .preamp { border-right:1px solid #d0b58e; margin-right:5px; padding-right:5px; }
.slider { height:82px; position:relative; cursor:ns-resize; touch-action:none; background:linear-gradient(transparent 49%,#d0b58e 49% 51%,transparent 51%); } .rail { position:absolute; top:7px; bottom:7px; width:4px; left:calc(50% - 2px); background:#d0b58e; opacity:.5; } .cap { position:absolute; left:50%; translate:-50% -50%; width:17px; height:10px; background:#ffedc7; border:1px solid #d0b58e; box-shadow:0 2px 3px #000; pointer-events:none; border-radius:6px; }
.frequency,.value { display:block; font-size:9px; margin-top:4px; } footer { margin-top:12px; font-size:10px; } button:focus-visible,.slider:focus-visible { outline:2px solid #ffedc7; outline-offset:2px; }
</style>