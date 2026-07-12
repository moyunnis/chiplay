use crate::spectrum::{self, SharedSamples, SpectrumTap};
use rodio::source::SamplesConverter;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::panic;
use std::path::PathBuf;
use std::time::Duration;

type TrackSource = SpectrumTap<SamplesConverter<Decoder<BufReader<File>>, f32>>;

pub struct Player {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Sink,
    duration: Option<Duration>,
    samples: SharedSamples,
}

impl Player {
    pub fn new() -> Self {
        let (stream, handle) = OutputStream::try_default().expect("No audio output device");
        let sink = Sink::try_new(&handle).expect("Failed to create sink");
        sink.set_volume(0.5);
        Player {
            _stream: stream,
            handle,
            sink,
            duration: None,
            samples: spectrum::shared(),
        }
    }

    pub fn samples(&self) -> SharedSamples {
        self.samples.clone()
    }

    pub fn load(&mut self, path: &PathBuf) -> Result<(), String> {
        let vol = self.sink.volume();
        self.sink.stop();
        self.sink = Sink::try_new(&self.handle).map_err(|e| e.to_string())?;
        self.sink.set_volume(vol);
        spectrum::clear(&self.samples);

        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);

        let decode = panic::catch_unwind(panic::AssertUnwindSafe(|| Decoder::new(reader)));
        let decoder = match decode {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => return Err(format!("Decode error: {}", e)),
            Err(_) => return Err("Unsupported format".to_string()),
        };

        self.duration = decoder.total_duration();
        let source: TrackSource = spectrum::tap(decoder.convert_samples::<f32>(), self.samples.clone());
        self.sink.append(source);
        Ok(())
    }

    pub fn toggle_pause(&self) {
        if self.sink.is_paused() {
            self.sink.play();
        } else {
            self.sink.pause();
        }
    }

    pub fn play(&self) {
        self.sink.play();
    }

    pub fn pause(&self) {
        self.sink.pause();
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    pub fn stop(&self) {
        self.sink.stop();
        spectrum::clear(&self.samples);
    }

    pub fn is_empty(&self) -> bool {
        self.sink.empty()
    }

    pub fn volume(&self) -> f32 {
        self.sink.volume()
    }

    pub fn set_volume(&self, vol: f32) {
        self.sink.set_volume(vol.clamp(0.0, 1.0));
    }

    pub fn volume_up(&self) {
        self.set_volume(self.volume() + 0.05);
    }

    pub fn volume_down(&self) {
        self.set_volume(self.volume() - 0.05);
    }

    pub fn position(&self) -> Duration {
        self.sink.get_pos()
    }

    pub fn seek_to(&self, pos: Duration) {
        let _ = self.sink.try_seek(pos);
    }

    pub fn seek_forward(&self, secs: u64) {
        let pos = self.position() + Duration::from_secs(secs);
        if let Some(dur) = self.duration {
            if pos < dur {
                self.seek_to(pos);
            }
        } else {
            self.seek_to(pos);
        }
    }

    pub fn seek_backward(&self, secs: u64) {
        let pos = self.position();
        if pos > Duration::from_secs(secs) {
            self.seek_to(pos - Duration::from_secs(secs));
        } else {
            self.seek_to(Duration::ZERO);
        }
    }

    pub fn duration(&self) -> Option<Duration> {
        self.duration
    }
}
