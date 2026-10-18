use std::{path::PathBuf, time::Instant};

use mp_core::{
    Track,
    player::{Player, Status},
};
use ratatui::{layout::Rect, widgets::TableState};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Library,
    Albums,
    Queue,
}

pub const TABS: [Tab; 3] = [Tab::Library, Tab::Albums, Tab::Queue];

impl Tab {
    pub fn title(self) -> &'static str {
        match self {
            Tab::Library => "Biblioteca",
            Tab::Albums => "Álbumes",
            Tab::Queue => "Cola",
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
    pub tabs: Vec<Rect>,
    /// Inner area of the list table (rows start one line below, after the header).
    pub list: Rect,
    pub buttons: [Rect; 3],
    pub progress: Rect,
    pub volume: Rect,
}

pub struct App {
    pub player: Player,
    pub folder: PathBuf,
    pub library: Vec<Track>,
    pub albums: Vec<Album>,
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

impl App {
    pub fn new(player: Player, folder: PathBuf) -> Self {
        let library = mp_local::scan(&folder);
        let albums = group_albums(&library);
        let mut table = TableState::default();
        table.select((!library.is_empty()).then_some(0));
        Self {
            player,
            folder,
            library,
            albums,
            queue: Vec::new(),
            status: Status::default(),
            tab: Tab::Library,
            table,
            filter: String::new(),
            searching: false,
            areas: Areas::default(),
            last_click: None,
            quit: false,
        }
    }

    pub fn tick(&mut self) {
        self.status = self.player.status();
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
            Tab::Library => self.filtered().len(),
            Tab::Albums => self.albums.len(),
            Tab::Queue => self.queue.len(),
        }
    }

    pub fn set_tab(&mut self, tab: Tab) {
        self.tab = tab;
        let start = match tab {
            Tab::Queue => self.status.index.unwrap_or(0),
            _ => 0,
        };
        self.table = TableState::default();
        self.table.select(
            (self.row_count() > 0).then_some(start.min(self.row_count().saturating_sub(1))),
        );
    }

    pub fn cycle_tab(&mut self, step: isize) {
        let i = TABS.iter().position(|t| *t == self.tab).unwrap() as isize;
        self.set_tab(TABS[(i + step).rem_euclid(TABS.len() as isize) as usize]);
    }

    pub fn move_selection(&mut self, delta: isize) {
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
        let Some(sel) = self.table.selected() else {
            return;
        };
        let (queue, start) = match self.tab {
            Tab::Library => (self.filtered().into_iter().cloned().collect(), sel),
            Tab::Albums => match self.albums.get(sel) {
                Some(a) => (a.tracks.clone(), 0),
                None => return,
            },
            Tab::Queue => (self.queue.clone(), sel),
        };
        if start >= queue.len() {
            return;
        }
        let uris: Vec<String> = queue.iter().map(|t| t.uri.clone()).collect();
        if self.player.play_queue(&uris, start).is_ok() {
            self.queue = queue;
        }
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
        let first_row = self.areas.list.y + 1;
        if y < first_row || y >= self.areas.list.bottom() {
            return None;
        }
        let row = self.table.offset() + (y - first_row) as usize;
        (row < self.row_count()).then_some(row)
    }
}

pub fn group_albums(tracks: &[Track]) -> Vec<Album> {
    let mut albums: Vec<Album> = Vec::new();
    // Tracks come sorted by artist, album, track number, so albums are contiguous.
    for t in tracks {
        match albums.last_mut() {
            Some(a) if a.artist == t.artist && a.name == t.album => a.tracks.push(t.clone()),
            _ => albums.push(Album {
                artist: t.artist.clone(),
                name: t.album.clone(),
                tracks: vec![t.clone()],
            }),
        }
    }
    albums
}

pub fn fmt_time(s: f64) -> String {
    if !s.is_finite() || s <= 0.0 {
        return "0:00".into();
    }
    let s = s as u64;
    format!("{}:{:02}", s / 60, s % 60)
}

/// Where the library folder is remembered between runs.
pub fn folder_file() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("musicplayer").join("tui-folder"))
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
        }
    }

    pub fn app(tracks: Vec<Track>) -> App {
        let player = Player::new().unwrap();
        let mut a = App::new(player, PathBuf::from("/nonexistent"));
        a.albums = group_albums(&tracks);
        a.library = tracks;
        a.table.select(Some(0));
        a
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
        assert_eq!(a.tab, Tab::Queue);
        assert_eq!(a.table.selected(), None, "empty queue");
        a.cycle_tab(1);
        assert_eq!(a.tab, Tab::Library);
    }
}
