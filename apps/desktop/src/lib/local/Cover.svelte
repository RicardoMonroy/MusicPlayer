<script lang="ts">
  import { thumbnail } from "$lib/player.svelte";
  let { uri, title }: { uri: string; title: string } = $props();
  let src = $state("");
  function observe(node: HTMLElement) {
    let active = true;
    const observer = new IntersectionObserver(entries => {
      if (!entries.some(e => e.isIntersecting)) return;
      observer.disconnect();
      thumbnail(uri).then(url => { if (active) src = url; }).catch(() => {});
    }, { rootMargin: "200px" });
    observer.observe(node);
    return { destroy() { active = false; observer.disconnect(); } };
  }
</script>
<div class="cover" use:observe>
  {#if src}<img src={src} alt="" onerror={() => src = ""} />{:else}<span aria-hidden="true">{title.trim().split(/\s+/).slice(0, 2).map(s => s[0]).join("").toLocaleUpperCase() || "♪"}</span>{/if}
</div>
<style>
  .cover { width: 100%; aspect-ratio: 1; display: grid; place-items: center; overflow: hidden; border-radius: var(--radius); background: var(--art-bg); color: var(--fg-dim); font-size: clamp(24px, 4vw, 64px); }
  img { width: 100%; height: 100%; object-fit: cover; }
</style>
