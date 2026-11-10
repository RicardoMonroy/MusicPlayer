<script lang="ts" module>
  import type { Shelf, SearchResults, ArtistPage } from "$lib/yt";
  import type { Track } from "$lib/player.svelte";
  // Home is slow-ish (~2 s) and changes little: keep it for the session.
  let homeCache: Shelf[] | null = null;

  /** An opened album/playlist (or "Me gusta"), or an artist page. */
  type Page =
    | { type: "list"; title: string; subtitle: string; tracks: Track[]; albumId?: string }
    | { type: "artist"; page: ArtistPage };
  // Module level, so leaving the view (Biblioteca, Cola) and coming back keeps
  // the search and the pages opened on top of it, like YouTube Music.
  let query = $state("");
  let results = $state<SearchResults | null>(null);
  let stack = $state<Page[]>([]);
</script>

<script lang="ts">
  import { open as pickFile } from "@tauri-apps/plugin-dialog";
  import { invoke } from "@tauri-apps/api/core";
  import { play } from "$lib/player.svelte";
  import TrackList from "$lib/TrackList.svelte";
  import Cards from "$lib/Cards.svelte";
  import HomeShelf from "$lib/HomeShelf.svelte";
  import * as yt from "$lib/yt";
  import { tick, untrack } from "svelte";
  import { nav, openArtist } from "$lib/nav.svelte";

  let busy = $state(false);
  let error = $state("");
  let account = $state(false);
  let lib = $state<yt.Library | null>(null);
  const top = $derived(stack.at(-1));
  let scroller: HTMLElement | undefined = $state();
  let allSongs = $state(false);
  let tab = $state<"inicio" | "biblioteca">("inicio");
  let shelves = $state<Shelf[] | null>(homeCache);
  let homeError = $state("");

  async function loadHome(force = false) {
    if (shelves && !force) return;
    homeError = "";
    try {
      shelves = homeCache = await yt.home();
    } catch (e) {
      homeError = String(e);
    }
  }
  $effect(() => {
    loadHome();
  });

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
    stack = [];
    allSongs = false;
    results = (await run(() => yt.search(query.trim()))) ?? null;
  }

  async function push(page: Page) {
    stack = [...stack, page];
    await tick();
    scroller?.scrollTo({ top: 0 });
  }
  const back = () => (stack = stack.slice(0, -1));

  async function openCard(c: yt.Collection) {
    if (c.kind === "artist") {
      const page = await run(() => yt.artist(c.id));
      if (page) push({ type: "artist", page });
      return;
    }
    const tracks = await run(() => yt.openCollection(c));
    if (tracks) push({ type: "list", title: c.title, subtitle: c.subtitle, tracks, albumId: c.kind === "album" ? c.id : undefined });
  }

  // Artist/album links elsewhere in the app (now playing, queue, track lists).
  $effect(() => {
    const t = nav.yt;
    if (!t) return;
    nav.yt = null;
    untrack(() => openCard({ kind: t.kind, id: t.id, title: t.title, subtitle: "", cover: null }));
  });

  async function openLiked() {
    const tracks = await run(yt.liked);
    if (tracks) push({ type: "list", title: "Me gusta", subtitle: "Tus canciones favoritas", tracks });
  }

  let copied = $state(false);
  async function copyAlbumUrl(albumId: string) {
    if (await run(() => invoke("copy_album_url", { albumId })) === undefined) return;
    copied = true;
    setTimeout(() => (copied = false), 2000);
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
      <input class="field" placeholder="Busca canciones, artistas, álbumes, playlists" bind:value={query} />
    </form>
    {#if account}
      <button class="pill ghost" onclick={disconnect}>Desconectar cuenta</button>
    {:else}
      <button class="pill" onclick={connect}>Conectar cuenta</button>
    {/if}
  </header>

  {#if error}<p class="error">{error}</p>{/if}

  <div class="scroll" bind:this={scroller}>
    {#if top?.type === "list"}
      {@const o = top}
      {@const first = o.tracks[0]}
      <div class="opened">
        <button class="pill ghost" onclick={back}>← Volver</button>
        {#if o.albumId && first?.cover}<img class="album-art" src={first.cover} alt="" />{/if}
        <div>
          <h2>{o.title}</h2>
          <p>
            {#if o.albumId && first}
              <button class="nav-link" disabled={!first.artist_id} onclick={() => openArtist(first)}>{first.artist}</button>
            {:else}
              {o.subtitle}
            {/if}
            · {o.tracks.length} canciones
          </p>
        </div>
        <button class="pill" onclick={() => play(o.tracks, 0)}>▶ Reproducir</button>
        {#if o.albumId}
          {@const id = o.albumId}
          <button class="pill ghost" onclick={() => copyAlbumUrl(id)}>{copied ? "¡Copiada!" : "Copiar URL"}</button>
        {/if}
      </div>
      <TrackList tracks={o.tracks} onplay={(i) => play(o.tracks, i)} />
    {:else if top?.type === "artist"}
      {@const a = top.page}
      <div class="artist-head" style:--cover={a.cover ? `url(${a.cover})` : "none"}>
        <button class="pill ghost" onclick={back}>← Volver</button>
        <div class="avatar">{#if a.cover}<img src={a.cover} alt="" />{/if}</div>
        <div class="who">
          <h2>{a.name}</h2>
          <p>{a.tracks.length} canciones populares · {a.albums.length} álbumes</p>
        </div>
        <button class="pill" onclick={() => play(a.tracks, 0)} disabled={!a.tracks.length}>▶ Reproducir</button>
      </div>
      {#if a.tracks.length}
        <h3>Populares</h3>
        <TrackList tracks={a.tracks} onplay={(i) => play(a.tracks, i)} />
      {/if}
      {#if a.albums.length}
        <h3>Álbumes</h3>
        <Cards items={a.albums} onopen={openCard} />
      {/if}
      {#if a.playlists.length}
        <h3>Playlists</h3>
        <Cards items={a.playlists} onopen={openCard} />
      {/if}
    {:else if results}
      {@const r = results}
      {#if r.top}
        {@const top = r.top}
        <div class="top-row">
          <section class="top">
            <h3>Resultado principal</h3>
            {#if top.type === "artist"}
              <button class="top-card" onclick={() => openCard(top.item)}>
                <div class="avatar">{#if top.item.cover}<img src={top.item.cover} alt="" />{/if}</div>
                <strong>{top.item.title}</strong>
                <span>{top.item.subtitle}</span>
              </button>
            {:else}
              <button class="top-card" onclick={() => play([top.item], 0)}>
                <div class="square">{#if top.item.cover}<img src={top.item.cover} alt="" />{/if}</div>
                <strong>{top.item.title}</strong>
                <span>Canción · {top.item.artist}</span>
              </button>
            {/if}
          </section>
          {#if r.tracks.length}
            <section class="top-songs">
              <h3>Canciones</h3>
              <TrackList tracks={r.tracks.slice(0, 4)} onplay={(i) => play(r.tracks, i)} />
            </section>
          {/if}
        </div>
      {/if}
      {#if r.tracks.length > 4}
        <h3>
          Más canciones
          <button class="link" onclick={() => (allSongs = !allSongs)}>
            {allSongs ? "Ver menos" : `Ver todas (${r.tracks.length})`}
          </button>
        </h3>
        {@const rest = r.tracks.slice(4, allSongs ? undefined : 8)}
        <TrackList tracks={rest} offset={4} onplay={(i) => play(r.tracks, i + 4)} />
      {/if}
      {#if r.artists.length}
        <h3>Artistas</h3>
        <Cards items={r.artists} onopen={openCard} />
      {/if}
      {#if r.albums.length}
        <h3>Álbumes</h3>
        <Cards items={r.albums} onopen={openCard} />
      {/if}
      {#if r.playlists.length}
        <h3>Playlists</h3>
        <Cards items={r.playlists} onopen={openCard} />
      {/if}
      {#if r.videos.length}
        <h3>Videos</h3>
        <TrackList tracks={r.videos.slice(0, 8)} onplay={(i) => play(r.videos, i)} />
      {/if}
    {:else}
      <div class="tabs" role="tablist">
        <button role="tab" aria-selected={tab === "inicio"} class:on={tab === "inicio"} onclick={() => (tab = "inicio")}>Inicio</button>
        <button role="tab" aria-selected={tab === "biblioteca"} class:on={tab === "biblioteca"} onclick={() => (tab = "biblioteca")}>Mi biblioteca</button>
        {#if tab === "inicio"}
          <button class="reload" onclick={() => loadHome(true)} title="Recargar recomendaciones">↻</button>
        {/if}
      </div>
      {#if tab === "inicio"}
        {#if homeError}
          <p class="error">{homeError}</p>
        {:else if !shelves}
          <div class="empty"><p>Cargando tus recomendaciones…</p></div>
        {:else}
          {#each shelves as shelf (shelf.title)}
            <HomeShelf {shelf} onplay={play} onopen={openCard} />
          {/each}
        {/if}
      {:else}
        {#if account && lib}
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
      {/if}
    {/if}
  </div>
</section>

<style>
  .tabs {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 4px 0 18px;
  }
  .tabs button {
    border: 1px solid var(--border);
    border-radius: var(--radius-pill);
    background: var(--surface);
    color: var(--fg-dim);
    padding: 6px 16px;
    font-weight: 600;
    cursor: pointer;
  }
  .tabs button.on {
    background: rgb(var(--accent));
    border-color: transparent;
    color: var(--on-accent);
  }
  .tabs .reload {
    margin-left: auto;
    padding: 6px 12px;
  }
  .top-row {
    display: grid;
    grid-template-columns: minmax(240px, 340px) 1fr;
    gap: 24px;
    align-items: start;
  }
  .top-card {
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
    padding: 18px;
    border: 0;
    border-radius: var(--radius-lg);
    background: var(--surface);
    color: inherit;
    text-align: left;
    cursor: pointer;
    transition: background 0.15s;
  }
  .top-card:hover {
    background: var(--surface-2);
  }
  .top-card strong {
    font-size: 24px;
    margin-top: 8px;
  }
  .top-card span {
    color: var(--fg-dim);
    font-size: 13px;
  }
  .avatar,
  .square {
    width: 104px;
    height: 104px;
    overflow: hidden;
    background: var(--art-bg);
    flex: none;
  }
  .avatar {
    border-radius: 50%;
  }
  .square {
    border-radius: var(--radius);
  }
  .avatar img,
  .square img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .link {
    background: none;
    border: 0;
    color: var(--accent-c);
    font: inherit;
    font-size: 13px;
    margin-left: 10px;
    cursor: pointer;
  }
  .artist-head {
    position: relative;
    display: flex;
    align-items: center;
    gap: 20px;
    padding: 24px;
    border-radius: var(--radius-lg);
    overflow: hidden;
    background: var(--surface);
    margin-bottom: 8px;
  }
  .artist-head::before {
    content: "";
    position: absolute;
    inset: -40px;
    background: var(--cover) center / cover;
    filter: blur(40px) brightness(0.45);
  }
  .artist-head > * {
    position: relative;
  }
  .artist-head .avatar {
    width: 132px;
    height: 132px;
    box-shadow: 0 12px 30px rgba(0, 0, 0, 0.5);
  }
  .who {
    flex: 1;
    min-width: 0;
  }
  .who h2 {
    margin: 0;
    font-size: 34px;
  }
  .who p {
    margin: 6px 0 0;
    color: var(--fg-dim);
  }
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
    flex-wrap: wrap;
    align-items: center;
    gap: 16px;
    margin-bottom: 12px;
  }
  .opened div {
    flex: 1;
    min-width: 0;
  }
  .album-art {
    width: 96px;
    height: 96px;
    object-fit: cover;
    border-radius: var(--radius);
    box-shadow: var(--raised);
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
