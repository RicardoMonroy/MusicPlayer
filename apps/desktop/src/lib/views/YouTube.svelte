<script lang="ts">
  import { open as pickFile } from "@tauri-apps/plugin-dialog";
  import { play, type Track } from "$lib/player.svelte";
  import TrackList from "$lib/TrackList.svelte";
  import Cards from "$lib/Cards.svelte";
  import * as yt from "$lib/yt";

  let query = $state("");
  let busy = $state(false);
  let error = $state("");
  let account = $state(false);
  let results = $state<yt.SearchResults | null>(null);
  let lib = $state<yt.Library | null>(null);
  /** An opened album/playlist (or "Me gusta"), shown over the results. */
  let opened = $state<{ title: string; subtitle: string; tracks: Track[] } | null>(null);

  $effect(() => {
    yt.loggedIn().then((ok) => {
      account = ok;
      if (ok) loadLibrary();
    });
  });

  async function run<T>(f: () => Promise<T>): Promise<T | undefined> {
    busy = true;
    error = "";
    try {
      return await f();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function doSearch(e: SubmitEvent) {
    e.preventDefault();
    if (!query.trim()) return;
    opened = null;
    results = (await run(() => yt.search(query.trim()))) ?? null;
  }

  async function openCard(c: yt.Collection) {
    const tracks = await run(() => yt.openCollection(c));
    if (tracks) opened = { title: c.title, subtitle: c.subtitle, tracks };
  }

  async function openLiked() {
    const tracks = await run(yt.liked);
    if (tracks) opened = { title: "Me gusta", subtitle: "Tus canciones favoritas", tracks };
  }

  async function loadLibrary() {
    lib = (await run(yt.library)) ?? null;
  }

  async function connect() {
    const path = await pickFile({ filters: [{ name: "cookies.txt", extensions: ["txt"] }] });
    if (typeof path !== "string") return;
    await run(async () => {
      await yt.login(path);
      account = await yt.loggedIn();
      if (!account) throw new Error("Las cookies no son válidas o expiraron.");
    });
    if (account) loadLibrary();
  }

  async function disconnect() {
    await run(yt.logout);
    account = false;
    lib = null;
  }
</script>

<section class="view">
  <header>
    <h1>YouTube Music</h1>
    <span class="count">{busy ? "Cargando…" : ""}</span>
    <form onsubmit={doSearch}>
      <input class="field" placeholder="Busca canciones, álbumes, playlists" bind:value={query} />
    </form>
    {#if account}
      <button class="pill ghost" onclick={disconnect}>Desconectar cuenta</button>
    {:else}
      <button class="pill" onclick={connect}>Conectar cuenta</button>
    {/if}
  </header>

  {#if error}<p class="error">{error}</p>{/if}

  <div class="scroll">
    {#if opened}
      <div class="opened">
        <button class="pill ghost" onclick={() => (opened = null)}>← Volver</button>
        <div>
          <h2>{opened.title}</h2>
          <p>{opened.subtitle} · {opened.tracks.length} canciones</p>
        </div>
        <button class="pill" onclick={() => opened && play(opened.tracks, 0)}>▶ Reproducir</button>
      </div>
      <TrackList tracks={opened.tracks} onplay={(i) => opened && play(opened.tracks, i)} />
    {:else if results}
      {@const r = results}
      {#if r.tracks.length}
        <h3>Canciones</h3>
        <TrackList tracks={r.tracks} onplay={(i) => play(r.tracks, i)} />
      {/if}
      {#if r.albums.length}
        <h3>Álbumes</h3>
        <Cards items={r.albums} onopen={openCard} />
      {/if}
      {#if r.playlists.length}
        <h3>Playlists</h3>
        <Cards items={r.playlists} onopen={openCard} />
      {/if}
    {:else if account && lib}
      <h3>Tu biblioteca</h3>
      <Cards
        items={[{ kind: "playlist", id: "", title: "Me gusta", subtitle: "Canciones que te gustan", cover: null }]}
        onopen={openLiked}
      />
      {#if lib.playlists.length}
        <h3>Playlists</h3>
        <Cards items={lib.playlists} onopen={openCard} />
      {/if}
      {#if lib.albums.length}
        <h3>Álbumes guardados</h3>
        <Cards items={lib.albums} onopen={openCard} />
      {/if}
    {:else if !account}
      <div class="empty">
        <p>Busca cualquier canción arriba, no necesitas cuenta.</p>
        <p>
          Para ver tu biblioteca, exporta las cookies de music.youtube.com como
          <b>cookies.txt</b> (extensión “Get cookies.txt LOCALLY”, en una ventana privada) y pulsa
          <b>Conectar cuenta</b>.
        </p>
        <p class="small">Las cookies se guardan solo en este equipo.</p>
      </div>
    {/if}
  </div>
</section>

<style>
  form {
    display: contents;
  }
  h3 {
    margin: 22px 0 10px;
    font-size: 17px;
  }
  .error {
    color: #ff8a8a;
    margin: 0 0 10px;
    font-size: 13px;
  }
  .opened {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 12px;
  }
  .opened div {
    flex: 1;
    min-width: 0;
  }
  .opened h2 {
    margin: 0;
    font-size: 22px;
  }
  .opened p {
    margin: 2px 0 0;
    color: var(--fg-dim);
    font-size: 13px;
  }
  .empty p {
    max-width: 460px;
    margin: 6px 0;
    line-height: 1.5;
  }
  .small {
    font-size: 12px;
  }
</style>
