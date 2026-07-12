use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;

struct StreamBuffer {
    data: Arc<Mutex<Vec<u8>>>,
    pos: usize,
    finished: Arc<AtomicBool>,
}

impl Read for StreamBuffer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        for _ in 0..500 {
            let data = self.data.lock().unwrap();
            if self.pos < data.len() {
                let available = &data[self.pos..];
                let to_read = buf.len().min(available.len());
                buf[..to_read].copy_from_slice(&available[..to_read]);
                self.pos += to_read;
                return Ok(to_read);
            }
            if self.finished.load(Ordering::SeqCst) {
                return Ok(0);
            }
            drop(data);
            thread::sleep(Duration::from_millis(20));
        }
        Ok(0)
    }
}

impl Seek for StreamBuffer {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let len = self.data.lock().unwrap().len() as i64;
        let new_pos = match pos {
            SeekFrom::Start(p) => p as i64,
            SeekFrom::Current(p) => self.pos as i64 + p,
            SeekFrom::End(p) => len + p,
        };
        self.pos = new_pos.max(0) as usize;
        Ok(self.pos as u64)
    }
}

pub struct RadioPlayer {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Sink,
    stop_flag: Arc<AtomicBool>,
    pub playing: bool,
    pub station_name: String,
}

impl RadioPlayer {
    pub fn new() -> Self {
        let (stream, handle) = OutputStream::try_default().expect("No audio output device");
        let sink = Sink::try_new(&handle).expect("Failed to create sink");
        sink.set_volume(0.5);
        RadioPlayer {
            _stream: stream,
            handle,
            sink,
            stop_flag: Arc::new(AtomicBool::new(false)),
            playing: false,
            station_name: String::new(),
        }
    }

    pub fn play_url(&mut self, url: &str, name: &str) -> Result<(), String> {
        self.stop();
        self.stop_flag = Arc::new(AtomicBool::new(false));

        let client = reqwest::blocking::Client::builder()
            .timeout(None)
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;

        let response = client
            .get(url)
            .header("Icy-MetaData", "1")
            .header("User-Agent", "chiplay/0.2")
            .send()
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP {}", response.status()));
        }

        let shared_buf = Arc::new(Mutex::new(Vec::with_capacity(262144)));
        let finished = Arc::new(AtomicBool::new(false));

        let buf_writer = shared_buf.clone();
        let fin_writer = finished.clone();
        let stop_writer = self.stop_flag.clone();

        thread::spawn(move || {
            let mut reader = response;
            let mut chunk = [0u8; 8192];
            loop {
                if stop_writer.load(Ordering::SeqCst) {
                    break;
                }
                match reader.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(n) => buf_writer.lock().unwrap().extend_from_slice(&chunk[..n]),
                    Err(_) => break,
                }
            }
            fin_writer.store(true, Ordering::SeqCst);
        });

        loop {
            if shared_buf.lock().unwrap().len() >= 65536 {
                break;
            }
            if finished.load(Ordering::SeqCst) {
                return Err("Stream ended before enough data".to_string());
            }
            thread::sleep(Duration::from_millis(50));
        }

        let stream_reader = StreamBuffer {
            data: shared_buf,
            pos: 0,
            finished,
        };

        let source = Decoder::new(stream_reader).map_err(|e| format!("Decode error: {}", e))?;

        let vol = self.sink.volume();
        self.sink = Sink::try_new(&self.handle).map_err(|e| e.to_string())?;
        self.sink.set_volume(vol);
        self.sink.append(source);

        self.playing = true;
        self.station_name = name.to_string();
        Ok(())
    }

    pub fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        self.sink.stop();
        self.sink = Sink::try_new(&self.handle).unwrap_or_else(|_| {
            let (stream, handle) = OutputStream::try_default().expect("No audio device");
            self._stream = stream;
            self.handle = handle;
            Sink::try_new(&self.handle).expect("Failed to create sink")
        });
        self.playing = false;
        self.station_name.clear();
    }

    pub fn volume(&self) -> f32 {
        self.sink.volume()
    }

    pub fn set_volume(&self, vol: f32) {
        self.sink.set_volume(vol.clamp(0.0, 1.0));
    }

    pub fn toggle_pause(&self) {
        if self.sink.is_paused() {
            self.sink.play();
        } else {
            self.sink.pause();
        }
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }
}
