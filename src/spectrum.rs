use rodio::Source;
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::collections::VecDeque;
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const RING_CAPACITY: usize = 4096;
const FFT_SIZE: usize = 2048;

pub type SharedSamples = Arc<Mutex<VecDeque<f32>>>;

pub fn shared() -> SharedSamples {
    Arc::new(Mutex::new(VecDeque::with_capacity(RING_CAPACITY)))
}

pub fn clear(buf: &SharedSamples) {
    buf.lock().unwrap().clear();
}

pub struct SpectrumTap<S> {
    inner: S,
    buf: SharedSamples,
    channels: u16,
    frame_pos: u16,
}

pub fn tap<S>(inner: S, buf: SharedSamples) -> SpectrumTap<S>
where
    S: Source<Item = f32>,
{
    let channels = inner.channels().max(1);
    SpectrumTap {
        inner,
        buf,
        channels,
        frame_pos: 0,
    }
}

impl<S> Iterator for SpectrumTap<S>
where
    S: Source<Item = f32>,
{
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let sample = self.inner.next()?;
        if self.frame_pos == 0 {
            let mut ring = self.buf.lock().unwrap();
            if ring.len() >= RING_CAPACITY {
                ring.pop_front();
            }
            ring.push_back(sample);
        }
        self.frame_pos = (self.frame_pos + 1) % self.channels;
        Some(sample)
    }
}

impl<S> Source for SpectrumTap<S>
where
    S: Source<Item = f32>,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }
    fn channels(&self) -> u16 {
        self.inner.channels()
    }
    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }
    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}

pub fn compute_bars(buf: &SharedSamples, n_bars: usize) -> Vec<f32> {
    if n_bars == 0 {
        return Vec::new();
    }

    let samples: Vec<f32> = {
        let ring = buf.lock().unwrap();
        if ring.len() < FFT_SIZE {
            return vec![0.0; n_bars];
        }
        ring.iter().rev().take(FFT_SIZE).rev().copied().collect()
    };

    let mut input: Vec<Complex<f32>> = samples
        .iter()
        .enumerate()
        .map(|(i, &s)| {
            let w = 0.5 - 0.5 * (2.0 * PI * i as f32 / (FFT_SIZE as f32 - 1.0)).cos();
            Complex::new(s * w, 0.0)
        })
        .collect();

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(FFT_SIZE);
    fft.process(&mut input);

    let bins = FFT_SIZE / 2;
    let mut bars = vec![0.0f32; n_bars];
    for b in 0..n_bars {
        let lo = ((b as f32 / n_bars as f32).powf(2.2) * bins as f32) as usize;
        let hi = (((b + 1) as f32 / n_bars as f32).powf(2.2) * bins as f32) as usize;
        let hi = hi.max(lo + 1).min(bins);
        let mut peak = 0.0f32;
        for bin in &input[lo..hi] {
            peak = peak.max(bin.norm());
        }
        let db = (peak + 1e-6).log10();
        bars[b] = ((db + 3.0) / 4.0).clamp(0.0, 1.0);
    }
    bars
}
