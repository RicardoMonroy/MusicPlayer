<script lang="ts">
  import { player } from "$lib/player.svelte";
  const playing = $derived(player.status.index != null && !player.status.paused);
</script>

<div class="hero" style="--ink: {player.accent}"><div class="cover">{#if player.coverUrl}<img src={player.coverUrl} alt="Portada del álbum" />{:else}<span class="empty">♪</span>{/if}</div><div class="spectrum" aria-hidden="true">{#each Array(19) as _, i}<i style:height={`${playing ? Math.max(2, (player.spectrum[i] ?? 0)*100) : 2}%`}></i>{/each}</div></div>

<style>

  .hero { width:100%; height:100%; position:relative; box-sizing:border-box; overflow:hidden; }
  .hero :global(*) { box-sizing:border-box; }
  img { width:100%; height:100%; object-fit:cover; display:block; }
  .hero { padding:10px 18px; } .cover { width:244px; height:244px; overflow:hidden; border-radius:14px; background:var(--art-bg,#23232b); box-shadow:0 8px 15px #0005; display:grid; place-items:center; } .empty { font:80px serif; color:rgb(var(--ink)); }
  .spectrum { position:absolute; bottom:3px; left:33px; right:33px; height:17px; display:flex; gap:5px; align-items:end; } .spectrum i { flex:1; min-height:2px; border-radius:2px; background:rgb(var(--ink)); opacity:.65; transition:height .2s ease-out; }
  @media (prefers-reduced-motion:reduce) { .spectrum i { height:2px !important; transition:none; } }

</style>
