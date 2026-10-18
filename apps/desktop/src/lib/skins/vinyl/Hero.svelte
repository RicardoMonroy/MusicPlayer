<script lang="ts">
  import { player } from "$lib/player.svelte";
  const playing = $derived(player.status.index != null && !player.status.paused);
  const duration = $derived(Math.max(0, player.status.duration || 0));
  const position = $derived(Math.max(0, Math.min(duration, player.status.position || 0)));
  const progress = $derived(duration > 0 ? position / duration : 0);
  const armAngle = $derived(player.track ? 12 + 12 * progress : 0);
</script>

<div class="hero" style="--ink: {player.accent}">
  <div class="platter"><div class="record" style:animation-play-state={playing ? 'running' : 'paused'}><div class="label">{#if player.coverUrl}<img src={player.coverUrl} alt="Portada del álbum" />{:else}<span class="empty">♪</span>{/if}</div></div></div>
  <div class="arm" style:rotate={`${armAngle}deg`}><div class="rod"></div><div class="head"></div><div class="pivot"></div></div>
  <span class="speed">33⅓ · ESTÉREO</span>
</div>

<style>

  .hero { width:100%; height:100%; position:relative; box-sizing:border-box; overflow:hidden; }
  .hero :global(*) { box-sizing:border-box; }
  img { width:100%; height:100%; object-fit:cover; display:block; }
  .platter { position:absolute; width:248px; height:248px; left:5px; top:15px; padding:5px; border-radius:50%; background:linear-gradient(130deg,#aaa,#343434,#999); box-shadow:0 5px 10px #0007; }
  .record { width:100%; height:100%; border-radius:50%; display:grid; place-items:center; background:repeating-radial-gradient(#111 0 1px,#242424 1px 2px,#111 2px 4px); animation:spin 1.8s linear infinite; }
  .label { position:relative; width:79px; height:79px; border-radius:50%; overflow:hidden; display:grid; place-items:center; background:rgb(var(--ink)); box-shadow:0 0 0 3px #090909; }
  .label::after { content:""; position:absolute; width:7px; height:7px; border-radius:50%; background:#ccc; box-shadow:0 0 0 2px #222; }
  .empty { color:#202020; font:30px serif; }
  /* The light sheen stays still like a real room light; only the record turns. */
  .platter::after { content:""; position:absolute; inset:5px; border-radius:50%; pointer-events:none; background:conic-gradient(from 25deg,#0000 0 35deg,#ffffff20 48deg,#0000 64deg 210deg,#ffffff16 230deg,#0000 250deg); }
  .arm { position:absolute; left:254px; top:26px; width:0; height:0; transition:rotate .8s ease; }
  .pivot { position:absolute; left:-12px; top:-12px; width:24px; height:24px; border-radius:50%; background:radial-gradient(circle,#555 0 15%,#c6c6c6 20%,#777 65%,#333 70%); box-shadow:0 3px 5px #0008; }
  .rod { position:absolute; left:-3px; width:6px; height:177px; background:linear-gradient(90deg,#777,#eee,#999); transform-origin:top; rotate:8deg; border-radius:4px; box-shadow:3px 2px 4px #0007; }
  .head { position:absolute; left:-34px; top:169px; width:17px; height:29px; background:linear-gradient(90deg,#252525,#484848); rotate:8deg; border-bottom:3px solid #d4b57e; }
  .speed { position:absolute; bottom:1px; right:14px; font:8px monospace; letter-spacing:2px; color:#c2ad98; }
  @keyframes spin { to { rotate:360deg; } }
  @media (prefers-reduced-motion:reduce) { .record { animation:none; } .arm { transition:none; } }

</style>
