<script lang="ts">
  import { player, EQ_FREQS, EQ_RANGE, setEqualizer, applyPreset } from "$lib/player.svelte";
  let { theme }: { theme: "classic" | "cassette" | "vinyl" } = $props();
  let presets = $state(false);
  let activePointer: number | null = null;
  const values = $derived([player.eq.preamp, ...player.eq.gains]);
  const curve = $derived(player.eq.gains.map((gain, i, gains) => {
    const x = 8 + i * 32;
    const y = 20 - gain / EQ_RANGE * 15;
    if (!i) return `M ${x} ${y}`;
    const previousY = 20 - gains[i - 1] / EQ_RANGE * 15;
    return `C ${x - 16} ${previousY}, ${x - 16} ${y}, ${x} ${y}`;
  }).join(" "));
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

<section class="equalizer {theme}" aria-label="Ecualizador gráfico">
  <header data-tauri-drag-region>ECUALIZADOR <span>10 BANDAS · ESTÉREO</span></header>
  <div class="toolbar">
    <button class:on={player.eq.enabled} aria-pressed={player.eq.enabled} onclick={() => setEqualizer({ ...player.eq, enabled: !player.eq.enabled })}><i></i> ON</button>
    <button onclick={() => setEqualizer({ ...player.eq, enabled: true, preamp: 0, gains: EQ_FREQS.map(() => 0) })}>Plano</button>
    <svg class="curve" viewBox="0 0 304 40" preserveAspectRatio="none" role="img" aria-label="Curva de respuesta de las diez bandas"><path class="grid" d="M0 20H304 M8 0V40 M152 0V40 M296 0V40"/><path d={curve}/></svg>
    <div class="preset">
      <button aria-expanded={presets} onclick={() => presets = !presets}>PRESETS ▾</button>
      {#if presets}
        <div class="menu">
          <button class="close" onclick={() => presets = false}>Cerrar ×</button>
          {#each player.eqPresets as [name]}
            <button onclick={() => { applyPreset(name); presets = false; }}>{name}</button>
          {:else}<span>Sin ajustes disponibles</span>{/each}
        </div>
      {/if}
    </div>
  </div>
  <div class="sliders">
    <div class="marks" aria-hidden="true"><span>+12</span><span>0 dB</span><span>−12</span></div>
    {#each values as value, index}
      <div class="channel" class:preamp={index === 0}>
        <div class="slider" role="slider" tabindex="0" aria-label={index === 0 ? "Preamplificación" : `${EQ_FREQS[index - 1]} Hz`} aria-orientation="vertical" aria-valuemin={-EQ_RANGE} aria-valuemax={EQ_RANGE} aria-valuenow={value} aria-valuetext={`${value} dB`}
          onpointerdown={(event) => { if (event.button !== 0) return; activePointer = event.pointerId; event.currentTarget.setPointerCapture(event.pointerId); move(event, index); }}
          onpointermove={(event) => { if (activePointer === event.pointerId && event.currentTarget.hasPointerCapture(event.pointerId)) move(event, index); }}
          onpointerup={() => activePointer = null} onpointercancel={() => activePointer = null} onlostpointercapture={() => activePointer = null}
          onwheel={(event) => { event.preventDefault(); if (event.deltaY) change(index, value + (event.deltaY < 0 ? 1 : -1)); }}
          ondblclick={() => change(index, 0)} onkeydown={(event) => key(event, index)}
          title={`${index === 0 ? "PREAMP" : EQ_FREQS[index - 1] + " Hz"}: ${value} dB`}>
          <span class="rail"></span><span class="cap" style:top={`${7 + (EQ_RANGE - value) / (2 * EQ_RANGE) * 68}px`} style:--gain-color={`hsl(${120 - (value + EQ_RANGE) / (2 * EQ_RANGE) * 120} 80% 58%)`}></span>
        </div>
        <span class="frequency">{index === 0 ? "PREAMP" : EQ_FREQS[index - 1] >= 1000 ? `${EQ_FREQS[index - 1] / 1000}K` : EQ_FREQS[index - 1]}</span>
      </div>
    {/each}
  </div>
</section>

<style>
  .equalizer { --ink:#c5c6ce; --light:#92949b; --dark:#17191f; --cap:#b4b88d; --curve:#91f16e; box-sizing:border-box; width:100%; height:210px; padding:0 12px 12px; border:2px solid; border-color:var(--light) var(--dark) var(--dark) var(--light); background:linear-gradient(110deg,#41434c,#30323b 55%,#454751); color:var(--ink); font:9px monospace; user-select:none; }
  header { height:28px; display:flex; align-items:center; gap:18px; padding-right:132px; font-weight:bold; letter-spacing:1px; white-space:nowrap; }
  header span { font-size:7px; opacity:.6; }
  .toolbar { display:flex; align-items:center; gap:6px; height:43px; }
  button { color:var(--ink); font:9px monospace; border:2px solid; border-color:var(--light) var(--dark) var(--dark) var(--light); background:linear-gradient(#686b74,#3b3e47); padding:4px 6px; cursor:pointer; white-space:nowrap; }
  button:active { border-style:inset; }
  i { display:inline-block; width:5px; height:5px; background:#18271c; border:1px solid #121710; }
  .on i { background:#9dff65; box-shadow:0 0 4px #91ed73; }
  .on { color:#a2f989; }
  .curve { min-width:30px; flex:1; height:32px; background:#080e0b; border:2px inset #656973; }
  path { fill:none; stroke:var(--curve); stroke-width:1.6; }
  path.grid { stroke:#304331; stroke-width:.5; }
  .preset { position:relative; }
  .menu { position:absolute; right:0; top:27px; z-index:5; width:160px; max-height:140px; overflow:auto; padding:4px; background:var(--dark); border:1px solid var(--light); box-shadow:0 3px 8px #0009; }
  .menu button { display:block; width:100%; text-align:left; margin-bottom:2px; }
  .menu span { display:block; padding:6px; }
  .sliders { display:flex; gap:3px; padding-top:4px; }
  .marks { width:27px; flex-shrink:0; height:82px; display:flex; flex-direction:column; justify-content:space-between; font-size:8px; padding:3px 0; }
  .channel { flex:1; min-width:0; text-align:center; }
  .preamp { margin-right:10px; }
  .slider { position:relative; height:82px; touch-action:none; cursor:ns-resize; background:repeating-linear-gradient(to bottom,transparent 0 12px,#91939233 12px 13px); }
  .rail { position:absolute; width:5px; top:4px; bottom:4px; left:calc(50% - 3px); background:#101317; border:1px inset #777; }
  .cap { position:absolute; left:50%; translate:-50% -50%; width:21px; height:12px; border:2px outset #b4b7a1; background:linear-gradient(transparent 40%,#171c1680 40% 60%,transparent 60%),var(--gain-color); box-shadow:0 2px 3px #0008; pointer-events:none; }
  .frequency { display:block; margin-top:7px; font-size:8px; }
  button:focus-visible,.slider:focus-visible { outline:2px solid var(--curve); outline-offset:2px; }
  .cassette { --ink:#302d29; --light:#e2e1dc; --dark:#57585a; --curve:#e7b86d; background:repeating-linear-gradient(0deg,#ffffff08 0 1px,#00000008 1px 2px),linear-gradient(110deg,#bfc0c2,#8f9297,#d3d2cf); border-radius:10px; }
  .cassette header { letter-spacing:.5px; }
  .cassette header span,.vinyl header span { display:none; }
  .cassette button { background:#efe8d8; }
  .cassette .on { color:#304b25; }
  .cassette .cap { background:linear-gradient(#e9e8df,#a8a9a6); border-color:#ededdf; height:15px; width:19px; }
  .vinyl { --ink:#f0dfbf; --light:#997b50; --dark:#2a1c12; --curve:#f2b366; background:repeating-linear-gradient(92deg,#00000010 0 3px,transparent 3px 9px),linear-gradient(160deg,#6b4a32,#3e2a1c); border-radius:10px; }
  .vinyl button { background:linear-gradient(#6c583c,#3e3020); }
  .vinyl .cap { background:linear-gradient(#f3e6c9,#baa16b); border-color:#c3a46b; width:17px; height:14px; border-radius:2px; }
  .vinyl .preamp { margin-right:4px; }
  .vinyl .frequency { font-size:7px; }
</style>
