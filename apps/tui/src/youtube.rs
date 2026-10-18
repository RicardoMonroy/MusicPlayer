use mp_core::Track;
use mp_ytmusic::{Collection, Library, SearchResults, Top};

#[derive(Clone)]
pub enum YoutubeRow {
    Track(Track),
    Collection(Collection),
    Liked,
}

impl YoutubeRow {
    pub fn label(&self) -> (String, String, String) {
        match self {
            Self::Track(t) => ("♪".into(), t.title.clone(), t.artist.clone()),
            Self::Collection(c) => (
                match c.kind {
                    mp_ytmusic::Kind::Artist => "♬ Artista",
                    mp_ytmusic::Kind::Album => "◉ Álbum",
                    mp_ytmusic::Kind::Playlist => "≡ Playlist",
                }
                .into(),
                c.title.clone(),
                c.subtitle.clone(),
            ),
            Self::Liked => ("♥".into(), "Me gusta".into(), String::new()),
        }
    }
}

#[derive(Clone, Default)]
pub struct YoutubeView {
    pub history: Vec<YoutubeView>,
    pub artist_name: Option<String>,
    pub columns: [Vec<YoutubeRow>; 4],
    pub selections: [ratatui::widgets::TableState; 4],
    pub focus: usize,
    pub searched: bool,
    pub top: Option<Top>,
    pub songs: usize,
    pub rows: Vec<YoutubeRow>,
    pub opened: Option<(String, Vec<YoutubeRow>, usize)>,
    pub error: Option<String>,
}

impl YoutubeView {
    pub fn search(&mut self, results: SearchResults) {
        self.history.clear();
        self.artist_name = None;
        self.songs = results.tracks.len();
        self.top = results.top;
        self.columns = [
            results
                .tracks
                .into_iter()
                .chain(results.videos)
                .map(YoutubeRow::Track)
                .collect(),
            results
                .artists
                .into_iter()
                .map(YoutubeRow::Collection)
                .collect(),
            results
                .albums
                .into_iter()
                .map(YoutubeRow::Collection)
                .collect(),
            results
                .playlists
                .into_iter()
                .map(YoutubeRow::Collection)
                .collect(),
        ];
        self.selections = std::array::from_fn(|i| {
            ratatui::widgets::TableState::default()
                .with_selected((!self.columns[i].is_empty()).then_some(0))
        });
        self.focus = 0;
        self.searched = true;
        self.rows = self.columns[0].clone();
        self.opened = None;
        self.error = None;
    }

    pub fn results(&self) -> bool {
        (self.searched || self.artist_name.is_some()) && self.opened.is_none()
    }

    pub fn focus_column(&mut self, step: isize) {
        self.focus = (self.focus as isize + step).rem_euclid(self.column_count() as isize) as usize;
        self.rows = self.columns[self.focus].clone();
    }

    pub fn column_count(&self) -> usize {
        if self.artist_name.is_some() {
            if self.columns[2].is_empty() { 2 } else { 3 }
        } else {
            4
        }
    }

    fn push(&mut self, selection: usize) {
        if self.results() {
            self.selections[self.focus].select(Some(selection));
        }
        let history = std::mem::take(&mut self.history);
        let snapshot = self.clone();
        self.history = history;
        self.history.push(snapshot);
    }

    pub fn artist(&mut self, page: mp_ytmusic::ArtistPage, selection: usize) {
        self.push(selection);
        self.artist_name = Some(page.name);
        self.top = None;
        self.opened = None;
        self.songs = page.tracks.len();
        self.columns = [
            page.tracks.into_iter().map(YoutubeRow::Track).collect(),
            page.albums
                .into_iter()
                .map(YoutubeRow::Collection)
                .collect(),
            page.playlists
                .into_iter()
                .map(YoutubeRow::Collection)
                .collect(),
            Vec::new(),
        ];
        self.selections = std::array::from_fn(|i| {
            ratatui::widgets::TableState::default()
                .with_selected((!self.columns[i].is_empty()).then_some(0))
        });
        self.focus = 1;
        self.rows = self.columns[1].clone();
        self.error = None;
    }

    pub fn library(&mut self, library: Library) {
        self.history.clear();
        self.artist_name = None;
        self.opened = None;
        self.rows = std::iter::once(YoutubeRow::Liked)
            .chain(library.playlists.into_iter().map(YoutubeRow::Collection))
            .chain(library.albums.into_iter().map(YoutubeRow::Collection))
            .collect();
        self.error = None;
    }

    pub fn open(&mut self, title: String, tracks: Vec<Track>, selection: usize) {
        self.push(selection);
        let rows = tracks.into_iter().map(YoutubeRow::Track).collect();
        self.opened = Some((title, std::mem::replace(&mut self.rows, rows), selection));
        self.error = None;
    }

    pub fn back(&mut self) -> Option<usize> {
        if let Some(mut previous) = self.history.pop() {
            previous.history = std::mem::take(&mut self.history);
            let selection = if previous.results() {
                previous.selections[previous.focus].selected().unwrap_or(0)
            } else {
                self.opened.as_ref().map_or(0, |o| o.2)
            };
            *self = previous;
            return Some(selection);
        }
        let (_, rows, selection) = self.opened.take()?;
        self.rows = rows;
        self.error = None;
        Some(selection)
    }

    pub fn tracks(&self) -> Vec<Track> {
        self.rows
            .iter()
            .filter_map(|r| match r {
                YoutubeRow::Track(t) => Some(t.clone()),
                _ => None,
            })
            .collect()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::app::tests::track;
    use mp_ytmusic::Kind;

    fn collection(kind: Kind) -> Collection {
        Collection {
            kind,
            id: "id".into(),
            title: "Título".into(),
            subtitle: "Artista".into(),
            cover: None,
        }
    }

    pub fn artist_page() -> mp_ytmusic::ArtistPage {
        mp_ytmusic::ArtistPage {
            name: "Artista de prueba".into(),
            cover: None,
            tracks: vec![track("popular", "A", "B")],
            albums: vec![collection(Kind::Album), collection(Kind::Album)],
            playlists: Vec::new(),
        }
    }

    #[test]
    fn artist_starts_on_albums_and_stack_restores_columns() {
        let mut view = YoutubeView::default();
        view.search(SearchResults {
            tracks: vec![track("uno", "A", "B"), track("dos", "A", "B")],
            artists: vec![collection(Kind::Artist), collection(Kind::Artist)],
            ..Default::default()
        });
        view.selections[0].select(Some(1));
        view.focus_column(1);
        view.artist(artist_page(), 1);
        assert_eq!(view.focus, 1);
        assert_eq!(view.selections[1].selected(), Some(0));
        assert_eq!(view.column_count(), 2);
        view.open("Álbum elegido".into(), vec![track("canción", "A", "B")], 1);
        assert!(!view.results());
        assert_eq!(view.back(), Some(1));
        assert_eq!(view.artist_name.as_deref(), Some("Artista de prueba"));
        assert_eq!(view.focus, 1);
        assert_eq!(view.selections[1].selected(), Some(1));
        assert_eq!(view.back(), Some(1));
        assert!(view.results());
        assert!(view.artist_name.is_none());
        assert_eq!(view.focus, 1);
        assert_eq!(view.selections[1].selected(), Some(1));
        assert_eq!(view.selections[0].selected(), Some(1));
        assert_eq!(view.back(), None);
    }

    #[test]
    fn search_rows_and_track_queue() {
        let mut view = YoutubeView::default();
        view.search(SearchResults {
            top: Some(Top::Artist(collection(Kind::Artist))),
            tracks: vec![track("a", "b", "c")],
            videos: vec![track("video", "b", "c")],
            artists: vec![collection(Kind::Artist)],
            albums: vec![collection(Kind::Album)],
            playlists: vec![collection(Kind::Playlist)],
        });
        assert!(matches!(view.rows[0], YoutubeRow::Track(_)));
        assert_eq!(view.columns[2][0].label().0, "◉ Álbum");
        assert_eq!(view.columns[3][0].label().0, "≡ Playlist");
        assert_eq!(view.tracks().len(), 2);
        assert_eq!(view.songs, 1);
        assert_eq!(view.tracks()[1].title, "video");
        assert!(matches!(view.top, Some(Top::Artist(_))));
        assert_eq!(view.columns[1][0].label().0, "♬ Artista");
        view.search(SearchResults {
            top: Some(Top::Track(track("principal", "artista", "álbum"))),
            ..Default::default()
        });
        assert!(matches!(view.top, Some(Top::Track(ref t)) if t.title == "principal"));
    }

    #[test]
    fn column_focus_wraps_and_keeps_independent_state() {
        let mut view = YoutubeView::default();
        view.search(SearchResults {
            tracks: vec![track("uno", "A", "B"), track("dos", "A", "B")],
            artists: vec![collection(Kind::Artist)],
            ..Default::default()
        });
        view.selections[0].select(Some(1));
        view.focus_column(-1);
        assert_eq!(view.focus, 3);
        assert!(view.rows.is_empty());
        view.focus_column(2);
        assert_eq!(view.focus, 1);
        assert!(matches!(view.rows[0], YoutubeRow::Collection(_)));
        view.open("Artista".into(), vec![track("popular", "A", "B")], 0);
        assert!(!view.results());
        assert_eq!(view.back(), Some(0));
        assert!(view.results());
        assert_eq!(view.focus, 1);
        view.focus_column(-1);
        assert_eq!(view.selections[0].selected(), Some(1));
        assert_eq!(view.rows.len(), 2);
    }

    #[test]
    fn library_order_and_back_restore_selection() {
        let mut view = YoutubeView::default();
        view.library(Library {
            playlists: vec![collection(Kind::Playlist)],
            albums: vec![collection(Kind::Album)],
        });
        assert!(matches!(view.rows[0], YoutubeRow::Liked));
        assert_eq!(view.rows[1].label().0, "≡ Playlist");
        view.open("Título".into(), vec![track("a", "b", "c")], 2);
        assert_eq!(view.rows.len(), 1);
        assert_eq!(view.opened.as_ref().unwrap().0, "Título");
        assert_eq!(view.back(), Some(2));
        assert_eq!(view.rows.len(), 3);
        assert_eq!(view.back(), None);
    }
}
