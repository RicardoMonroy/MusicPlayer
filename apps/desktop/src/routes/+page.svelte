<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let name = $state("");
  invoke<string>("app_name").then((n) => (name = n));
</script>

<main>
  <h1>{name}</h1>
  <p>M0 scaffold. Full view, compact skins and tray land in M1–M2.</p>
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: system-ui, sans-serif;
    background: #121212;
    color: #eee;
  }
  main {
    min-height: 100vh;
    display: grid;
    place-content: center;
    text-align: center;
  }
</style>
