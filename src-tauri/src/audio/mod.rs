//! 背景音：无缝循环，切换时两秒交叉渐变，音量实时可调。
//! 混音的算法在 `Engine` 里，不碰声卡，测试直接用它。`Player` 把混好的声音送出去，
//! 没有声卡时安静地什么都不播。

pub mod builtin;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use rodio::Source;

pub const SAMPLE_RATE: u32 = 44_100;
pub const CROSSFADE_SECS: f32 = 2.0;

struct Voice {
    samples: Vec<f32>,
    pos: usize,
}

impl Voice {
    fn next_sample(&mut self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sample = self.samples[self.pos % self.samples.len()];
        self.pos += 1;
        sample
    }
}

pub struct Engine {
    current: Option<Voice>,
    incoming: Option<Voice>,
    fade_done: usize,
    fade_len: usize,
    volume: f32,
}

impl Engine {
    pub fn new() -> Self {
        Self { current: None, incoming: None, fade_done: 0, fade_len: 0, volume: 1.0 }
    }

    /// 直接换成一段声音，没有渐变。
    pub fn set_clip(&mut self, samples: Vec<f32>) {
        self.current = Some(Voice { samples, pos: 0 });
        self.incoming = None;
    }

    /// 用两秒交叉渐变换成一段新的声音。
    pub fn switch_to(&mut self, samples: Vec<f32>) {
        if self.current.is_none() {
            self.set_clip(samples);
            return;
        }
        self.incoming = Some(Voice { samples, pos: 0 });
        self.fade_done = 0;
        self.fade_len = (SAMPLE_RATE as f32 * CROSSFADE_SECS) as usize;
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    pub fn render(&mut self, out: &mut [f32]) {
        for sample in out {
            *sample = self.sample();
        }
    }

    fn sample(&mut self) -> f32 {
        let current = self.current.as_mut().map_or(0.0, Voice::next_sample);
        let Some(incoming) = self.incoming.as_mut() else {
            return current * self.volume;
        };
        let t = self.fade_done as f32 / self.fade_len as f32;
        let incoming = incoming.next_sample();
        self.fade_done += 1;
        let mixed = (current * (1.0 - t) + incoming * t) * self.volume;
        if self.fade_done >= self.fade_len {
            self.current = self.incoming.take();
        }
        mixed
    }
}

/// 一直从 `Engine` 里取声音的音源。
struct EngineSource {
    engine: Arc<Mutex<Engine>>,
}

impl Iterator for EngineSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        Some(self.engine.lock().unwrap_or_else(|e| e.into_inner()).sample())
    }
}

impl Source for EngineSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

/// 背景音播放器。没有声卡时照样能用，只是没有声音。
pub struct Player {
    engine: Arc<Mutex<Engine>>,
    _output: Option<(rodio::OutputStream, rodio::Sink)>,
}

impl Player {
    pub fn open() -> Self {
        let engine = Arc::new(Mutex::new(Engine::new()));
        let output = rodio::OutputStream::try_default().ok().and_then(|(stream, handle)| {
            let sink = rodio::Sink::try_new(&handle).ok()?;
            sink.append(EngineSource { engine: engine.clone() });
            Some((stream, sink))
        });
        Self { engine, _output: output }
    }

    pub fn set_volume(&self, volume: f32) {
        self.engine.lock().unwrap_or_else(|e| e.into_inner()).set_volume(volume);
    }

    pub fn switch_to(&self, samples: Vec<f32>) {
        self.engine.lock().unwrap_or_else(|e| e.into_inner()).switch_to(samples);
    }

    pub fn stop(&self) {
        self.engine.lock().unwrap_or_else(|e| e.into_inner()).set_clip(Vec::new());
    }
}
