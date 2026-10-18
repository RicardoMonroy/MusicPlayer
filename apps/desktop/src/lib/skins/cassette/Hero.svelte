<script lang="ts">
  import { player } from "$lib/player.svelte";
  const playing = $derived(player.status.index != null && !player.status.paused);
  const duration = $derived(Math.max(0, player.status.duration || 0));
  const position = $derived(Math.max(0, Math.min(duration, player.status.position || 0)));
  const progress = $derived(duration > 0 ? position / duration : 0);
</script>

<div class="hero" style="--ink: {player.accent}"><div class="cassette">
  {#each [0,1,2,3] as screw}<i class="screw s{screw}"></i>{/each}
  <div class="label"><div class="print">{#if player.coverUrl}<img src={player.coverUrl} alt="Portada del álbum" />{:else}<span class="empty">♪</span>{/if}<b>{player.track?.title || 'Inserta un casete'}</b></div>
  <div class="window">{#each [1-progress, progress] as fill}<div class="slot"><div class="tape" style:width={`${30+fill*30}px`} style:height={`${30+fill*30}px`}></div><div class="reel" style:animation-play-state={playing ? 'running' : 'paused'}>{#each [0,60,120] as angle}<i style:rotate={`${angle}deg`}></i>{/each}</div></div>{/each}<div class="ribbon" style:animation-play-state={playing ? 'running' : 'paused'}></div></div>
  <footer><b>A</b><span>ESTÉREO · CINTA NORMAL</span><span>60</span></footer></div>
  <div class="bridge"><i></i><i></i><i></i><i></i></div>
</div></div>

<style>

  .hero { width:100%; height:100%; position:relative; box-sizing:border-box; overflow:hidden; }
  .hero :global(*) { box-sizing:border-box; }
  img { width:100%; height:100%; object-fit:cover; display:block; }
  .hero { display:grid; place-items:center; font:10px 'Courier New',monospace; }
  .cassette { position:relative; width:270px; height:177px; padding:13px 15px 30px; border:1px solid #66625c; border-radius:10px; background:linear-gradient(150deg,#484647,#242326); box-shadow:0 9px 12px #0006,inset 0 1px #ffffff30; }
  .label { height:119px; border-radius:5px; padding:5px 9px; background:#e7dfc9; color:#2c271e; border-bottom:7px solid rgb(var(--ink)); }
  .print { height:28px; display:flex; align-items:center; gap:7px; border-bottom:2px solid #b67e43; overflow:hidden; }
  .print img { width:48px; height:25px; } .print b { white-space:nowrap; overflow:hidden; text-overflow:ellipsis; font-size:10px; } .empty { font-size:20px; }
  .window { height:61px; margin:4px 0; padding:0 8px; position:relative; border-radius:28px; background:#171715; box-shadow:inset 0 2px 5px #000; display:flex; align-items:center; justify-content:space-between; overflow:hidden; }
  .slot { width:62px; height:61px; display:grid; place-items:center; position:relative; z-index:1; }
  .tape { position:absolute; border-radius:50%; background:repeating-radial-gradient(#513b29 0 1px,#34271c 1px 2px); transition:width .4s,height .4s; }
  .reel { width:25px; height:25px; border-radius:50%; background:#ded7c5; position:relative; animation:turn 2s linear infinite; }
  .reel i { position:absolute; left:11px; top:2px; width:3px; height:21px; border-radius:2px; background:#25221b; }
  .ribbon { position:absolute; left:38px; right:38px; bottom:9px; height:3px; background:repeating-linear-gradient(90deg,#725337 0 5px,#38281c 5px 10px); animation:travel .5s linear infinite; }
  footer { display:flex; align-items:center; justify-content:space-between; font-size:7px; } footer b { font-size:15px; }
  .screw { position:absolute; width:6px; height:6px; border-radius:50%; background:linear-gradient(45deg,#888 40%,#222 42% 55%,#888 57%); } .s0 { left:5px; top:5px; } .s1 { right:5px; top:5px; } .s2 { left:5px; bottom:5px; } .s3 { right:5px; bottom:5px; }
  .bridge { position:absolute; bottom:0; left:60px; width:150px; height:24px; background:linear-gradient(#414042,#29282a); clip-path:polygon(10% 0,90% 0,100% 100%,0 100%); display:flex; justify-content:space-around; align-items:center; }
  .bridge i { width:9px; height:9px; border-radius:50%; background:#141313; border:1px solid #65615b; }
  @keyframes turn { to { rotate:-360deg; } } @keyframes travel { to { background-position:10px 0; } }
  @media (prefers-reduced-motion:reduce) { .reel,.ribbon { animation:none; } .tape { transition:none; } }

</style>
