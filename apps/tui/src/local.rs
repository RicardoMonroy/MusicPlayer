//! Local browsing state. Paths are normalized as strings so Windows libraries also work on Unix.
use crate::app::{Album, fmt_time};
use mp_core::Track;
use ratatui::widgets::TableState;
use std::{collections::BTreeMap, path::Path};

pub fn artist(t: &Track) -> &str {
    t.album_artist.as_deref().unwrap_or(&t.artist)
}
pub fn group(tracks: &[Track]) -> Vec<Album> {
    let mut groups = BTreeMap::<(String, String), Vec<Track>>::new();
    for t in tracks {
        groups
            .entry((artist(t).into(), t.album.clone()))
            .or_default()
            .push(t.clone());
    }
    let mut albums: Vec<_> = groups
        .into_iter()
        .map(|((artist, name), mut tracks)| {
            tracks.sort_by_key(|t| {
                (
                    t.disc_no.unwrap_or(1),
                    t.track_no.unwrap_or(0),
                    t.title.clone(),
                )
            });
            Album {
                artist,
                name,
                tracks,
            }
        })
        .collect();
    sort(&mut albums, Order::Artist);
    albums
}
pub fn year(a: &Album) -> Option<u32> {
    a.tracks.iter().filter_map(|t| t.year).min()
}
pub fn genre(a: &Album) -> String {
    let mut counts = BTreeMap::<&str, usize>::new();
    for g in a.tracks.iter().filter_map(|t| t.genre.as_deref()) {
        *counts.entry(g).or_default() += 1;
    }
    counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(a.0)))
        .map_or_else(|| "Sin género".into(), |(g, _)| g.into())
}
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum Order {
    #[default]
    Artist,
    Year,
    Alphabetic,
    Genre,
}
impl Order {
    pub fn next(self) -> Self {
        match self {
            Self::Artist => Self::Year,
            Self::Year => Self::Alphabetic,
            Self::Alphabetic => Self::Genre,
            Self::Genre => Self::Artist,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Artist => "Artista",
            Self::Year => "Año",
            Self::Alphabetic => "Alfabético",
            Self::Genre => "Género",
        }
    }
}
pub fn sort(albums: &mut [Album], order: Order) {
    albums.sort_by_key(|a| {
        let ar = a.artist.to_lowercase();
        let name = a.name.to_lowercase();
        let y = year(a);
        match order {
            Order::Artist => (ar, y, name),
            Order::Year => (String::new(), y, format!("{ar}\0{name}")),
            Order::Alphabetic => (name, None, ar),
            Order::Genre => (genre(a).to_lowercase(), y, format!("{ar}\0{name}")),
        }
    });
}
pub fn album_label(a: &Album, short: bool) -> String {
    let y = year(a).map_or_else(|| "Sin año".into(), |y| y.to_string());
    if short {
        format!("{y} · {} · {} canciones", a.name, a.tracks.len())
    } else {
        format!(
            "{} — {} · {y} · {} canciones · {} · {}",
            a.name,
            a.artist,
            a.tracks.len(),
            fmt_time(a.tracks.iter().filter_map(|t| t.duration).sum()),
            genre(a)
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Page {
    Songs,
    Search,
    Artists,
    Albums,
    Folders(String),
    Artist(String),
    Album(usize),
}
// Rows are rebuilt per screen and mostly tracks; boxing them buys nothing.
#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum Row {
    Track(Track),
    Artist(String),
    Album(usize),
    Folder(String),
    Disc(u32),
}
#[derive(Clone)]
pub struct Snapshot {
    pub page: Page,
    pub filter: String,
    pub focus: usize,
    pub tables: [TableState; 4],
}
impl Default for Snapshot {
    fn default() -> Self {
        Self {
            page: Page::Songs,
            filter: String::new(),
            focus: 0,
            tables: Default::default(),
        }
    }
}
#[derive(Default)]
pub struct Browser {
    pub results: [Vec<Row>; 4],
    pub state: Snapshot,
    pub history: Vec<Snapshot>,
    pub order: Order,
}
impl Browser {
    pub fn open(&mut self, page: Page, table: TableState, filter: &str) {
        self.state.tables[self.state.focus] = table;
        self.state.filter = filter.into();
        self.history.push(self.state.clone());
        self.state = Snapshot {
            focus: usize::from(matches!(page, Page::Artist(_))),
            page,
            ..Default::default()
        };
        for t in &mut self.state.tables {
            t.select(Some(0));
        }
    }
    pub fn back(&mut self) -> Option<Snapshot> {
        self.state = self.history.pop()?;
        Some(self.state.clone())
    }
    pub fn breadcrumb(&self, albums: &[Album]) -> String {
        let label = |p: &Page| match p {
            Page::Songs => "Canciones".into(),
            Page::Artists => "Artistas".into(),
            Page::Search => "Buscar en local".into(),
            Page::Albums => "Álbumes".into(),
            Page::Folders(s) => format!("Carpetas / {s}"),
            Page::Artist(s) => s.clone(),
            Page::Album(i) => albums.get(*i).map_or_else(String::new, |a| a.name.clone()),
        };
        std::iter::once("Local".into())
            .chain(self.history.iter().map(|s| label(&s.page)))
            .chain([label(&self.state.page)])
            .collect::<Vec<_>>()
            .join(" › ")
    }
}
pub fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

fn normalize(s: &str) -> String {
    s.replace('\\', "/").trim_end_matches('/').to_string()
}
pub fn relative(t: &Track, root: &Path) -> String {
    let path = normalize(t.uri.strip_prefix("file://").unwrap_or(&t.uri));
    let root = normalize(&root.to_string_lossy());
    path.strip_prefix(&format!("{root}/"))
        .unwrap_or(&path)
        .trim_start_matches('/')
        .to_string()
}
pub fn under(t: &Track, root: &Path, folder: &str) -> bool {
    folder.is_empty() || relative(t, root).starts_with(&format!("{folder}/"))
}
pub fn rows(
    page: &Page,
    column: usize,
    library: &[Track],
    albums: &[Album],
    root: &Path,
    filter: &str,
    order: Order,
) -> Vec<Row> {
    let mut rows = match page {
        Page::Search => return search(library, albums, root, filter)[column].clone(),
        Page::Songs => library.iter().cloned().map(Row::Track).collect(),
        Page::Artists => {
            let names: std::collections::BTreeSet<_> =
                albums.iter().map(|a| a.artist.clone()).collect();
            names.into_iter().map(Row::Artist).collect()
        }
        Page::Albums | Page::Artist(_) => {
            let mut indices: Vec<_> = albums
                .iter()
                .enumerate()
                .filter(|(_, a)| match page {
                    Page::Artist(name) => {
                        a.artist == *name || a.tracks.iter().any(|t| t.artist == *name)
                    }
                    _ => true,
                })
                .map(|(i, _)| i)
                .collect();
            indices.sort_by_key(|&i| {
                let a = &albums[i];
                let ar = a.artist.to_lowercase();
                let name = a.name.to_lowercase();
                let y = year(a);
                match if matches!(page, Page::Artist(_)) {
                    Order::Year
                } else {
                    order
                } {
                    Order::Artist => (ar, y, name),
                    Order::Year => (String::new(), y, format!("{ar}\0{name}")),
                    Order::Alphabetic => (name, None, ar),
                    Order::Genre => (genre(a).to_lowercase(), y, format!("{ar}\0{name}")),
                }
            });
            if matches!(page, Page::Artist(_)) && column == 0 {
                indices
                    .into_iter()
                    .flat_map(|i| albums[i].tracks.iter().cloned().map(Row::Track))
                    .collect()
            } else {
                indices.into_iter().map(Row::Album).collect()
            }
        }
        Page::Album(i) => {
            let mut rows = Vec::new();
            if let Some(a) = albums.get(*i) {
                let multi = a
                    .tracks
                    .iter()
                    .map(|t| t.disc_no.unwrap_or(1))
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    > 1;
                let mut disc = None;
                for t in &a.tracks {
                    let d = t.disc_no.unwrap_or(1);
                    if multi && disc != Some(d) {
                        rows.push(Row::Disc(d));
                        disc = Some(d);
                    }
                    rows.push(Row::Track(t.clone()));
                }
            }
            rows
        }
        Page::Folders(folder) => {
            let mut dirs = BTreeMap::<String, usize>::new();
            let mut tracks = Vec::new();
            for t in library.iter().filter(|t| under(t, root, folder)) {
                let rel = relative(t, root);
                let rest = if folder.is_empty() {
                    rel.as_str()
                } else {
                    &rel[folder.len() + 1..]
                };
                if let Some((dir, _)) = rest.split_once('/') {
                    let path = if folder.is_empty() {
                        dir.into()
                    } else {
                        format!("{folder}/{dir}")
                    };
                    *dirs.entry(path).or_default() += 1;
                } else {
                    tracks.push(Row::Track(t.clone()));
                }
            }
            dirs.into_keys().map(Row::Folder).chain(tracks).collect()
        }
    };
    let q = filter.to_lowercase();
    if !q.is_empty() {
        rows.retain(|r| {
            !matches!(r, Row::Disc(_))
                && label(r, albums, library, root).to_lowercase().contains(&q)
        });
    }
    rows
}
pub fn label(row: &Row, albums: &[Album], library: &[Track], root: &Path) -> String {
    match row {
        Row::Track(t) => format!(
            "{} — {} · {}",
            t.title,
            t.artist,
            fmt_time(t.duration.unwrap_or(0.0))
        ),
        Row::Album(i) => album_label(&albums[*i], false),
        Row::Artist(name) => {
            let a: Vec<_> = albums.iter().filter(|a| a.artist == *name).collect();
            format!(
                "{name} · {} · {} canciones",
                plural(a.len(), "álbum", "álbumes"),
                a.iter().map(|a| a.tracks.len()).sum::<usize>()
            )
        }
        Row::Folder(path) => format!(
            "▸ {} · {} canciones",
            path.rsplit('/').next().unwrap_or(path),
            library.iter().filter(|t| under(t, root, path)).count()
        ),
        Row::Disc(d) => format!("Disco {d}"),
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::app::tests::track;

    pub fn discography() -> Vec<Track> {
        let mut a = track("Primera", "Invitado", "Still Life");
        a.album_artist = Some("Opeth".into());
        a.year = Some(1999);
        a.genre = Some("Metal".into());
        a.disc_no = Some(1);
        a.track_no = Some(2);
        a.uri = "/music/Opeth/Still Life/one.flac".into();
        let mut b = a.clone();
        b.title = "Segunda".into();
        b.artist = "Opeth".into();
        b.disc_no = Some(2);
        b.track_no = Some(1);
        b.genre = Some("Rock".into());
        b.uri = "/music/Opeth/Still Life/two.flac".into();
        let mut c = a.clone();
        c.title = "Inicio".into();
        c.track_no = Some(1);
        c.uri = "/music/Opeth/Still Life/start.flac".into();
        let mut d = track("Temprana", "Opeth", "Morningrise");
        d.year = Some(1996);
        d.uri = "/music/Opeth/Morningrise/early.flac".into();
        vec![b, a, d, c]
    }

    #[test]
    fn grouping_album_artist_genre_and_disc_order() {
        let albums = group(&discography());
        assert_eq!(albums.len(), 2);
        assert_eq!(albums[0].name, "Morningrise");
        let album = &albums[1];
        assert_eq!(album.artist, "Opeth");
        assert_eq!(genre(album), "Metal");
        assert_eq!(
            album
                .tracks
                .iter()
                .map(|t| t.title.as_str())
                .collect::<Vec<_>>(),
            ["Inicio", "Primera", "Segunda"]
        );
        let rows = rows(
            &Page::Album(1),
            0,
            &discography(),
            &albums,
            Path::new("/music"),
            "",
            Order::Artist,
        );
        assert!(matches!(rows[0], Row::Disc(1)));
        assert!(matches!(rows[3], Row::Disc(2)));
        let artists = rows_for(&Page::Artists, "");
        assert_eq!(artists.len(), 1);
        assert!(matches!(&artists[0],Row::Artist(n) if n=="Opeth"));
        assert_eq!(
            label(&artists[0], &albums, &discography(), Path::new("/music")),
            "Opeth · 2 álbumes · 4 canciones"
        );
    }
    fn rows_for(page: &Page, filter: &str) -> Vec<Row> {
        let tracks = discography();
        rows(
            page,
            0,
            &tracks,
            &group(&tracks),
            Path::new("/music"),
            filter,
            Order::Artist,
        )
    }
    #[test]
    fn folder_tree_windows_and_unix_and_filter() {
        let mut tracks = discography();
        tracks[0].uri = "\\music\\Opeth\\Still Life\\two.flac".into();
        tracks.push(Track {
            uri: "/music/root.flac".into(),
            title: "Raíz".into(),
            ..Default::default()
        });
        let albums = group(&tracks);
        let list = rows(
            &Page::Folders(String::new()),
            0,
            &tracks,
            &albums,
            Path::new("/music"),
            "",
            Order::Artist,
        );
        assert_eq!(list.len(), 2);
        assert!(matches!(&list[0],Row::Folder(p) if p=="Opeth"));
        assert!(matches!(&list[1],Row::Track(t) if t.title=="Raíz"));
        assert_eq!(
            tracks
                .iter()
                .filter(|t| under(t, Path::new("/music"), "Opeth"))
                .count(),
            4
        );
        assert!(!under(&tracks[0], Path::new("/music"), "Ope"));
        let list = rows_for(&Page::Folders("Opeth".into()), "Still");
        assert_eq!(list.len(), 1);
        assert!(matches!(&list[0],Row::Folder(p) if p=="Opeth/Still Life"));
        let windows = Track {
            uri: "C:\\Music\\Artist\\song.mp3".into(),
            ..Default::default()
        };
        assert_eq!(
            relative(&windows, Path::new("C:\\Music")),
            "Artist/song.mp3"
        );
    }
    #[test]
    fn ordering_cycles_and_album_rows() {
        let mut order = Order::Artist;
        for expected in [Order::Year, Order::Alphabetic, Order::Genre, Order::Artist] {
            order = order.next();
            assert_eq!(order, expected);
        }
        let mut tracks = discography();
        let mut extra = track("Otra", "A", "Zeta");
        extra.year = Some(2005);
        extra.genre = Some("Ambient".into());
        tracks.push(extra);
        let mut albums = group(&tracks);
        assert_eq!(albums[0].name, "Zeta");
        sort(&mut albums, Order::Year);
        assert_eq!(albums[0].name, "Morningrise");
        sort(&mut albums, Order::Alphabetic);
        assert_eq!(albums[0].name, "Morningrise");
        sort(&mut albums, Order::Genre);
        assert_eq!(albums[0].name, "Zeta");
        for order in [Order::Artist, Order::Year, Order::Alphabetic, Order::Genre] {
            let list = rows(
                &Page::Albums,
                0,
                &tracks,
                &albums,
                Path::new("/music"),
                "",
                order,
            );
            let mut expected = group(&tracks);
            sort(&mut expected, order);
            let names: Vec<_> = list
                .iter()
                .map(|r| {
                    if let Row::Album(i) = r {
                        albums[*i].name.as_str()
                    } else {
                        panic!()
                    }
                })
                .collect();
            assert_eq!(
                names,
                expected.iter().map(|a| a.name.as_str()).collect::<Vec<_>>()
            );
        }
    }
    #[test]
    fn navigation_back_restores_focus_filter_selection_and_offset() {
        let mut browser = Browser::default();
        let mut table = TableState::default().with_selected(Some(8));
        *table.offset_mut() = 4;
        browser.open(Page::Artists, table, "busco");
        browser.open(
            Page::Artist("Opeth".into()),
            TableState::default().with_selected(Some(2)),
            "Opeth",
        );
        assert_eq!(browser.state.focus, 1);
        let mut albums = TableState::default().with_selected(Some(1));
        *albums.offset_mut() = 1;
        browser.open(Page::Album(1), albums, "Life");
        let previous = browser.back().unwrap();
        assert_eq!(previous.focus, 1);
        assert_eq!(previous.filter, "Life");
        assert_eq!(previous.tables[1].selected(), Some(1));
        assert_eq!(previous.tables[1].offset(), 1);
        browser.back().unwrap();
        let previous = browser.back().unwrap();
        assert_eq!(previous.page, Page::Songs);
        assert_eq!(previous.filter, "busco");
        assert_eq!(previous.tables[0].selected(), Some(8));
        assert_eq!(previous.tables[0].offset(), 4);
        assert!(browser.back().is_none());
    }
}

/// Fold Latin metadata and decomposed accent marks without changing stored labels.
pub fn fold(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter_map(|c| {
            Some(match c {
                '\u{300}'..='\u{36f}' => return None,
                'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' => 'a',
                'é' | 'è' | 'ê' | 'ë' => 'e',
                'í' | 'ì' | 'î' | 'ï' => 'i',
                'ó' | 'ò' | 'ô' | 'ö' | 'õ' => 'o',
                'ú' | 'ù' | 'û' | 'ü' => 'u',
                'ñ' => 'n',
                'ç' => 'c',
                other => other,
            })
        })
        .collect()
}
fn rank(value: &str, query: &str) -> Option<usize> {
    let value = fold(value);
    if value == query {
        Some(0)
    } else if value.starts_with(query) {
        Some(1)
    } else if value.contains(query) {
        Some(2)
    } else {
        None
    }
}
pub fn search(library: &[Track], albums: &[Album], root: &Path, query: &str) -> [Vec<Row>; 4] {
    let q = fold(query.trim());
    if q.is_empty() {
        return Default::default();
    }
    let mut songs: Vec<_> = library
        .iter()
        .filter_map(|t| {
            let folder = relative(t, root)
                .rsplit_once('/')
                .map_or(String::new(), |(p, _)| p.into());
            [
                t.title.as_str(),
                &t.artist,
                artist(t),
                &t.album,
                t.genre.as_deref().unwrap_or(""),
                &folder,
            ]
            .into_iter()
            .filter_map(|v| rank(v, &q))
            .min()
            .map(|r| (r, Row::Track(t.clone())))
        })
        .collect();
    songs.sort_by_key(|(r, _)| *r);
    let names: std::collections::BTreeSet<_> = albums
        .iter()
        .map(|a| a.artist.clone())
        .chain(library.iter().map(|t| t.artist.clone()))
        .collect();
    let mut artists: Vec<_> = names
        .into_iter()
        .filter_map(|n| rank(&n, &q).map(|r| (r, Row::Artist(n))))
        .collect();
    artists.sort_by_key(|(r, _)| *r);
    let mut discs: Vec<_> = albums
        .iter()
        .enumerate()
        .filter_map(|(i, a)| {
            // Title first, then artist and genre, so "opeth" lists Opeth's albums.
            let genre = a
                .tracks
                .first()
                .and_then(|t| t.genre.as_deref())
                .unwrap_or("");
            [a.name.as_str(), &a.artist, genre]
                .into_iter()
                .filter_map(|v| rank(v, &q))
                .min()
                .map(|r| (r, Row::Album(i)))
        })
        .collect();
    discs.sort_by_key(|(r, _)| *r);
    let mut folders = std::collections::BTreeSet::new();
    for t in library {
        let rel = relative(t, root);
        let mut path = rel.as_str();
        while let Some((parent, _)) = path.rsplit_once('/') {
            folders.insert(parent.to_string());
            path = parent;
        }
    }
    let mut folders: Vec<_> = folders
        .into_iter()
        .filter_map(|p| rank(p.rsplit('/').next().unwrap_or(&p), &q).map(|r| (r, Row::Folder(p))))
        .collect();
    folders.sort_by_key(|(r, _)| *r);
    [songs, artists, discs, folders].map(|rows| rows.into_iter().map(|(_, r)| r).collect())
}
pub fn top(columns: &[Vec<Row>; 4], albums: &[Album], query: &str) -> Option<Row> {
    let q = fold(query.trim());
    columns[1]
        .iter()
        .find(|r| matches!(r,Row::Artist(n) if fold(n)==q))
        .or_else(|| {
            columns[2]
                .iter()
                .find(|r| matches!(r,Row::Album(i) if fold(&albums[*i].name)==q))
        })
        .or_else(|| columns[0].first())
        .cloned()
}

#[cfg(test)]
mod search_tests {
    use super::*;
    use crate::app::tests::track;
    #[test]
    fn ranking_accents_fields_and_top_choice() {
        let mut tracks = vec![
            track("Una canción", "Otro", "Disco"),
            track("Canción larga", "Otro", "Disco"),
            track("CANCIÓN", "Björk", "Álbum"),
        ];
        tracks[0].genre = Some("Electrónica".into());
        tracks[0].album_artist = Some("Ártista".into());
        tracks[0].uri = "/music/Música/uno.flac".into();
        let albums = group(&tracks);
        let root = Path::new("/music");
        let columns = search(&tracks, &albums, root, "cancion");
        let names: Vec<_> = columns[0]
            .iter()
            .map(|r| match r {
                Row::Track(t) => t.title.as_str(),
                _ => panic!(),
            })
            .collect();
        assert_eq!(names, ["CANCIÓN", "Canción larga", "Una canción"]);
        assert_eq!(fold("Mu\u{301}sica"), "musica");
        for q in ["electronica", "artista", "musica"] {
            assert!(!search(&tracks, &albums, root, q)[0].is_empty());
        }
        let c = search(&tracks, &albums, root, "bjork");
        assert!(matches!(top(&c, &albums, "bjork"), Some(Row::Artist(_))));
        let c = search(&tracks, &albums, root, "album");
        assert!(matches!(top(&c, &albums, "album"), Some(Row::Album(_))));
        assert!(
            matches!(top(&columns,&albums,"cancion"),Some(Row::Track(t)) if t.title=="CANCIÓN")
        );
    }
}
