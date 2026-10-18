<script lang="ts">
  import { player, fmtTime } from "$lib/player.svelte";
  const playing = $derived(player.status.index != null && !player.status.paused);
  const duration = $derived(Math.max(0, player.status.duration || 0));
  const position = $derived(Math.max(0, Math.min(duration, player.status.position || 0)));
  const progress = $derived(duration > 0 ? position / duration : 0);
</script>

<div class="hero"><div class="dialog"><header>Reproductor multimedia <span>− □</span></header><nav>Archivo　Ver　Reproducir</nav><div class="display"><div class="cover">{#if player.coverUrl}<img src={player.coverUrl} alt="Portada del álbum" />{:else}<span class="empty">♪</span>{/if}</div><div class="visual"><span>ESPECTRO</span><div class="spectrum" aria-hidden="true">{#each Array(19) as _, i}<i style:height={`${playing ? Math.max(2, (player.spectrum[i] ?? 0)*100) : 2}%`}></i>{/each}</div></div></div><div class="title">{player.track?.title || 'Sin archivo'}</div><div class="progress"><i style:width={`${progress*100}%`}></i></div><footer><span>{playing ? 'Reproduciendo' : 'Detenido / Pausa'}</span><span>{fmtTime(position)}</span></footer></div></div>

<style>

  .hero { width:100%; height:100%; position:relative; box-sizing:border-box; overflow:hidden; }
  .hero :global(*) { box-sizing:border-box; }
  img { width:100%; height:100%; object-fit:cover; display:block; }
  .hero { display:grid; place-items:center; font:10px Tahoma,system-ui,sans-serif; color:#000; }
  .dialog { width:274px; padding:3px; border:2px solid; border-color:#fff #404040 #404040 #fff; background:#c0c0c0; box-shadow:1px 3px 6px #0004; }
  header { display:flex; justify-content:space-between; background:linear-gradient(90deg,#000080,#174ba1); color:white; padding:4px; font-size:10px; font-weight:bold; } header span { background:#c0c0c0; color:#000; padding:0 3px; box-shadow:inset 1px 1px white,inset -1px -1px #444; }
  nav { padding:6px 3px; font-size:9px; } .display { padding:5px; display:flex; gap:5px; height:122px; border:2px inset #eee; background:#000; }
  .cover { width:103px; flex:none; border:2px inset #aaa; background:#808080; display:grid; place-items:center; } .empty { font:40px serif; color:#ddd; }
  .visual { flex:1; min-width:0; padding:4px; color:#00df00; font:7px monospace; background:repeating-linear-gradient(0deg,transparent 0 9px,#004000 9px 10px); } .spectrum { display:flex; align-items:end; gap:1px; height:85px; } .spectrum i { flex:1; min-height:2px; background:repeating-linear-gradient(to top,transparent 0 3px,#000 3px 4px),linear-gradient(to top,#00b000,#b5f000 80%,#ff0); transition:height .18s; }
  .title { margin:7px 3px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; } .progress { height:9px; margin:0 3px 7px; border:2px inset #ddd; } .progress i { display:block; height:100%; background:#000080; }
  footer { display:flex; justify-content:space-between; gap:4px; font-size:9px; } footer span { padding:3px; border:1px inset #eee; } footer span:first-child { flex:1; }
  @media (prefers-reduced-motion:reduce) { .spectrum i { height:2px !important; transition:none; } }

</style>
