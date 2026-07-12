use lofty::file::TaggedFileExt;
use lofty::prelude::*;
use std::path::{Path, PathBuf};

pub struct Track {
    pub path: PathBuf,
    pub title: String,
    pub artist: Option<String>,
}

impl Track {
    /// Build a track, reading Artist/Title from tags and falling back to the file name.
    pub fn from_path(path: PathBuf) -> Self {
        let (title, artist) = read_tags(&path);
        let title = title.unwrap_or_else(|| filename_stem(&path));
        Track {
            path,
            title,
            artist,
        }
    }

    /// Line shown in the track list and now-playing header.
    pub fn display(&self) -> String {
        match &self.artist {
            Some(a) if !a.is_empty() => format!("{} — {}", a, self.title),
            _ => self.title.clone(),
        }
    }
}

fn filename_stem(path: &Path) -> String {
    path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

fn read_tags(path: &Path) -> (Option<String>, Option<String>) {
    let Ok(tagged) = lofty::read_from_path(path) else {
        return (None, None);
    };
    let tag = tagged.primary_tag().or_else(|| tagged.first_tag());
    match tag {
        Some(t) => (
            t.title().map(|s| s.to_string()).filter(|s| !s.is_empty()),
            t.artist().map(|s| s.to_string()).filter(|s| !s.is_empty()),
        ),
        None => (None, None),
    }
}
