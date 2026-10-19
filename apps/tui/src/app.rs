use std::{path::PathBuf, time::Instant};

use crate::youtube::{YoutubeRow, YoutubeView};
use mp_ytmusic::YtMusic;
use tokio::runtime::Runtime;

use mp_core::{
    Track,
    player::{Player, Status},
};
use ratatui::{layout::Rect, widgets::TableState};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Library,
    Albums,
    Artists,
    Folders,
    LocalSearch,
    Queue,
    Youtube,
    Playlists,
    Equalizer,
}

pub const TABS: [Tab; 9] = [
    Tab::Library,
    Tab::Albums,
    Tab::Artists,
    Tab::Folders,
    Tab::LocalSearch,
    Tab::Youtube,
    Tab::Playlists,
    Tab::Queue,
    Tab::Equalizer,
];

impl Tab {
    pub fn title(self) -> &'static str {
        match self {
            Tab::Library => "Canciones",
            Tab::Albums => "Álbumes",
            Tab::Artists => "Artistas",
            Tab::Folders => "Carpetas",
            Tab::LocalSearch => "Buscar en local",
            Tab::Queue => "Cola",
            Tab::Youtube => "Buscar",
            Tab::Playlists => "Mi biblioteca",
            Tab::Equalizer => "Ecualizador",
        }
    }
}

pub struct Album {
    pub artist: String,
    pub name: String,
    pub tracks: Vec<Track>,
}

/// Screen areas from the last frame, used to map mouse clicks.
#[derive(Default)]
pub struct Areas {
    pub eq_bars: Vec<Rect>,
    pub eq_presets: Vec<(usize, Rect)>,
    pub tabs: Vec<Rect>,
    pub search: Rect,
    pub top: Rect,
    pub switcher: Rect,
    pub columns: Vec<(usize, Rect)>,
    /// Inner area of the list table (rows start one line below, after the header).
    pub list: Rect,
    pub buttons: [Rect; 3],
    pub progress: Rect,
    pub volume: Rect,
}

// One per user action; boxing the big variant would buy nothing.
#[allow(clippy::large_enum_variant)]
pub enum NetworkAction {
    Account,
    Search,
    Library,
    Open(YoutubeRow, usize),
}

/// A finished network request, applied on the UI thread by `tick`.
pub enum NetworkResult {
    Account(Result<std::sync::Arc<YtMusic>, String>, bool),
    Lyrics(String, Option<String>),
    Search(Result<mp_ytmusic::SearchResults, String>),
    Artist(Tab, usize, Result<mp_ytmusic::ArtistPage, String>),
    Library(bool, Result<Option<mp_ytmusic::Library>, String>),
    /// Tab that asked, title, previous selection, tracks.
    Open(Tab, String, usize, Result<Vec<Track>, String>),
}

#[derive(Default)]
pub struct LyricsPanel {
    pub uri: String,
    pub title: String,
    pub text: Option<String>,
    pub loading: bool,
    pub scroll: u16,
}

impl LyricsPanel {
    pub fn move_by(&mut self, delta: isize) {
        self.scroll = (self.scroll as isize)
            .saturating_add(delta)
            .clamp(0, u16::MAX as isize) as u16;
    }
}

pub struct App {
    pub palette: crate::palette::Palette,
    skin_checked: Instant,
    pub eq: crate::equalizer::EqState,
    eq_dirty: Option<Instant>,
    requests: usize,
    pub account_checking: bool,
    pub started: Instant,
    pub lyrics: Option<LyricsPanel>,
    pub runtime: Result<Runtime, String>,
    pub youtube: Result<std::sync::Arc<YtMusic>, String>,
    net: (
        std::sync::mpsc::Sender<NetworkResult>,
        std::sync::mpsc::Receiver<NetworkResult>,
    ),
    pub yt_view: YoutubeView,
    pub playlists: YoutubeView,
    pub yt_query: String,
    pub local_query: String,
    pub pending: Option<NetworkAction>,
    pub message: Option<String>,
    /// Smoothed spectrum bars shown in the now-playing panel.
    pub bars: [f32; mp_core::player::BANDS],
    /// Short-lived warning in the now-playing panel, e.g. a track that failed to load.
    pub notice: Option<(String, Instant)>,
    pub library_loaded: bool,
    pub logged_in: bool,
    pub player: Player,
    pub folder: PathBuf,
    pub library: Vec<Track>,
    pub albums: Vec<Album>,
    /// (done, total) while the library is being scanned in the background.
    pub scan: Option<(usize, usize)>,
    scan_rx: Option<std::sync::mpsc::Receiver<ScanMsg>>,
    pub local: crate::local::Browser,
    /// What mpv is playing, in order; `status.index` points into it.
    pub queue: Vec<Track>,
    pub status: Status,
    pub tab: Tab,
    pub table: TableState,
    pub filter: String,
    pub searching: bool,
    pub areas: Areas,
    pub last_click: Option<(usize, Instant)>,
    pub quit: bool,
}

enum ScanMsg {
    Progress(usize, usize),
    Done(Vec<Track>),
}

impl App {
    pub fn new(player: Player, folder: PathBuf) -> Self {
        // Filled by `start_scan`, so the UI is up before a slow first scan finishes.
        let library = Vec::new();
        let albums = group_albums(&library);
        let mut table = TableState::default();
        table.select((!library.is_empty()).then_some(0));
        let runtime = Runtime::new().map_err(|e| format!("No se pudo iniciar Tokio: {e}"));
        let youtube = (|| {
            let dir = dirs::data_dir()
                .ok_or("No se encontró la carpeta de datos")?
                .join("dev.ricardo.musicplayer")
                .join("ytmusic");
            std::fs::create_dir_all(&dir)
                .map_err(|e| format!("No se pudo crear la carpeta: {e}"))?;
            YtMusic::new(&dir)
                .map(std::sync::Arc::new)
                .map_err(|e| format!("No se pudo iniciar YouTube Music: {e}"))
        })();
        // Login is done in the desktop app; reuse its cookies so sign-in-only tracks stream.
        if let Ok(yt) = &youtube {
            let _ = player.set_ytdl_cookies(yt.cookies_file().as_deref());
        }
        let mut app = Self {
            palette: crate::palette::lookup(
                mp_core::config::skin().as_deref().unwrap_or("musicplayer"),
            ),
            skin_checked: Instant::now(),
            eq: crate::equalizer::EqState::new(mp_core::config::equalizer()),
            eq_dirty: None,
            requests: 0,
            account_checking: true,
            started: Instant::now(),
            lyrics: None,
            runtime,
            youtube,
            net: std::sync::mpsc::channel(),
            yt_view: YoutubeView::default(),
            playlists: YoutubeView::default(),
            yt_query: String::new(),
            local_query: String::new(),
            pending: None,
            message: None,
            notice: None,
            bars: Default::default(),
            library_loaded: false,
            logged_in: false,
            player,
            folder,
            library,
            albums,
            scan: None,
            scan_rx: None,
            local: Default::default(),
            queue: Vec::new(),
            status: Status::default(),
            tab: Tab::Library,
            table,
            filter: String::new(),
            searching: false,
            areas: Areas::default(),
            last_click: None,
            quit: false,
        };
        app.schedule(NetworkAction::Account, "Comprobando cuenta…");
        if !cfg!(test) {
            app.run_pending();
        }
        app
    }

    pub fn is_playing(&self) -> bool {
        self.status.index.is_some() && !self.status.paused
    }

    pub fn cycle_palette(&mut self) {
        self.palette = self.palette.next();
        if let Err(e) = mp_core::config::set_skin(self.palette.id) {
            self.notice = Some((format!("No se pudo guardar tema: {e}"), Instant::now()));
        }
        self.skin_checked = Instant::now();
    }

    pub fn eq_changed(&mut self) {
        if let Err(e) = self.player.set_equalizer(&self.eq.value) {
            self.notice = Some((
                format!("No se pudo aplicar ecualizador: {e}"),
                Instant::now(),
            ));
        }
        self.eq_dirty = Some(Instant::now());
    }

    pub fn flush_eq(&mut self) {
        if self.eq_dirty.take().is_some()
            && let Err(e) = mp_core::config::set_equalizer(&self.eq.value)
        {
            self.notice = Some((
                format!("No se pudo guardar ecualizador: {e}"),
                Instant::now(),
            ));
        }
    }

    /// Scan the music folder on a worker thread; `tick` shows progress and the result.
    pub fn start_scan(&mut self) {
        let (tx, rx) = std::sync::mpsc::channel();
        let folder = self.folder.clone();
        std::thread::spawn(move || {
            let tracks = match mp_core::config::library_db() {
                Some(db) => mp_local::scan_cached_with_progress(&folder, &db, |d, t| {
                    let _ = tx.send(ScanMsg::Progress(d, t));
                }),
                None => mp_local::scan(&folder),
            };
            let _ = tx.send(ScanMsg::Done(tracks));
        });
        self.scan = Some((0, 0));
        self.scan_rx = Some(rx);
    }

    fn poll_scan(&mut self) {
        let Some(rx) = &self.scan_rx else { return };
        while let Ok(msg) = rx.try_recv() {
            match msg {
                ScanMsg::Progress(d, t) => self.scan = Some((d, t)),
                ScanMsg::Done(tracks) => {
                    self.albums = group_albums(&tracks);
                    self.library = tracks;
                    let query = self
                        .local
                        .history
                        .iter()
                        .chain(std::iter::once(&self.local.state))
                        .find(|s| s.page == crate::local::Page::Search)
                        .map(|s| s.filter.clone());
                    if let Some(query) = query {
                        self.local.results =
                            crate::local::search(&self.library, &self.albums, &self.folder, &query);
                    }
                    self.scan = None;
                    self.scan_rx = None;
                    self.table.select((self.row_count() > 0).then_some(0));
                    return;
                }
            }
        }
    }

    pub fn tick(&mut self) {
        self.poll_scan();
        if self.skin_checked.elapsed().as_secs() >= 2 {
            self.palette =
                crate::palette::lookup(mp_core::config::skin().as_deref().unwrap_or("musicplayer"));
            self.skin_checked = Instant::now();
        }
        if self
            .eq_dirty
            .is_some_and(|t| t.elapsed().as_millis() >= 300)
        {
            self.flush_eq();
        }
        self.status = self.player.status();
        let target = self.player.spectrum().filter(|_| self.is_playing());
        for (i, bar) in self.bars.iter_mut().enumerate() {
            let t = target.as_ref().map_or(0.0, |b| b[i]);
            // Jump up, fall slowly, so bars read like a hardware analyser.
            *bar = if t > *bar { t } else { *bar * 0.85 + t * 0.15 };
        }
        while let Ok(result) = self.net.1.try_recv() {
            self.apply_network(result);
        }
        if self.logged_in && !self.library_loaded && self.requests == 0 && self.pending.is_none() {
            self.schedule(NetworkAction::Library, "Cargando biblioteca…");
        }
        if self
            .lyrics
            .as_ref()
            .is_some_and(|panel| self.now_playing().is_some_and(|t| t.uri != panel.uri))
        {
            self.lyrics = None;
            self.toggle_lyrics();
        }
        if let Some(t) = self
            .player
            .take_failures()
            .last()
            .and_then(|&i| self.queue.get(i))
        {
            self.notice = Some((
                format!(
                    "No se pudo reproducir “{}”; se saltó. ¿Requiere iniciar sesión?",
                    t.title
                ),
                Instant::now(),
            ));
        }
        if self
            .notice
            .as_ref()
            .is_some_and(|(_, at)| at.elapsed().as_secs() >= 8)
        {
            self.notice = None;
        }
    }

    pub fn now_playing(&self) -> Option<&Track> {
        self.status.index.and_then(|i| self.queue.get(i))
    }

    /// Library rows matching the search filter.
    pub fn filtered(&self) -> Vec<&Track> {
        let q = self.filter.to_lowercase();
        self.library
            .iter()
            .filter(|t| {
                q.is_empty()
                    || format!("{} {} {}", t.title, t.artist, t.album)
                        .to_lowercase()
                        .contains(&q)
            })
            .collect()
    }

    pub fn row_count(&self) -> usize {
        match self.tab {
            Tab::Equalizer => mp_core::eq::PRESETS.len(),
            Tab::Library | Tab::Albums | Tab::Artists | Tab::Folders | Tab::LocalSearch => {
                self.local_rows(self.local.state.focus).len()
            }
            Tab::Queue => self.queue.len(),
            Tab::Youtube | Tab::Playlists => {
                if self.remote_error().is_some() {
                    0
                } else {
                    self.remote_view().rows.len()
                }
            }
        }
    }

    pub fn set_tab(&mut self, tab: Tab) {
        if self.search_results() {
            self.yt_view.selections[self.yt_view.focus] = self.table;
        }
        if self.is_local() {
            self.local.state.tables[self.local.state.focus] = self.table;
            self.local.state.filter = self.filter.clone();
        }
        let changed = self.tab != tab;
        self.tab = tab;
        if changed
            && matches!(
                tab,
                Tab::Library | Tab::Albums | Tab::Artists | Tab::Folders | Tab::LocalSearch
            )
        {
            self.local = Default::default();
            self.local.state.page = match tab {
                Tab::Albums => crate::local::Page::Albums,
                Tab::Artists => crate::local::Page::Artists,
                Tab::Folders => crate::local::Page::Folders(String::new()),
                Tab::LocalSearch => crate::local::Page::Search,
                _ => crate::local::Page::Songs,
            };
            self.filter.clear();
        }
        self.lyrics = None;
        self.searching = false;
        self.last_click = None;
        if matches!(tab, Tab::Youtube | Tab::Playlists) {
            if !self.logged_in && !self.account_checking {
                self.account_checking = true;
                self.schedule(NetworkAction::Account, "Comprobando cuenta…");
            } else if self.logged_in && !self.library_loaded {
                self.schedule(NetworkAction::Library, "Cargando biblioteca…");
            }
        }
        if tab == Tab::Youtube && !self.yt_view.searched && self.yt_view.opened.is_none() {
            self.yt_view.rows.clear();
        }
        let start = match tab {
            Tab::Queue => self.status.index.unwrap_or(0),
            _ => 0,
        };
        self.table = TableState::default();
        self.table.select(
            (self.row_count() > 0).then_some(start.min(self.row_count().saturating_sub(1))),
        );
        if self.search_results() {
            self.table = self.yt_view.selections[self.yt_view.focus];
        }
        if self.is_local() {
            self.filter = self.local.state.filter.clone();
            self.table = self.local.state.tables[self.local.state.focus];
            if self.table.selected().is_none() && self.row_count() > 0 {
                self.table.select(Some(0));
            }
        }
    }

    pub fn is_local(&self) -> bool {
        matches!(
            self.tab,
            Tab::Library | Tab::Albums | Tab::Artists | Tab::Folders | Tab::LocalSearch
        ) && self.lyrics.is_none()
    }

    pub fn local_artist(&self) -> bool {
        self.is_local() && matches!(self.local.state.page, crate::local::Page::Artist(_))
    }

    pub fn column_focus(&self) -> usize {
        if self.local_artist() || self.local_results() {
            self.local.state.focus
        } else {
            self.yt_view.focus
        }
    }

    pub fn local_results(&self) -> bool {
        self.is_local() && matches!(self.local.state.page, crate::local::Page::Search)
    }

    pub fn run_local_search(&mut self) {
        self.filter = self.local_query.clone();
        self.local.history.clear();
        self.local.state.page = crate::local::Page::Search;
        self.local.state.focus = 0;
        self.local.state.filter = self.filter.clone();
        self.local.results =
            crate::local::search(&self.library, &self.albums, &self.folder, &self.filter);
        self.local.state.tables = std::array::from_fn(|i| {
            TableState::default().with_selected((!self.local_rows(i).is_empty()).then_some(0))
        });
        self.table = self.local.state.tables[0];
    }

    pub fn local_rows(&self, column: usize) -> Vec<crate::local::Row> {
        if self.local_results() {
            return self.local.results[column].clone();
        }
        crate::local::rows(
            &self.local.state.page,
            column,
            &self.library,
            &self.albums,
            &self.folder,
            if self.local_results() {
                &self.local.state.filter
            } else {
                &self.filter
            },
            self.local.order,
        )
    }

    pub fn open_local(&mut self, page: crate::local::Page) {
        self.local.open(page, self.table, &self.filter);
        self.filter.clear();
        self.table = self.local.state.tables[self.local.state.focus];
        self.table.select((self.row_count() > 0).then_some(0));
        self.last_click = None;
    }

    fn play_local_queue(&mut self, queue: Vec<Track>, start: usize) {
        if start < queue.len()
            && self
                .player
                .play_queue(
                    &queue.iter().map(|t| t.uri.clone()).collect::<Vec<_>>(),
                    start,
                )
                .is_ok()
        {
            self.queue = queue;
        }
    }

    pub fn local_collection(&mut self, shuffle: bool) {
        use crate::local::Page;
        let mut tracks = match &self.local.state.page {
            Page::Album(i) => self.albums[*i].tracks.clone(),
            Page::Folders(path) => self
                .library
                .iter()
                .filter(|t| crate::local::under(t, &self.folder, path))
                .cloned()
                .collect(),
            _ => return,
        };
        if shuffle {
            use std::time::{SystemTime, UNIX_EPOCH};
            let mut seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            for i in (1..tracks.len()).rev() {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                tracks.swap(i, (seed % (i as u64 + 1)) as usize);
            }
        }
        self.play_local_queue(tracks, 0);
    }

    pub fn search_results(&self) -> bool {
        self.tab == Tab::Youtube && self.yt_view.results() && self.lyrics.is_none()
    }

    pub fn focus_column(&mut self, step: isize) {
        if self.local_artist() || self.local_results() {
            self.local.state.tables[self.local.state.focus] = self.table;
            self.local.state.focus = (self.local.state.focus as isize + step)
                .rem_euclid(if self.local_results() { 4 } else { 2 })
                as usize;
            self.table = self.local.state.tables[self.local.state.focus];
            let count = self.row_count();
            self.table.select(if count == 0 {
                None
            } else {
                Some(self.table.selected().unwrap_or(0).min(count - 1))
            });
            self.last_click = None;
            return;
        }
        self.yt_view.selections[self.yt_view.focus] = self.table;
        self.yt_view.focus_column(step);
        self.table = self.yt_view.selections[self.yt_view.focus];
        self.last_click = None;
    }

    pub fn cycle_tab(&mut self, step: isize) {
        let i = TABS.iter().position(|t| *t == self.tab).unwrap() as isize;
        self.set_tab(TABS[(i + step).rem_euclid(TABS.len() as isize) as usize]);
    }

    pub fn move_selection(&mut self, delta: isize) {
        if let Some(panel) = &mut self.lyrics {
            panel.move_by(delta);
            return;
        }
        let n = self.row_count();
        if n == 0 {
            return;
        }
        let cur = self.table.selected().unwrap_or(0) as isize;
        self.table
            .select(Some((cur + delta).clamp(0, n as isize - 1) as usize));
    }

    /// Play the selected row: the visible list becomes the queue.
    pub fn play_selected(&mut self) {
        if self.is_local() {
            use crate::local::{Page, Row};
            let rows = self.local_rows(self.local.state.focus);
            let sel = self.table.selected().unwrap_or(0);
            match rows.get(sel) {
                Some(Row::Artist(name)) => self.open_local(Page::Artist(name.clone())),
                Some(Row::Album(i)) => self.open_local(Page::Album(*i)),
                Some(Row::Folder(path)) => self.open_local(Page::Folders(path.clone())),
                Some(Row::Track(t)) => {
                    let queue: Vec<_> = if let Page::Album(i) = self.local.state.page {
                        self.albums[i].tracks.clone()
                    } else if self.local_artist() {
                        crate::local::rows(
                            &self.local.state.page,
                            0,
                            &self.library,
                            &self.albums,
                            &self.folder,
                            "",
                            self.local.order,
                        )
                        .into_iter()
                        .filter_map(|r| if let Row::Track(t) = r { Some(t) } else { None })
                        .collect()
                    } else {
                        rows.iter()
                            .filter_map(|r| {
                                if let Row::Track(t) = r {
                                    Some(t.clone())
                                } else {
                                    None
                                }
                            })
                            .collect()
                    };
                    let start = queue.iter().position(|x| x.uri == t.uri).unwrap_or(0);
                    self.play_local_queue(queue, start);
                }
                _ => {}
            }
            return;
        }
        let Some(sel) = self.table.selected() else {
            return;
        };
        let (queue, start) = match self.tab {
            Tab::Equalizer => {
                self.eq.apply_preset(sel);
                self.eq_changed();
                return;
            }
            Tab::Artists | Tab::Folders | Tab::LocalSearch => return,
            Tab::Library => (self.filtered().into_iter().cloned().collect(), sel),
            Tab::Albums => match self.albums.get(sel) {
                Some(a) => (a.tracks.clone(), 0),
                None => return,
            },
            Tab::Queue => (self.queue.clone(), sel),
            Tab::Youtube | Tab::Playlists => {
                let Some(row) = self.remote_view().rows.get(sel).cloned() else {
                    return;
                };
                match row {
                    YoutubeRow::Track(_) => (self.remote_view().tracks(), sel),
                    _ => {
                        self.schedule(NetworkAction::Open(row, sel), "Abriendo colección…");
                        return;
                    }
                }
            }
        };
        if start >= queue.len() {
            return;
        }
        let uris: Vec<String> = queue.iter().map(|t| t.uri.clone()).collect();
        if self.player.play_queue(&uris, start).is_ok() {
            self.queue = queue;
        }
    }

    pub fn play_collection(&mut self) {
        if self.remote_view().opened.is_none() || self.lyrics.is_some() {
            return;
        }
        let selected = self.table.selected();
        self.table.select(Some(0));
        self.play_selected();
        self.table.select(selected);
    }

    pub fn open_top(&mut self) {
        if self.local_results() {
            let columns = std::array::from_fn(|i| self.local_rows(i));
            if let Some(row) = crate::local::top(&columns, &self.albums, &self.local.state.filter) {
                match row {
                    crate::local::Row::Artist(n) => self.open_local(crate::local::Page::Artist(n)),
                    crate::local::Row::Album(i) => self.open_local(crate::local::Page::Album(i)),
                    crate::local::Row::Track(t) => {
                        self.focus_column(-(self.column_focus() as isize));
                        self.table.select(columns[0].iter().position(
                            |r| matches!(r,crate::local::Row::Track(x) if x.uri == t.uri),
                        ));
                        self.play_selected();
                    }
                    _ => {}
                }
            }
            return;
        }

        let row = match self.yt_view.top.clone() {
            Some(mp_ytmusic::Top::Artist(c)) => YoutubeRow::Collection(c),
            Some(mp_ytmusic::Top::Track(t)) => {
                let uris = vec![t.uri.clone()];
                if self.player.play_queue(&uris, 0).is_ok() {
                    self.queue = vec![t];
                }
                return;
            }
            None => return,
        };
        self.schedule(
            NetworkAction::Open(row, self.table.selected().unwrap_or(0)),
            "Cargando artista…",
        );
    }

    pub fn remote_view(&self) -> &YoutubeView {
        if self.tab == Tab::Playlists {
            &self.playlists
        } else {
            &self.yt_view
        }
    }

    pub fn remote_view_mut(&mut self) -> &mut YoutubeView {
        if self.tab == Tab::Playlists {
            &mut self.playlists
        } else {
            &mut self.yt_view
        }
    }

    pub fn remote_error(&self) -> Option<&str> {
        self.runtime
            .as_ref()
            .err()
            .or(self.youtube.as_ref().err())
            .or(self.remote_view().error.as_ref())
            .map(String::as_str)
    }

    pub fn back(&mut self) {
        if self.lyrics.take().is_some() {
            return;
        }
        if self.is_local() {
            if let Some(state) = self.local.back() {
                self.filter = state.filter;
                self.table = state.tables[state.focus];
            } else if let crate::local::Page::Folders(path) = &self.local.state.page {
                let parent = path.rsplit_once('/').map_or("", |(p, _)| p).to_string();
                self.local.state.page = crate::local::Page::Folders(parent);
                self.filter.clear();
                self.table.select((self.row_count() > 0).then_some(0));
            } else {
                self.filter.clear();
                self.table.select((self.row_count() > 0).then_some(0));
            }
        } else if matches!(self.tab, Tab::Youtube | Tab::Playlists) {
            if let Some(selection) = self.remote_view_mut().back() {
                self.table = if self.search_results() {
                    self.yt_view.selections[self.yt_view.focus]
                } else {
                    TableState::default().with_selected(Some(selection))
                };
            } else if self.search_results() {
                self.yt_view.error = None;
            } else {
                if self.tab == Tab::Youtube && !self.yt_view.searched {
                    self.yt_view.rows.clear();
                }
                self.remote_view_mut().error = None;
                self.table.select((self.row_count() > 0).then_some(0));
            }
        } else {
            self.filter.clear();
            self.table.select((self.row_count() > 0).then_some(0));
        }
        self.last_click = None;
    }

    pub fn schedule(&mut self, action: NetworkAction, message: &str) {
        match &action {
            NetworkAction::Search => self.yt_view.error = None,
            NetworkAction::Library => {
                self.playlists.error = None;
                self.yt_view.error = None;
            }
            NetworkAction::Open(..) => {
                if self.search_results() {
                    self.yt_view.selections[self.yt_view.focus] = self.table;
                }
                self.remote_view_mut().error = None;
            }
            NetworkAction::Account => {}
        }
        if matches!(self.pending, Some(NetworkAction::Account))
            && !matches!(action, NetworkAction::Account)
        {
            self.account_checking = false;
        }
        self.pending = Some(action);
        self.message = Some(message.into());
    }

    /// Start the queued request in the background; `tick` applies its result.
    pub fn run_pending(&mut self) {
        if self.requests > 0 {
            return;
        }
        let Some(action) = self.pending.take() else {
            return;
        };
        let (Ok(runtime), Ok(yt)) = (&self.runtime, &self.youtube) else {
            self.message = None;
            self.account_checking = false;
            return;
        };
        let (yt, tx, tab) = (yt.clone(), self.net.0.clone(), self.tab);
        let query = self.yt_query.clone();
        let err = |e: mp_ytmusic::Error| e.to_string();
        self.requests += 1;
        runtime.spawn(async move {
            let result = match action {
                NetworkAction::Account => {
                    let fresh = youtube_dir().and_then(|dir| {
                        YtMusic::new(&dir)
                            .map(std::sync::Arc::new)
                            .map_err(|e| e.to_string())
                    });
                    let logged_in = match &fresh {
                        Ok(yt) => yt.logged_in().await,
                        Err(_) => false,
                    };
                    NetworkResult::Account(fresh, logged_in)
                }
                NetworkAction::Search => {
                    NetworkResult::Search(yt.search(&query).await.map_err(err))
                }
                NetworkAction::Library => {
                    let logged_in = yt.logged_in().await;
                    let lib = if logged_in {
                        yt.library().await.map(Some).map_err(err)
                    } else {
                        Ok(None)
                    };
                    NetworkResult::Library(logged_in, lib)
                }
                NetworkAction::Open(row, selection) => {
                    if let YoutubeRow::Collection(c) = &row
                        && c.kind == mp_ytmusic::Kind::Artist
                    {
                        let result = yt.artist(&c.id).await.map_err(err);
                        let _ = tx.send(NetworkResult::Artist(tab, selection, result));
                        return;
                    }
                    let (title, tracks) = match row {
                        YoutubeRow::Collection(c) => (c.title, yt.open(&c.kind, &c.id).await),
                        YoutubeRow::Liked => ("♥ Me gusta".into(), yt.liked().await),
                        YoutubeRow::Track(_) => return,
                    };
                    NetworkResult::Open(tab, title, selection, tracks.map_err(err))
                }
            };
            let _ = tx.send(result);
        });
    }

    fn apply_network(&mut self, result: NetworkResult) {
        if let NetworkResult::Lyrics(uri, text) = result {
            if let Some(panel) = &mut self.lyrics
                && panel.uri == uri
            {
                panel.text = text.filter(|s| !s.trim().is_empty());
                panel.loading = false;
            }
            return;
        }
        self.requests = self.requests.saturating_sub(1);
        if self.pending.is_none() {
            self.message = None;
        }
        let (tab, result) = match result {
            NetworkResult::Lyrics(..) => unreachable!(),
            NetworkResult::Account(yt, logged_in) => {
                self.youtube = yt;
                self.logged_in = logged_in;
                self.account_checking = false;
                self.library_loaded = false;
                if logged_in {
                    if let Ok(yt) = &self.youtube {
                        let _ = self.player.set_ytdl_cookies(yt.cookies_file().as_deref());
                    }
                    if self.pending.is_none() {
                        self.schedule(NetworkAction::Library, "Cargando biblioteca…");
                    }
                }
                return;
            }
            NetworkResult::Search(r) => (Tab::Youtube, r.map(|res| self.yt_view.search(res))),
            NetworkResult::Library(logged_in, r) => {
                self.library_loaded = true;
                self.logged_in = logged_in;
                if logged_in && let Ok(yt) = &self.youtube {
                    let _ = self.player.set_ytdl_cookies(yt.cookies_file().as_deref());
                }
                (
                    Tab::Playlists,
                    r.map(|lib| {
                        if let Some(lib) = lib {
                            self.playlists.library(lib);
                            if !self.yt_view.searched && self.yt_view.opened.is_none() {
                                self.yt_view.rows.clear();
                                self.yt_view.error = None;
                            }
                        }
                    }),
                )
            }
            NetworkResult::Artist(tab, selection, r) => {
                let view = if tab == Tab::Playlists {
                    &mut self.playlists
                } else {
                    &mut self.yt_view
                };
                (tab, r.map(|page| view.artist(page, selection)))
            }
            NetworkResult::Open(tab, title, selection, r) => {
                let view = if tab == Tab::Playlists {
                    &mut self.playlists
                } else {
                    &mut self.yt_view
                };
                (tab, r.map(|tracks| view.open(title, tracks, selection)))
            }
        };
        let view = if tab == Tab::Playlists {
            &mut self.playlists
        } else {
            &mut self.yt_view
        };
        if let Err(e) = result {
            view.error = Some(format!("Error de YouTube Music: {e}").replace(['\n', '\r'], " "));
        }
        if self.tab == tab
            || (tab == Tab::Playlists && self.tab == Tab::Youtube && self.yt_query.is_empty())
        {
            self.table = if self.search_results() {
                self.yt_view.selections[self.yt_view.focus]
            } else {
                TableState::default().with_selected((self.row_count() > 0).then_some(0))
            };
            self.last_click = None;
        }
    }

    pub fn toggle_lyrics(&mut self) {
        if self.lyrics.take().is_some() {
            return;
        }
        let Some(track) = self.now_playing().cloned() else {
            return;
        };
        self.lyrics = Some(LyricsPanel {
            uri: track.uri.clone(),
            title: track.title,
            loading: true,
            ..Default::default()
        });
        let Ok(runtime) = &self.runtime else {
            self.lyrics.as_mut().unwrap().loading = false;
            return;
        };
        let tx = self.net.0.clone();
        let yt = self.youtube.as_ref().ok().cloned();
        runtime.spawn(async move {
            let uri = track.uri;
            let text = if uri.starts_with("https://") {
                match yt {
                    Some(yt) => yt.lyrics(&uri).await.ok().flatten(),
                    None => None,
                }
            } else {
                let path = PathBuf::from(&uri);
                tokio::task::spawn_blocking(move || mp_local::lyrics(&path))
                    .await
                    .ok()
                    .flatten()
            };
            let _ = tx.send(NetworkResult::Lyrics(uri, text));
        });
    }

    pub fn prev(&mut self) -> Result<(), mp_core::player::Error> {
        self.player.prev()
    }

    pub fn toggle(&mut self) -> Result<(), mp_core::player::Error> {
        self.player.toggle_pause()
    }

    pub fn next(&mut self) -> Result<(), mp_core::player::Error> {
        self.player.next()
    }

    pub fn seek_by(&self, seconds: f64) {
        let _ = self
            .player
            .seek((self.status.position + seconds).clamp(0.0, self.status.duration));
    }

    pub fn volume_by(&self, delta: f64) {
        let _ = self.player.set_volume(self.status.volume + delta);
    }

    /// Table row under a screen line, if any.
    pub fn row_at(&self, y: u16) -> Option<usize> {
        if self.lyrics.is_some() || self.search_results() {
            return None;
        }
        let first_row = self.areas.list.y + 1;
        if y < first_row || y >= self.areas.list.bottom() {
            return None;
        }
        let row = self.table.offset() + (y - first_row) as usize;
        (row < self.row_count()).then_some(row)
    }
}

fn youtube_dir() -> Result<PathBuf, String> {
    dirs::data_dir()
        .map(|dir| dir.join("dev.ricardo.musicplayer/ytmusic"))
        .ok_or_else(|| "No se encontró la carpeta de datos".into())
}

pub fn group_albums(tracks: &[Track]) -> Vec<Album> {
    crate::local::group(tracks)
}

pub fn fmt_time(s: f64) -> String {
    if !s.is_finite() || s <= 0.0 {
        return "0:00".into();
    }
    let s = s as u64;
    format!("{}:{:02}", s / 60, s % 60)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn track(title: &str, artist: &str, album: &str) -> Track {
        Track {
            title: title.into(),
            artist: artist.into(),
            album: album.into(),
            uri: format!("/{artist}/{album}/{title}"),
            track_no: None,
            duration: Some(125.0),
            cover: None,
            ..Default::default()
        }
    }

    pub fn app(tracks: Vec<Track>) -> App {
        let player = Player::new().unwrap();
        let mut a = App::new(player, PathBuf::from("/nonexistent"));
        a.pending = None;
        a.albums = group_albums(&tracks);
        a.library = tracks;
        a.table.select(Some(0));
        a
    }

    #[test]
    fn artist_album_navigation_restores_tables_without_autoplay() {
        let mut a = app(Vec::new());
        a.tab = Tab::Youtube;
        a.yt_view.search(mp_ytmusic::SearchResults {
            tracks: vec![track("uno", "A", "B"), track("dos", "A", "B")],
            ..Default::default()
        });
        a.table.select(Some(1));
        *a.table.offset_mut() = 1;
        a.yt_view.selections[0] = a.table;
        a.apply_network(NetworkResult::Artist(
            Tab::Youtube,
            1,
            Ok(crate::youtube::tests::artist_page()),
        ));
        assert_eq!(a.yt_view.focus, 1);
        assert_eq!(a.table.selected(), Some(0));
        a.move_selection(1);
        a.play_selected();
        assert!(matches!(a.pending, Some(NetworkAction::Open(
            YoutubeRow::Collection(ref c), 1)) if c.kind == mp_ytmusic::Kind::Album));
        a.pending = None;
        a.apply_network(NetworkResult::Open(
            Tab::Youtube,
            "Disco completo".into(),
            1,
            Ok(vec![track("primera", "A", "B"), track("segunda", "A", "B")]),
        ));
        assert!(a.queue.is_empty());
        assert_eq!(a.yt_view.opened.as_ref().unwrap().0, "Disco completo");
        a.back();
        assert_eq!(a.yt_view.focus, 1);
        assert_eq!(a.table.selected(), Some(1));
        a.focus_column(-1);
        a.back();
        assert!(a.yt_view.artist_name.is_none());
        assert_eq!(a.yt_view.focus, 0);
        assert_eq!(a.table.selected(), Some(1));
        assert_eq!(a.table.offset(), 1);
    }

    #[test]
    fn lyrics_scroll_and_result_state() {
        let mut panel = LyricsPanel {
            text: Some("uno\ndos\ntres".into()),
            ..Default::default()
        };
        panel.move_by(100);
        assert_eq!(panel.scroll, 100);
        panel.move_by(-100);
        assert_eq!(panel.scroll, 0);
        let mut a = app(Vec::new());
        a.lyrics = Some(LyricsPanel {
            uri: "actual".into(),
            loading: true,
            ..Default::default()
        });
        a.apply_network(NetworkResult::Lyrics(
            "anterior".into(),
            Some("vieja".into()),
        ));
        assert!(a.lyrics.as_ref().unwrap().loading);
        a.apply_network(NetworkResult::Lyrics("actual".into(), Some(" ".into())));
        assert!(!a.lyrics.as_ref().unwrap().loading);
        assert!(a.lyrics.as_ref().unwrap().text.is_none());
        a.back();
        assert!(a.lyrics.is_none());
    }

    #[test]
    fn library_reaches_both_remote_tabs() {
        let mut a = app(Vec::new());
        a.apply_network(NetworkResult::Library(
            true,
            Ok(Some(mp_ytmusic::Library::default())),
        ));
        assert!(matches!(a.playlists.rows[0], YoutubeRow::Liked));
        assert!(a.yt_view.rows.is_empty());
        a.account_checking = false;
        a.logged_in = false;
        a.set_tab(Tab::Youtube);
        assert!(matches!(a.pending, Some(NetworkAction::Account)));
    }

    #[test]
    fn columns_preserve_selection_and_return_from_artist() {
        let mut a = app(Vec::new());
        a.yt_view.search(mp_ytmusic::SearchResults {
            tracks: vec![track("uno", "A", "B"), track("dos", "A", "B")],
            artists: vec![mp_ytmusic::Collection {
                kind: mp_ytmusic::Kind::Artist,
                id: "artist".into(),
                title: "Artista".into(),
                subtitle: "10 suscriptores".into(),
                cover: None,
            }],
            ..Default::default()
        });
        a.set_tab(Tab::Youtube);
        a.move_selection(1);
        a.focus_column(1);
        assert_eq!(a.yt_view.focus, 1);
        assert_eq!(a.table.selected(), Some(0));
        a.play_selected();
        assert!(
            matches!(a.pending, Some(NetworkAction::Open(YoutubeRow::Collection(ref c), 0)) if c.kind == mp_ytmusic::Kind::Artist)
        );
        a.apply_network(NetworkResult::Open(
            Tab::Youtube,
            "Artista".into(),
            0,
            Ok(vec![track("popular", "A", "B")]),
        ));
        assert!(!a.search_results());
        a.back();
        assert!(a.search_results());
        assert_eq!(a.yt_view.focus, 1);
        assert_eq!(a.table.selected(), Some(0));
        a.focus_column(-1);
        assert_eq!(a.table.selected(), Some(1));
        a.focus_column(-1);
        assert_eq!(a.yt_view.focus, 3);
        assert_eq!(a.table.selected(), None);
        a.focus_column(1);
        a.set_tab(Tab::Library);
        a.set_tab(Tab::Youtube);
        assert_eq!(a.table.selected(), Some(1));
    }

    #[test]
    fn groups_contiguous_albums() {
        let albums = group_albums(&[
            track("a", "X", "One"),
            track("b", "X", "One"),
            track("c", "X", "Two"),
            track("d", "Y", "Two"),
        ]);
        let shape: Vec<_> = albums
            .iter()
            .map(|a| (a.artist.as_str(), a.name.as_str(), a.tracks.len()))
            .collect();
        assert_eq!(shape, [("X", "One", 2), ("X", "Two", 1), ("Y", "Two", 1)]);
    }

    #[test]
    fn filter_matches_any_field_case_insensitive() {
        let mut a = app(vec![
            track("Midnight City", "M83", "Hurry Up"),
            track("Wait", "M83", "Hurry Up"),
            track("Intro", "Other", "Dreams"),
        ]);
        a.filter = "m83".into();
        assert_eq!(a.filtered().len(), 2);
        a.filter = "dream".into();
        assert_eq!(a.filtered()[0].title, "Intro");
    }

    #[test]
    fn click_maps_to_row_with_scroll_offset() {
        let mut a = app((0..30).map(|i| track(&i.to_string(), "A", "B")).collect());
        a.areas.list = Rect::new(0, 5, 80, 10);
        assert_eq!(a.row_at(5), None, "header line");
        assert_eq!(a.row_at(6), Some(0));
        *a.table.offset_mut() = 7;
        assert_eq!(a.row_at(8), Some(9));
        assert_eq!(a.row_at(15), None, "below list");
    }

    #[test]
    fn selection_and_tabs_stay_in_bounds() {
        let mut a = app(vec![track("a", "X", "One"), track("b", "Y", "Two")]);
        a.move_selection(10);
        assert_eq!(a.table.selected(), Some(1));
        a.move_selection(-10);
        assert_eq!(a.table.selected(), Some(0));
        a.cycle_tab(-1);
        assert_eq!(a.tab, Tab::Equalizer);
        assert_eq!(a.table.selected(), Some(0), "preset list");
        a.cycle_tab(-1);
        assert_eq!(a.tab, Tab::Queue);
        assert_eq!(a.table.selected(), None, "empty queue");
        a.cycle_tab(1);
        a.cycle_tab(1);
        assert_eq!(a.tab, Tab::Library);
    }
}
