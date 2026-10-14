//! 超过十分钟的背景音不整段放进内存：边读边播，回到开头时和前两秒交叉。

use std::collections::VecDeque;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use rodio::Source;

use super::SAMPLE_RATE;

pub struct LiveFeed {
    queue: Arc<Mutex<VecDeque<f32>>>,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl LiveFeed {
    pub fn open(path: PathBuf, gain_db: f32) -> Self {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let q = queue.clone();
        let flag = stop.clone();
        let handle = std::thread::spawn(move || {
            let Ok(mut cursor) = Cursor::open(&path, gain_db, SAMPLE_RATE as usize * 2) else {
                return;
            };
            while !flag.load(Ordering::Relaxed) {
                let sample = cursor.next_sample();
                let mut buf = q.lock().unwrap_or_else(|e| e.into_inner());
                if buf.len() >= SAMPLE_RATE as usize * 2 {
                    drop(buf);
                    std::thread::sleep(Duration::from_millis(20));
                    continue;
                }
                buf.push_back(sample);
            }
        });
        let start = std::time::Instant::now();
        while queue.lock().unwrap_or_else(|e| e.into_inner()).len() < 2048 && start.elapsed() < Duration::from_millis(800) {
            std::thread::sleep(Duration::from_millis(5));
        }
        Self { queue, stop, handle: Some(handle) }
    }

    pub fn next_sample(&self) -> f32 {
        self.queue.lock().unwrap_or_else(|e| e.into_inner()).pop_front().unwrap_or(0.0)
    }
}

impl Drop for LiveFeed {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// 把一段短文件循环 `count` 个采样，用来核对首尾交叉。淡化长度由调用方指定。
pub fn render_loop(path: &Path, fade: usize, count: usize) -> Result<Vec<f32>, String> {
    let mut cursor = Cursor::open(path, 0.0, fade)?;
    Ok((0..count).map(|_| cursor.next_sample()).collect())
}

struct Pull {
    dec: rodio::Decoder<BufReader<File>>,
    channels: usize,
    step: f64,
    acc: f64,
    last: f32,
}

impl Pull {
    fn open(path: &Path) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let dec = rodio::Decoder::new(BufReader::new(file)).map_err(|_| "这个文件读不出来".to_string())?;
        let channels = dec.channels().max(1) as usize;
        let rate = dec.sample_rate().max(1);
        Ok(Self { dec, channels, step: rate as f64 / SAMPLE_RATE as f64, acc: 0.0, last: 0.0 })
    }

    fn next_out(&mut self) -> Option<f32> {
        self.acc += self.step;
        let mut produced = false;
        while self.acc >= 1.0 {
            self.last = self.frame()?;
            produced = true;
            self.acc -= 1.0;
        }
        if produced || self.step < 1.0 { Some(self.last) } else { None }
    }

    fn frame(&mut self) -> Option<f32> {
        let mut acc = 0.0f32;
        for _ in 0..self.channels {
            acc += self.dec.next()? as f32 / 32768.0;
        }
        Some(acc / self.channels as f32)
    }
}

struct Cursor {
    path: PathBuf,
    gain: f32,
    fade: usize,
    head: Vec<f32>,
    ahead: VecDeque<f32>,
    pull: Pull,
    eof: bool,
    fading: bool,
    fade_i: usize,
}

impl Cursor {
    fn open(path: &Path, gain_db: f32, fade: usize) -> Result<Self, String> {
        let fade = fade.max(1);
        let gain = 10f32.powf(gain_db / 20.0);
        let mut priming = Pull::open(path)?;
        let mut head = Vec::new();
        for _ in 0..fade {
            match priming.next_out() {
                Some(sample) => head.push(sample * gain),
                None => break,
            }
        }
        if head.is_empty() {
            return Err("这个文件里没有声音".into());
        }
        Ok(Self {
            path: path.to_path_buf(),
            gain,
            fade,
            head,
            ahead: VecDeque::new(),
            pull: Pull::open(path)?,
            eof: false,
            fading: false,
            fade_i: 0,
        })
    }

    fn next_sample(&mut self) -> f32 {
        if self.fading {
            return self.fade_one();
        }
        while self.ahead.len() < self.fade && !self.eof {
            match self.pull.next_out() {
                Some(sample) => self.ahead.push_back(sample * self.gain),
                None => self.eof = true,
            }
        }
        if self.eof {
            self.begin_fade();
            return self.fade_one();
        }
        match self.pull.next_out() {
            Some(sample) => {
                let out = self.ahead.pop_front().unwrap_or(0.0);
                self.ahead.push_back(sample * self.gain);
                out
            }
            None => {
                self.eof = true;
                self.begin_fade();
                self.fade_one()
            }
        }
    }

    fn begin_fade(&mut self) {
        self.fading = true;
        self.fade_i = 0;
        self.eof = false;
        if let Ok(mut pull) = Pull::open(&self.path) {
            for _ in 0..self.head.len() {
                if pull.next_out().is_none() {
                    break;
                }
            }
            self.pull = pull;
        }
    }

    fn fade_one(&mut self) -> f32 {
        let n = self.head.len().max(1) as f32;
        let t = self.fade_i as f32 / n;
        let tail = self.ahead.pop_front().unwrap_or(0.0);
        let head = self.head.get(self.fade_i).copied().unwrap_or(0.0);
        self.fade_i += 1;
        if self.fade_i >= self.head.len() {
            self.fading = false;
            self.fade_i = 0;
        }
        (tail * (1.0 - t) + head * t).clamp(-1.0, 1.0)
    }
}
