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
    pub tracks: Vec<PathBuf>,
    pub cursor: usize,
    pub playing_index: Option<usize>,
    pub shuffle: bool,
    pub repeat: RepeatMode,
    pub tab: Tab,
    pub radio_index: usize,
    pub running: bool,
    pub radio_playing: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new(tracks: Vec<PathBuf>) -> Self {
        App {
            tracks,
            cursor: 0,
            playing_index: None,
            shuffle: false,
            repeat: RepeatMode::Off,
            tab: Tab::Tracks,
            radio_index: 0,
            running: true,
            radio_playing: false,
            status_message: None,
        }
    }

    pub fn playing_track(&self) -> Option<&PathBuf> {
        self.playing_index.and_then(|i| self.tracks.get(i))
    }

    pub fn playing_name(&self) -> String {
        self.playing_track()
            .map(|p| {
                p.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            })
            .unwrap_or_else(|| "No track".to_string())
    }

    pub fn advance_track(&mut self) {
        if self.tracks.is_empty() {
            return;
        }
        let current = self.playing_index.unwrap_or(0);
        let next = if self.shuffle {
            use rand::Rng;
            rand::thread_rng().gen_range(0..self.tracks.len())
        } else {
            (current + 1) % self.tracks.len()
        };
        self.playing_index = Some(next);
        self.cursor = next;
    }

    pub fn retreat_track(&mut self) {
        if self.tracks.is_empty() {
            return;
        }
        let current = self.playing_index.unwrap_or(0);
        let prev = if current == 0 {
            self.tracks.len() - 1
        } else {
            current - 1
        };
        self.playing_index = Some(prev);
        self.cursor = prev;
    }

    pub fn play_at_cursor(&mut self) {
        if self.cursor < self.tracks.len() {
            self.playing_index = Some(self.cursor);
        }
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
    }

    pub fn toggle_repeat(&mut self) {
        self.repeat = self.repeat.next();
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
                if self.cursor + 1 < self.tracks.len() {
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
}
