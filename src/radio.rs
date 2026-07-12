use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;

const KEEP_BEHIND: usize = 262144;
const PREBUFFER: usize = 65536;

struct RingData {
    buf: Vec<u8>,
    base: usize,
}

struct StreamBuffer {
    data: Arc<Mutex<RingData>>,
    pos: usize,
    finished: Arc<AtomicBool>,
}

impl Read for StreamBuffer {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        for _ in 0..500 {
            {
                let mut ring = self.data.lock().unwrap();
                let rel = self.pos.saturating_sub(ring.base);
                if rel < ring.buf.len() {
                    let available = &ring.buf[rel..];
                    let n = out.len().min(available.len());
                    out[..n].copy_from_slice(&available[..n]);
                    self.pos += n;

                    let consumed = self.pos - ring.base;
                    if consumed > KEEP_BEHIND {
                        let drop = consumed - KEEP_BEHIND;
                        ring.buf.drain(0..drop);
                        ring.base += drop;
                    }
                    return Ok(n);
                }
            }
            if self.finished.load(Ordering::SeqCst) {
                return Ok(0);
            }
            thread::sleep(Duration::from_millis(20));
        }
        Ok(0)
    }
}

impl Seek for StreamBuffer {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let ring = self.data.lock().unwrap();
        let end = (ring.base + ring.buf.len()) as i64;
        let new_pos = match pos {
            SeekFrom::Start(p) => p as i64,
            SeekFrom::Current(p) => self.pos as i64 + p,
            SeekFrom::End(p) => end + p,
        };
        self.pos = new_pos.max(ring.base as i64) as usize;
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
            .header("Icy-MetaData", "0")
            .header("User-Agent", "chiplay/1.0")
            .send()
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP {}", response.status()));
        }

        let ring = Arc::new(Mutex::new(RingData {
            buf: Vec::with_capacity(PREBUFFER * 2),
            base: 0,
        }));
        let finished = Arc::new(AtomicBool::new(false));

        let ring_writer = ring.clone();
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
                    Ok(n) => ring_writer.lock().unwrap().buf.extend_from_slice(&chunk[..n]),
                    Err(_) => break,
                }
            }
            fin_writer.store(true, Ordering::SeqCst);
        });

        loop {
            if ring.lock().unwrap().buf.len() >= PREBUFFER {
                break;
            }
            if finished.load(Ordering::SeqCst) {
                return Err("Stream ended before enough data".to_string());
            }
            thread::sleep(Duration::from_millis(50));
        }

        let stream_reader = StreamBuffer {
            data: ring,
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
