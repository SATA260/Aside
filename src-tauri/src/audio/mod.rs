//! 背景音：无缝循环，切换时两秒交叉渐变，音量实时可调。
//! 混音的算法在 `Engine` 里，不碰声卡，测试直接用它。`Player` 把混好的声音送出去，
//! 没有声卡时安静地什么都不播。

pub mod builtin;
pub mod fileloop;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use rodio::Source;

pub const SAMPLE_RATE: u32 = 44_100;
pub const CROSSFADE_SECS: f32 = 2.0;

struct Voice {
    samples: Vec<f32>,
    pos: usize,
    once: bool,
    live: Option<fileloop::LiveFeed>,
}

impl Voice {
    fn clip(samples: Vec<f32>, once: bool) -> Self {
        Self { samples, pos: 0, once, live: None }
    }

    fn next_sample(&mut self) -> f32 {
        if let Some(live) = self.live.as_mut() {
            return live.next_sample();
        }
        if self.samples.is_empty() || (self.once && self.pos >= self.samples.len()) {
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
        self.current = Some(Voice::clip(samples, false));
        self.incoming = None;
    }

    /// 用两秒交叉渐变换成一段新的声音。
    pub fn switch_to(&mut self, samples: Vec<f32>) {
        self.switch_voice(Voice::clip(samples, false));
    }

    /// 换成一个边播边读的长文件，同样用两秒交叉渐变。
    pub fn switch_live(&mut self, live: fileloop::LiveFeed) {
        self.switch_voice(Voice { samples: Vec::new(), pos: 0, once: false, live: Some(live) });
    }

    fn switch_voice(&mut self, voice: Voice) {
        if self.current.is_none() {
            self.current = Some(voice);
            self.incoming = None;
            return;
        }
        self.incoming = Some(voice);
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

/// 到点时的一声：八秒里慢慢响起来。
pub fn chime() -> Vec<f32> {
    let total = SAMPLE_RATE as usize * 8;
    let step = 523.25 / SAMPLE_RATE as f32 * std::f32::consts::TAU;
    let mut phase = 0.0f32;
    (0..total)
        .map(|i| {
            let env = (i as f32 / total as f32).clamp(0.0, 1.0);
            let sample = phase.sin() * env * 0.12;
            phase = (phase + step) % std::f32::consts::TAU;
            sample
        })
        .collect()
}

/// 时间到时可选的一声“嘎嘎”：两下短促的叫声，默认不播。
pub fn quack() -> Vec<f32> {
    let mut out = Vec::new();
    for (freq, gain) in [(380.0_f32, 0.35), (540.0, 0.55)] {
        let count = (SAMPLE_RATE as f32 * 0.16) as usize;
        let step = freq / SAMPLE_RATE as f32 * std::f32::consts::TAU;
        let mut phase = 0.0f32;
        for i in 0..count {
            let env = (i as f32 / count as f32 * std::f32::consts::PI).sin();
            out.push((phase.sin() * env * gain).clamp(-1.0, 1.0));
            phase = (phase + step) % std::f32::consts::TAU;
        }
        out.extend(std::iter::repeat(0.0).take(SAMPLE_RATE as usize / 12));
    }
    out
}

/// 同一声连响三次，一次比一次响。
pub fn alarm() -> Vec<f32> {
    let tone = chime();
    [0.4, 0.7, 1.0]
        .into_iter()
        .flat_map(|gain| tone.iter().map(move |s| s * gain))
        .collect()
}
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

    pub fn play_once(&self, samples: Vec<f32>) {
        let mut engine = self.engine.lock().unwrap_or_else(|e| e.into_inner());
        engine.set_volume(1.0);
        engine.current = Some(Voice::clip(samples, true));
        engine.incoming = None;
    }

    pub fn switch_live(&self, path: std::path::PathBuf, gain_db: f32) {
        let live = fileloop::LiveFeed::open(path, gain_db);
        self.engine.lock().unwrap_or_else(|e| e.into_inner()).switch_live(live);
    }

    pub fn stop(&self) {
        self.engine.lock().unwrap_or_else(|e| e.into_inner()).set_clip(Vec::new());
    }
}
