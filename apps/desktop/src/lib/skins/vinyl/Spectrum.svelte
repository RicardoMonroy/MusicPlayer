<script lang="ts">
  import { onMount } from "svelte";
  import { player } from "$lib/player.svelte";

  let levels = $state<number[]>(Array(19).fill(0));

  onMount(() => {
    const motion = window.matchMedia("(prefers-reduced-motion: reduce)");
    let frame = 0;
    let previous = performance.now();
    const animate = (now: number) => {
      const elapsed = Math.min(now - previous, 100);
      previous = now;
      const active = !motion.matches && !player.status.paused && player.status.index != null;
      const bands = active ? player.spectrum : [];
      const targets = Array.from({ length: 19 }, (_, i) => Math.min(1, Math.max(0, bands[i] ?? 0)));
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

<svg class="spectrum" viewBox="0 0 332 332" aria-hidden="true">
  {#each Array.from({ length: 38 }, (_, i) => i) as i}
    <line x1="166" y1="13" x2="166" y2={13 - levels[i < 19 ? i : 37 - i] * 9}
      transform="rotate({i * 360 / 38} 166 166)" />
  {/each}
</svg>

<style>
  .spectrum { position: absolute; left: 8px; top: 12px; width: 332px; height: 332px; pointer-events: none; }
  line { stroke: #d6ad72; stroke-width: 2; stroke-linecap: round; opacity: 0.65; }
  @media (prefers-reduced-motion: reduce) { .spectrum { display: none; } }
</style>
