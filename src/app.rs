use crate::track::Track;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq)]
pub enum RepeatMode {
    Off,
    One,
    All,
}

impl RepeatMode {
    pub fn next(self) -> Self {
        match self {
            RepeatMode::Off => RepeatMode::One,
            RepeatMode::One => RepeatMode::All,
            RepeatMode::All => RepeatMode::Off,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            RepeatMode::Off => "Off",
            RepeatMode::One => "One",
            RepeatMode::All => "All",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Tracks,
    Radio,
}

pub struct App {
    pub tracks: Vec<Track>,
    pub filtered: Vec<usize>,
    pub cursor: usize,
    pub playing_index: Option<usize>,
    pub shuffle: bool,
    pub repeat: RepeatMode,
    pub tab: Tab,
    pub radio_index: usize,
    pub running: bool,
    pub radio_playing: bool,
    pub show_viz: bool,
    pub search_mode: bool,
    pub query: String,
    pub status_message: Option<String>,
}

impl App {
    pub fn new(tracks: Vec<Track>) -> Self {
        let filtered = (0..tracks.len()).collect();
        App {
            tracks,
            filtered,
            cursor: 0,
            playing_index: None,
            shuffle: false,
            repeat: RepeatMode::Off,
            tab: Tab::Tracks,
            radio_index: 0,
            running: true,
            radio_playing: false,
            show_viz: true,
            search_mode: false,
            query: String::new(),
            status_message: None,
        }
    }

    pub fn playing_track(&self) -> Option<&Track> {
        self.playing_index.and_then(|i| self.tracks.get(i))
    }

    pub fn playing_path(&self) -> Option<PathBuf> {
        self.playing_track().map(|t| t.path.clone())
    }

    pub fn playing_name(&self) -> String {
        self.playing_track()
            .map(|t| t.display())
            .unwrap_or_else(|| "No track".to_string())
    }

    pub fn all_paths(&self) -> Vec<PathBuf> {
        self.tracks.iter().map(|t| t.path.clone()).collect()
    }

    pub fn apply_filter(&mut self) {
        if self.query.is_empty() {
            self.filtered = (0..self.tracks.len()).collect();
        } else {
            let q = self.query.to_lowercase();
            self.filtered = self
                .tracks
                .iter()
                .enumerate()
                .filter(|(_, t)| t.display().to_lowercase().contains(&q))
                .map(|(i, _)| i)
                .collect();
        }
        if self.cursor >= self.filtered.len() {
            self.cursor = self.filtered.len().saturating_sub(1);
        }
    }

    fn playing_pos(&self) -> Option<usize> {
        let pi = self.playing_index?;
        self.filtered.iter().position(|&x| x == pi)
    }

    pub fn advance_track(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        let next = if self.shuffle {
            use rand::Rng;
            rand::thread_rng().gen_range(0..self.filtered.len())
        } else {
            match self.playing_pos() {
                Some(pos) => (pos + 1) % self.filtered.len(),
                None => 0,
            }
        };
        self.playing_index = Some(self.filtered[next]);
        self.cursor = next;
    }

    pub fn retreat_track(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        let prev = match self.playing_pos() {
            Some(0) => self.filtered.len() - 1,
            Some(pos) => pos - 1,
            None => 0,
        };
        self.playing_index = Some(self.filtered[prev]);
        self.cursor = prev;
    }

    pub fn has_next(&self) -> bool {
        match self.playing_pos() {
            Some(pos) => pos + 1 < self.filtered.len(),
            None => false,
        }
    }

    pub fn play_at_cursor(&mut self) {
        if let Some(&idx) = self.filtered.get(self.cursor) {
            self.playing_index = Some(idx);
        }
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
    }

    pub fn toggle_repeat(&mut self) {
        self.repeat = self.repeat.next();
    }

    pub fn toggle_viz(&mut self) {
        self.show_viz = !self.show_viz;
    }

    pub fn scroll_up(&mut self) {
        match self.tab {
            Tab::Tracks => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            Tab::Radio => {
                if self.radio_index > 0 {
                    self.radio_index -= 1;
                }
            }
        }
    }

    pub fn scroll_down(&mut self, max_radio: usize) {
        match self.tab {
            Tab::Tracks => {
                if self.cursor + 1 < self.filtered.len() {
                    self.cursor += 1;
                }
            }
            Tab::Radio => {
                if self.radio_index + 1 < max_radio {
                    self.radio_index += 1;
                }
            }
        }
    }

    pub fn toggle_tab(&mut self) {
        self.tab = match self.tab {
            Tab::Tracks => Tab::Radio,
            Tab::Radio => Tab::Tracks,
        };
    }

    pub fn start_search(&mut self) {
        self.search_mode = true;
    }

    pub fn push_query(&mut self, c: char) {
        self.query.push(c);
        self.apply_filter();
    }

    pub fn pop_query(&mut self) {
        self.query.pop();
        self.apply_filter();
    }

    pub fn end_search(&mut self, clear: bool) {
        self.search_mode = false;
        if clear {
            self.query.clear();
            self.apply_filter();
        }
    }
}
