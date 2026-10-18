use mp_core::Track;
use mp_ytmusic::{Collection, Library, SearchResults};

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

#[derive(Default)]
pub struct YoutubeView {
    pub rows: Vec<YoutubeRow>,
    pub opened: Option<(String, Vec<YoutubeRow>, usize)>,
    pub error: Option<String>,
}

impl YoutubeView {
    pub fn search(&mut self, results: SearchResults) {
        self.rows = results
            .tracks
            .into_iter()
            .map(YoutubeRow::Track)
            .chain(results.albums.into_iter().map(YoutubeRow::Collection))
            .chain(results.playlists.into_iter().map(YoutubeRow::Collection))
            .collect();
        self.opened = None;
        self.error = None;
    }

    pub fn library(&mut self, library: Library) {
        self.rows = std::iter::once(YoutubeRow::Liked)
            .chain(library.playlists.into_iter().map(YoutubeRow::Collection))
            .chain(library.albums.into_iter().map(YoutubeRow::Collection))
            .collect();
        self.error = None;
    }

    pub fn open(&mut self, title: String, tracks: Vec<Track>, selection: usize) {
        let rows = tracks.into_iter().map(YoutubeRow::Track).collect();
        self.opened = Some((title, std::mem::replace(&mut self.rows, rows), selection));
        self.error = None;
    }

    pub fn back(&mut self) -> Option<usize> {
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
mod tests {
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

    #[test]
    fn search_rows_and_track_queue() {
        let mut view = YoutubeView::default();
        view.search(SearchResults {
            tracks: vec![track("a", "b", "c")],
            albums: vec![collection(Kind::Album)],
            playlists: vec![collection(Kind::Playlist)],
        });
        assert!(matches!(view.rows[0], YoutubeRow::Track(_)));
        assert_eq!(view.rows[1].label().0, "◉ Álbum");
        assert_eq!(view.rows[2].label().0, "≡ Playlist");
        assert_eq!(view.tracks().len(), 1);
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
