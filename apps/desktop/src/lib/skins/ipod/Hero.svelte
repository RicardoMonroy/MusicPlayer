<script lang="ts">
  import { player, fmtTime } from "$lib/player.svelte";
  const playing = $derived(player.status.index != null && !player.status.paused);
  const duration = $derived(Math.max(0, player.status.duration || 0));
  const position = $derived(Math.max(0, Math.min(duration, player.status.position || 0)));
  const progress = $derived(duration > 0 ? position / duration : 0);
</script>

<div class="hero"><div class="device"><div class="lcd"><header><span>Ahora suena</span><span>{playing ? '▶' : 'Ⅱ'}</span></header><div class="content"><div class="cover">{#if player.coverUrl}<img src={player.coverUrl} alt="Portada del álbum" />{:else}<span class="empty">♪</span>{/if}</div><div class="info"><b>{player.track?.title || 'Sin canción'}</b><span>{player.track?.artist || 'Elige música'}</span><span>{player.track?.album || 'Sin álbum'}</span></div></div><div class="progress"><i style:width={`${progress*100}%`}></i></div><footer><span>{fmtTime(position)}</span><span>−{fmtTime(duration-position)}</span></footer></div><div class="wheel"><span>MENÚ</span><div></div><b>◀◀</b><b>▶▶</b><small>▶Ⅱ</small></div></div></div>

<style>

  .hero { width:100%; height:100%; position:relative; box-sizing:border-box; overflow:hidden; }
  .hero :global(*) { box-sizing:border-box; }
  img { width:100%; height:100%; object-fit:cover; display:block; }
  .hero { display:grid; place-items:center; font:10px system-ui,sans-serif; color:#202b36; }
  .device { width:234px; height:272px; padding:16px; overflow:hidden; border-radius:22px; background:linear-gradient(110deg,#e5e7e9,#fff 45%,#e2e5e8); border:1px solid #aeb3b9; box-shadow:inset 0 1px 2px #fff,0 4px 7px #0002; }
  .lcd { padding:7px; height:169px; border:3px solid #4b5056; border-radius:6px; background:linear-gradient(130deg,#e2edf5,#b8cbdc); box-shadow:inset 0 1px 4px #0005; }
  header { display:flex; justify-content:space-between; font-weight:700; border-bottom:1px solid #8c9fae; padding-bottom:5px; }
  .content { display:flex; gap:8px; margin:10px 0; height:81px; } .cover { flex:none; width:78px; height:78px; background:#a1b6c7; display:grid; place-items:center; border:1px solid #73899c; } .empty { font-size:30px; }
  .info { min-width:0; display:flex; flex-direction:column; justify-content:center; gap:6px; } .info > * { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; } .info span { font-size:8px; }
  .progress { height:6px; border:1px solid #6a8296; background:#e2eaf0; } .progress i { display:block; height:100%; background:linear-gradient(#6e9abe,#315d83); } footer { display:flex; justify-content:space-between; margin-top:4px; font-size:8px; font-variant-numeric:tabular-nums; }
  .wheel { width:120px; height:120px; border-radius:50%; background:#e0e3e6; position:relative; margin:11px auto 0; color:#7c8389; } .wheel div { position:absolute; inset:38px; border-radius:50%; background:linear-gradient(#fff,#ebedef); } .wheel span { position:absolute; top:10px; width:100%; text-align:center; font-size:8px; } .wheel b { position:absolute; top:55px; left:10px; font-size:9px; } .wheel b + b { left:auto; right:10px; } .wheel small { position:absolute; bottom:10px; width:100%; text-align:center; }

</style>
