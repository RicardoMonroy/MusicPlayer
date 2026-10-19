<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { player, scan, savedFolder, play } from "$lib/player.svelte";

  const plural = (n: number, one: string, many: string) => `${n} ${n === 1 ? one : many}`;

  /** Album length as "48 min" or "1 h 58 min". */
  const longTime = (s: number) => {
    const m = Math.round(s / 60);
    return m < 60 ? `${m} min` : `${Math.floor(m / 60)} h ${m % 60} min`;
  };
  import TrackList from "$lib/local/Tracks.svelte";
  import AlbumGrid from "$lib/local/AlbumGrid.svelte";
  import Cover from "$lib/local/Cover.svelte";
  import { compareAlbums, groupAlbums, compareValues, buildLibrary, partsFromId, shuffled, trackSearch, type Album } from "$lib/local/library";
  import { librarySession as state, type Tab } from "$lib/local/session.svelte";

  const tabs: Tab[] = ["Álbumes", "Artistas", "Carpetas", "Canciones"];
  const library = $derived(buildLibrary(player.tracks, state.root));
  const query = $derived(state.queries[state.tab].trim().toLocaleLowerCase());
  const navigation = $derived(state.tab === "Artistas" ? state.artists : state.albums);
  const album = $derived((state.tab === "Álbumes" || state.tab === "Artistas") ? library.albums.get(navigation.album) : undefined);
  const albumDiscs = $derived(album?.discs.map(disc => ({ ...disc, tracks: disc.tracks.filter(t => !query || trackSearch(t).includes(query)) })).filter(disc => disc.tracks.length) ?? []);
  const artist = $derived((state.tab === "Álbumes" || state.tab === "Artistas") ? library.artists.get(navigation.artist) : undefined);
  const shownAlbums = $derived([...(artist?.albums ?? library.albums.values())].filter(a => !query || a.search.includes(query)));
  const genres = $derived.by(() => {
    const counts = new Map<string, number>([["", 0]]);
    for (const a of library.albums.values()) counts.set(a.genre ?? "", (counts.get(a.genre ?? "") ?? 0) + 1);
    return [...counts].sort((a, b) => compareValues(a[0], b[0]));
  });
  const albumGroups = $derived(groupAlbums(shownAlbums
    .filter(a => state.genre === null || (a.genre ?? "") === state.genre)
    .sort((a, b) => compareAlbums(a, b, state.albumSort, state.yearAscending)), state.albumSort));
  const shownArtists = $derived([...library.artists.values()].filter(a => !query || a.search.includes(query)));
  const shownTracks = $derived(player.tracks.filter(t => !query || trackSearch(t).includes(query)));
  const folder = $derived(library.folders.get(state.folder) ?? library.root);
  const children = $derived([...folder.children.values()].filter(f => !query || `${f.name.toLocaleLowerCase()} ${f.search}`.includes(query)).sort((a, b) => a.name.localeCompare(b.name)));
  const directTracks = $derived(folder.direct.filter(t => !query || trackSearch(t).includes(query)));
  const crumbs = $derived(partsFromId(folder.id).map((name, i, path) => ({ name, id: JSON.stringify(path.slice(0, i + 1)) })));
  const busy = $derived(state.busy || player.scan !== null);
  const progressText = $derived(player.scan?.total ? `Escaneando tu biblioteca… ${player.scan.done} / ${player.scan.total}` : "Buscando archivos…");

  onMount(() => {
    if (state.initialized) return;
    state.initialized = true;
    savedFolder().then(folder => {
      if (!folder) return;
      state.root = folder;
      if (!player.tracks.length && !player.scan && !state.busy) void rescan(folder);
    }).catch(e => { state.error = `No se pudo recuperar la carpeta: ${String(e)}`; });
  });

  async function rescan(path: string) {
    state.busy = true;
    state.error = "";
    try {
      await scan(path);
      state.root = path;
      state.folder = "";
      state.albums = { album: "", artist: "" };
      state.artists = { album: "", artist: "" };
    } catch (e) {
      state.error = `No se pudo escanear la carpeta: ${String(e)}`;
    } finally {
      state.busy = false;
    }
  }

  async function pickFolder() {
    try {
      const path = await open({ directory: true });
      if (typeof path === "string") await rescan(path);
    } catch (e) {
      state.error = `No se pudo abrir la carpeta: ${String(e)}`;
    }
  }

  function openAlbum(a: Album) { navigation.album = a.id; }
  function openArtist(name: string) { navigation.album = ""; navigation.artist = name; }
</script>

<section class="view library">
  <header>
    <div class="heading"><h1>Biblioteca</h1><span class="count">{plural(library.albums.size, "álbum", "álbumes")} · {plural(player.tracks.length, "canción", "canciones")}</span></div>
    <button class="pill" onclick={pickFolder} disabled={busy}>Elegir carpeta</button>
    {#if busy && player.tracks.length}
      <div class="inline-progress" role="status"><span class="spinner" aria-hidden="true"></span>{progressText}
        <progress max={player.scan?.total || 1} value={player.scan?.total ? player.scan.done : undefined} aria-label="Progreso del escaneo"></progress>
      </div>
    {/if}
  </header>
  <div class="toolbar">
    <nav class="segments" aria-label="Ver biblioteca por">
      {#each tabs as tab}<button class:selected={state.tab === tab} aria-pressed={state.tab === tab} onclick={() => state.tab = tab}>{tab}</button>{/each}
    </nav>
    <input class="field" type="search" aria-label={`Buscar en ${state.tab}`} placeholder="Buscar título, artista o álbum" bind:value={state.queries[state.tab]} />
  </div>
  {#if state.tab === "Álbumes" && !album && !artist}
    <div class="album-toolbar">
      <label>Ordenar por <select bind:value={state.albumSort}>
        <option value="artist">Artista</option><option value="year">Año</option>
        <option value="title">Alfabético</option><option value="genre">Género</option>
      </select></label>
      {#if state.albumSort === "year"}<button class="pill ghost" onclick={() => state.yearAscending = !state.yearAscending} aria-label="Cambiar orden del año">{state.yearAscending ? "Más antiguos primero ▲" : "Más recientes primero ▼"}</button>{/if}
      <label>Género <select bind:value={state.genre}>
        <option value={null}>Todos los géneros</option>
        {#each genres as [genre, count]}<option value={genre}>{genre || "Sin género"} ({count})</option>{/each}
      </select></label>
    </div>
  {/if}
  {#if state.error}<p class="error" role="alert">{state.error}</p>{/if}

  {#if !player.tracks.length && busy}
    <div class="scan-block" role="status">
      <span class="spinner large" aria-hidden="true"></span>
      <h2>Escaneando tu biblioteca…</h2>
      <p>{player.scan?.total ? `${player.scan.done} / ${player.scan.total}` : "Buscando archivos…"}</p>
      <progress max={player.scan?.total || 1} value={player.scan?.total ? player.scan.done : undefined} aria-label="Progreso del escaneo"></progress>
      <p>La primera lectura puede tardar unos minutos.</p>
    </div>
  {:else if !player.tracks.length}
    <div class="empty"><p>Aún no hay música.</p><button class="pill" onclick={pickFolder}>Elige tu carpeta de música</button></div>
  {:else}
    <div class="scroll">
      {#if album}
        {@const a = album}
        <button class="pill ghost back" onclick={() => navigation.album = ""}>← Volver</button>
        <div class="detail-head">
          <div class="hero-cover">{#key a.id}<Cover uri={a.tracks[0].uri} title={a.title} />{/key}</div>
          <div class="detail-info">
            <p class="eyebrow">ÁLBUM</p><h2>{a.title}</h2>
            <button class="artist-link" onclick={() => openArtist(a.artist)}>{a.artist}</button>
            <p>{a.year ?? "Sin año"} · {a.tracks.length} canciones · {longTime(a.duration)}</p>
            <div class="actions"><button class="pill" onclick={() => play(a.tracks, 0)}>▶ Reproducir álbum</button><button class="pill ghost" onclick={() => play(shuffled(a.tracks), 0)}>Aleatorio</button></div>
          </div>
        </div>
        {#each albumDiscs as disc (disc.number)}
          {#if a.discs.length > 1}<h3>Disco {disc.number}</h3>{/if}
          <TrackList tracks={disc.tracks} offset={disc.offset} onplay={i => play(a.tracks, a.tracks.indexOf(disc.tracks[i]))} />
        {/each}
        {#if !albumDiscs.length}<p class="no-results">No hay canciones que coincidan.</p>{/if}
      {:else if artist}
        {@const a = artist}
        <button class="pill ghost back" onclick={() => navigation.artist = ""}>← Volver</button>
        <div class="detail-head">
          <div class="artist-avatar">{#key a.name}<Cover uri={a.albums[0].tracks[0].uri} title={a.name} />{/key}</div>
          <div class="detail-info"><p class="eyebrow">ARTISTA</p><h2>{a.name}</h2><p>{plural(a.albums.length, "álbum", "álbumes")} · {plural(a.tracks.length, "canción", "canciones")}</p><button class="pill" onclick={() => play(a.tracks, 0)}>▶ Reproducir todo</button></div>
        </div>
        <h3>Álbumes</h3>
        {#if shownAlbums.length}<AlbumGrid albums={shownAlbums} onopen={openAlbum} />{:else}<p class="no-results">Sin resultados para «{state.queries[state.tab]}».</p>{/if}
      {:else if state.tab === "Álbumes"}
        {#each albumGroups as group (group.name)}
          {#if group.name}<h3>{group.name}</h3>{/if}
          <AlbumGrid albums={group.albums} onopen={openAlbum} />
        {:else}<p class="no-results">No hay álbumes que coincidan.</p>{/each}
      {:else if state.tab === "Artistas"}
        <div class="artists">
          {#each shownArtists as a (a.name)}
            <button class="artist-card" onclick={() => openArtist(a.name)}>
              <div class="artist-avatar"><Cover uri={a.albums[0].tracks[0].uri} title={a.name} /></div>
              <strong>{a.name}</strong><span>{plural(a.albums.length, "álbum", "álbumes")}</span>
            </button>
          {/each}
        </div>
        {#if !shownArtists.length}<p class="no-results">No hay artistas que coincidan.</p>{/if}
      {:else if state.tab === "Carpetas"}
        <nav class="breadcrumbs" aria-label="Ruta de carpetas">
          <button aria-current={folder.id === "" ? "location" : undefined} onclick={() => state.folder = ""}>{library.root.name}</button>
          {#each crumbs as crumb}<span aria-hidden="true">/</span><button aria-current={crumb.id === folder.id ? "location" : undefined} onclick={() => state.folder = crumb.id}>{crumb.name}</button>{/each}
        </nav>
        <div class="folder-head"><h2>{folder.name}</h2><span>{folder.tracks.length} canciones</span><button class="pill" onclick={() => play(folder.tracks, 0)} disabled={!folder.tracks.length}>▶ Reproducir carpeta</button></div>
        <div class="folders">
          {#each children as child (child.id)}
            <button class="folder" onclick={() => state.folder = child.id}>
              <span class="folder-art">{#if child.album}<Cover uri={child.album.tracks[0].uri} title={child.album.title} />{:else}<svg class="folder-icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M10 4H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-8z" /></svg>{/if}</span>
              <span class="folder-info"><strong>{child.name}</strong><span>{plural(child.tracks.length, "canción", "canciones")}</span></span><span aria-hidden="true">›</span>
            </button>
          {/each}
        </div>
        {#if directTracks.length}<h3>Canciones de esta carpeta</h3><TrackList tracks={directTracks} onplay={i => play(folder.tracks, folder.tracks.indexOf(directTracks[i]))} />{/if}
        {#if !children.length && !directTracks.length}<p class="no-results">No hay canciones ni carpetas que coincidan.</p>{/if}
      {:else}
        <TrackList sortable bind:sort={state.trackSort} tracks={shownTracks} onplay={i => play(shownTracks, i)} />
        {#if !shownTracks.length}<p class="no-results">No hay canciones que coincidan.</p>{/if}
      {/if}
    </div>
  {/if}
</section>

<style>
  .album-toolbar { display: flex; flex-wrap: wrap; align-items: center; gap: 12px; margin-bottom: 16px; }
  .album-toolbar label { display: flex; align-items: center; gap: 8px; }
  select { max-width: 100%; padding: 8px; color: var(--fg); background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius); font: inherit; }
  select:focus-visible { outline: 2px solid rgb(var(--accent)); }
  .library { color: var(--fg); font-family: var(--font); }
  header { flex-wrap: wrap; }
  .heading { display: flex; flex-wrap: wrap; align-items: baseline; gap: 12px; flex: 1; }
  .toolbar { display: flex; flex-wrap: wrap; justify-content: space-between; align-items: center; gap: 12px; padding: 4px 12px 20px 0; }
  .segments { display: flex; flex-wrap: wrap; gap: 3px; padding: 4px; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-pill); }
  button { font-family: var(--font); cursor: pointer; }
  .segments button { background: none; color: var(--fg-dim); border: 0; border-radius: var(--radius-pill); padding: 8px 14px; font-size: 13px; }
  .segments .selected { background: rgb(var(--accent)); color: var(--on-accent); box-shadow: var(--raised); }
  .field { max-width: 100%; width: 250px; }
  .field:focus { border-color: rgb(var(--accent)); }
  .pill { background: rgb(var(--accent)); }
  .pill.ghost { background: none; color: var(--fg); }
  button:disabled { opacity: .6; cursor: default; }
  .library :global(button:focus-visible), .field:focus-visible { outline: 2px solid rgb(var(--accent)); outline-offset: 3px; }
  .library :global(.active .title) { color: rgb(var(--accent)); }
  .scroll { flex: 1; min-height: 0; }
  .back { margin-bottom: 20px; }
  .detail-head { display: flex; flex-wrap: wrap; align-items: center; gap: 28px; padding: 24px; margin-bottom: 24px; background: var(--surface); border-radius: var(--radius-lg); }
  .hero-cover { width: clamp(140px, 23vw, 260px); flex: none; box-shadow: var(--raised); }
  .detail-info { flex: 1; min-width: 180px; }
  h2 { font-size: clamp(24px, 3vw, 38px); overflow-wrap: anywhere; margin: 4px 0 12px; }
  h3 { margin: 24px 0 12px; font-size: 17px; }
  .detail-info p, .eyebrow, .folder-head > span { color: var(--fg-dim); }
  .eyebrow { font-size: 12px; letter-spacing: .12em; }
  .artist-link { padding: 0; font: inherit; color: rgb(var(--accent)); border: 0; background: none; text-align: left; }
  .artist-link:hover { text-decoration: underline; }
  .actions { display: flex; flex-wrap: wrap; gap: 10px; margin-top: 24px; }
  .artists { display: grid; grid-template-columns: repeat(auto-fill, minmax(155px, 1fr)); gap: 18px; }
  .artist-card { display: flex; flex-direction: column; align-items: center; gap: 8px; min-width: 0; text-align: center; padding: 14px; background: none; border: 0; color: var(--fg); border-radius: var(--radius); }
  .artist-card:hover, .folder:hover { background: var(--surface-2); }
  .artist-card strong { max-width: 100%; overflow-wrap: anywhere; }
  .artist-card > span { color: var(--fg-dim); font-size: 13px; }
  .artist-avatar { width: 132px; max-width: 100%; border-radius: var(--radius-pill); overflow: hidden; flex: none; }
  .breadcrumbs { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; margin-bottom: 20px; }
  .breadcrumbs button { background: var(--surface); color: var(--fg); border: 1px solid var(--border); border-radius: var(--radius); padding: 6px 10px; overflow-wrap: anywhere; }
  .breadcrumbs [aria-current] { color: rgb(var(--accent)); }
  .folder-head { display: flex; flex-wrap: wrap; align-items: center; gap: 14px; margin-bottom: 18px; }
  .folder-head h2 { margin: 0; font-size: 24px; }
  .folder-head .pill { margin-left: auto; }
  .folders { display: grid; grid-template-columns: repeat(auto-fill, minmax(240px, 1fr)); gap: 8px; }
  .folder { display: flex; align-items: center; gap: 12px; min-width: 0; padding: 10px; text-align: left; color: var(--fg); border: 1px solid var(--border); background: var(--surface); border-radius: var(--radius); }
  .folder-art { width: 48px; flex: none; font-size: 32px; color: var(--fg-dim); }
  .folder-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 5px; }
  .folder-info strong { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .folder-info > span { color: var(--fg-dim); font-size: 12px; }
  .scan-block { flex: 1; display: flex; flex-direction: column; justify-content: center; align-items: center; padding: 36px; text-align: center; }
  .scan-block h2 { font-size: 24px; margin: 20px 0 0; }
  .scan-block p { color: var(--fg-dim); }
  progress { width: min(420px, 100%); height: 8px; accent-color: rgb(var(--accent)); border: 0; border-radius: var(--radius-pill); background: var(--surface-2); }
  progress::-webkit-progress-bar { background: var(--surface-2); border-radius: var(--radius-pill); }
  progress::-webkit-progress-value { background: rgb(var(--accent)); border-radius: var(--radius-pill); }
  progress::-moz-progress-bar { background: rgb(var(--accent)); }
  .inline-progress { width: 100%; display: flex; align-items: center; gap: 10px; color: var(--fg-dim); font-size: 13px; }
  .inline-progress progress { flex: 1; max-width: 180px; }
  .spinner { display: inline-block; width: 14px; height: 14px; flex: none; border: 2px solid var(--border); border-top-color: rgb(var(--accent)); border-radius: var(--radius-pill); animation: spin 1s linear infinite; }
  .spinner.large { width: 32px; height: 32px; border-width: 3px; }
  .error { color: var(--fg); background: var(--surface-2); border: 1px solid rgb(var(--accent)); border-radius: var(--radius); padding: 12px; }
  .no-results { padding: 24px; color: var(--fg-dim); }
  @keyframes spin { to { transform: rotate(360deg); } }
  @media (prefers-reduced-motion: reduce) { .spinner { animation: none; } }
  .folder-icon {
    width: 46%;
    fill: var(--fg-dim);
    opacity: 0.7;
  }
</style>
