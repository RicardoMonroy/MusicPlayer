<script lang="ts">
  import { player } from "$lib/player.svelte";
  const playing = $derived(player.status.index != null && !player.status.paused);
  const energy = $derived(playing ? player.spectrum.reduce((sum, n) => sum + n, 0) / 19 : 0);
</script>

<div class="hero" style="--ink: {player.accent}; --energy: {energy}"><div class="glow"></div><div class="cover">{#if player.coverUrl}<img src={player.coverUrl} alt="Portada del álbum" />{:else}<span class="empty">♪</span>{/if}</div><div class="reflection" aria-hidden="true">{#if player.coverUrl}<img src={player.coverUrl} alt="" />{/if}</div><div class="glass"></div></div>

<style>

  .hero { width:100%; height:100%; position:relative; box-sizing:border-box; overflow:hidden; }
  .hero :global(*) { box-sizing:border-box; }
  img { width:100%; height:100%; object-fit:cover; display:block; }
  .hero { isolation:isolate; } .glow { position:absolute; inset:21px 25px 65px; border-radius:40px; background:radial-gradient(ellipse,rgb(var(--ink)),#a693ff66 60%,transparent 75%); filter:blur(16px); opacity:calc(.25 + var(--energy)*.75); transform:scale(calc(1 + var(--energy)*.18)); transition:opacity .25s,transform .25s; }
  .cover { position:absolute; top:18px; left:32px; width:216px; height:216px; border:1px solid #ffffff30; border-radius:22px; overflow:hidden; display:grid; place-items:center; background:#222b48; box-shadow:0 12px 24px #0006; } .empty { font:72px serif; color:#ae97ff; }
  .reflection { position:absolute; top:244px; left:32px; width:216px; height:80px; overflow:hidden; transform:scaleY(-1); opacity:.3; filter:blur(3px); mask-image:linear-gradient(to top,#000,transparent 80%); } .reflection img { height:216px; position:absolute; bottom:0; }
  .glass { position:absolute; top:242px; left:21px; right:21px; height:25px; border-radius:50%; border-top:1px solid #ffffff20; background:linear-gradient(#ffffff0d,transparent); backdrop-filter:blur(4px); }
  @media (prefers-reduced-motion:reduce) { .glow { opacity:.35; transform:none; transition:none; } }

</style>
