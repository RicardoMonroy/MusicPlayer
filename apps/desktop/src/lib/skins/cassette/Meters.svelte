<script lang="ts">
  import { onMount } from "svelte";
  import { player } from "$lib/player.svelte";

  let levels = $state<number[]>(Array(2).fill(0));

  onMount(() => {
    const motion = window.matchMedia("(prefers-reduced-motion: reduce)");
    let frame = 0;
    let previous = performance.now();
    const animate = (now: number) => {
      const elapsed = Math.min(now - previous, 100);
      previous = now;
      const active = !motion.matches && !player.status.paused && player.status.index != null;
      const bands = active ? player.spectrum : [];
      // Frequency groups, not stereo channels: 10 low bands and 9 high bands.
      const targets = [bands.slice(0, 10), bands.slice(10, 19)].map((group) =>
        group.length ? Math.sqrt(group.reduce((sum, value) => sum + Math.min(1, Math.max(0, value)) ** 2, 0) / group.length) : 0,
      );
      levels = levels.map((level, i) => {
        const target = targets[i];
        const time = target > level ? 60 : 300;
        return level + (target - level) * (1 - Math.exp(-elapsed / time));
      });
      frame = requestAnimationFrame(animate);
    };
    frame = requestAnimationFrame(animate);
    return () => cancelAnimationFrame(frame);
  });
</script>

<div class="meters" aria-hidden="true">
  {#each levels as level, i}
    <svg viewBox="0 0 140 46">
      <rect x="1" y="1" width="138" height="44" rx="4" fill="#e9dfc6" stroke="#141311" stroke-width="2" />
      <path d="M 23 29 Q 70 -7 117 29" fill="none" stroke="#615c4d" stroke-width="1" />
      <path d="M 96 16 Q 108 20 117 29" fill="none" stroke="#b54432" stroke-width="3" />
      {#each [-54, -36, -18, 0, 18, 36, 54] as angle}
        <line x1="70" y1="5" x2="70" y2="10" transform="rotate({angle} 70 65)" stroke="#615c4d" />
      {/each}
      <text x="70" y="29" text-anchor="middle">VU</text>
      <text class="range" x="9" y="39">{i === 0 ? "GRAVES" : "AGUDOS"}</text>
      <line x1="70" y1="62" x2="70" y2="9" transform="rotate({-54 + level * 108} 70 62)" stroke="#a63c2e" stroke-width="1.5" />
    </svg>
  {/each}
</div>

<style>
  .meters { display: flex; justify-content: center; gap: 14px; margin-top: 5px; pointer-events: none; }
  svg { width: 140px; height: 46px; overflow: hidden; filter: drop-shadow(0 2px 2px #0008); }
  text { fill: #4d473b; font: bold 9px "Courier New", monospace; }
  .range { font-size: 5px; letter-spacing: 0.7px; }
  @media (prefers-reduced-motion: reduce) { .meters { visibility: hidden; } }
</style>
