<script lang="ts">
  import { player, fmtTime } from "$lib/player.svelte";
  const playing = $derived(player.status.index != null && !player.status.paused);
  const duration = $derived(Math.max(0, player.status.duration || 0));
  const position = $derived(Math.max(0, Math.min(duration, player.status.position || 0)));
  const progress = $derived(duration > 0 ? position / duration : 0);
</script>

<div class="hero"><svg viewBox="0 0 280 280" aria-hidden="true"><circle class="track" cx="140" cy="140" r="124"/><circle class="progress" cx="140" cy="140" r="124" pathLength="1" stroke-dasharray="1" stroke-dashoffset={1-progress}/></svg><div class="inside"><span class="label">{playing ? 'AHORA' : player.track ? 'EN PAUSA' : 'EN SILENCIO'}</span><div class="elapsed">{fmtTime(position)}</div><div class="remaining">−{fmtTime(duration-position)}</div><span class="label end">TIEMPO RESTANTE</span></div></div>

<style>

  .hero { width:100%; height:100%; position:relative; box-sizing:border-box; overflow:hidden; }
  .hero :global(*) { box-sizing:border-box; }
  .hero { color:#202421; font-family:'Helvetica Neue',system-ui,sans-serif; }
  svg { position:absolute; inset:0; width:100%; height:100%; fill:none; stroke-width:1; rotate:-90deg; } .track { stroke:#cbcfc5; } .progress { stroke:#476351; stroke-width:2; transition:stroke-dashoffset .5s linear; }
  .inside { position:absolute; inset:50px 25px; display:flex; flex-direction:column; align-items:center; justify-content:center; }
  .label { font-size:8px; letter-spacing:2.5px; color:#595e59; } .elapsed { font-size:58px; font-weight:200; letter-spacing:-3px; line-height:1.4; font-variant-numeric:tabular-nums; max-width:100%; } .remaining { font-size:23px; font-weight:300; color:#595e59; font-variant-numeric:tabular-nums; } .end { margin-top:12px; font-size:7px; }
  @media (prefers-reduced-motion:reduce) { .progress { transition:none; } }

</style>
