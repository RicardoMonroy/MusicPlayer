<script lang="ts">
  import type { Collection } from "$lib/yt";

  let { items, onopen }: { items: Collection[]; onopen: (c: Collection) => void } = $props();
</script>

<div class="cards">
  {#each items as c (c.kind + c.id)}
    <button class="card" onclick={() => onopen(c)}>
      <div class="art">
        {#if c.cover}<img src={c.cover} alt="" loading="lazy" />{/if}
      </div>
      <span class="t">{c.title}</span>
      <span class="s">{c.subtitle}</span>
    </button>
  {/each}
</div>

<style>
  .cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 18px;
  }
  .card {
    background: none;
    border: 0;
    padding: 8px;
    border-radius: 10px;
    text-align: left;
    display: flex;
    flex-direction: column;
    gap: 4px;
    cursor: pointer;
    min-width: 0;
  }
  .card:hover {
    background: #1b1b21;
  }
  .art {
    aspect-ratio: 1;
    border-radius: 8px;
    overflow: hidden;
    background: #23232b;
    margin-bottom: 6px;
  }
  .art img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .t,
  .s {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .t {
    font-weight: 600;
    font-size: 14px;
  }
  .s {
    font-size: 12px;
    color: var(--fg-dim);
  }
</style>
