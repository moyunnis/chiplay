use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig};
use std::sync::mpsc::{self, Receiver};

pub use souvlaki::MediaControlEvent as Event;

pub struct Mpris {
    controls: MediaControls,
    rx: Receiver<MediaControlEvent>,
}

impl Mpris {
    pub fn new() -> Option<Self> {
        let config = PlatformConfig {
            dbus_name: "chiplay",
            display_name: "chiplay",
            hwnd: None,
        };
        let mut controls = MediaControls::new(config).ok()?;
        let (tx, rx) = mpsc::channel();
        controls
            .attach(move |event| {
                let _ = tx.send(event);
            })
            .ok()?;
        Some(Mpris { controls, rx })
    }

    pub fn poll(&self) -> Vec<Event> {
        self.rx.try_iter().collect()
    }

    pub fn set_metadata(&mut self, title: &str, artist: Option<&str>) {
        let _ = self.controls.set_metadata(MediaMetadata {
            title: Some(title),
            artist,
            ..Default::default()
        });
    }

    pub fn set_playing(&mut self, playing: bool) {
        let state = if playing {
            MediaPlayback::Playing { progress: None }
        } else {
            MediaPlayback::Paused { progress: None }
        };
        let _ = self.controls.set_playback(state);
    }

    pub fn set_stopped(&mut self) {
        let _ = self.controls.set_playback(MediaPlayback::Stopped);
    }
}
