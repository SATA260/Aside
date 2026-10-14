//! 用户导入的素材：背景音、背景图片的校验、复制、响度测量和压暗。

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use image::imageops::FilterType;
use image::RgbaImage;
use rodio::Source;

use crate::audio::SAMPLE_RATE;
use crate::error::{AppResult, invalid};
use crate::store::{Store, media};

pub const MAX_SOUND_BYTES: u64 = 200 * 1024 * 1024;
pub const MAX_IMAGE_BYTES: u64 = 30 * 1024 * 1024;
pub const TEN_MIN_MS: i64 = 10 * 60 * 1000;
/// 导入的声音往这个响度上靠，大概和内置环境音一样，不会突然很吵。
pub const TARGET_RMS: f32 = 0.05;
const NAME_MAX: usize = 20;
const BACKDROP_EDGE: u32 = 1600;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopMode {
    /// 十分钟以内：整段解码后无缝循环
    Seamless,
    /// 更长：边播边读，首尾交叉
    Stream,
}

pub fn loop_mode(duration_ms: i64) -> LoopMode {
    if duration_ms <= TEN_MIN_MS { LoopMode::Seamless } else { LoopMode::Stream }
}

pub fn sound_error(ext: &str, len: u64) -> Option<String> {
    let ext = normalize_ext(ext);
    if !matches!(ext.as_str(), "mp3" | "m4a" | "wav" | "flac") {
        return Some(format!("不支持 {} 格式，可以先转成 mp3", shown_ext(&ext)));
    }
    if len == 0 {
        return Some("这个文件是空的".into());
    }
    if len > MAX_SOUND_BYTES {
        return Some("这个声音超过 200MB，换一个小一点的文件".into());
    }
    None
}

pub fn image_error(ext: &str, len: u64) -> Option<String> {
    let ext = normalize_ext(ext);
    if !matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "webp") {
        return Some(format!("不支持 {} 格式，可以换 jpg、png 或 webp", shown_ext(&ext)));
    }
    if len == 0 {
        return Some("这个文件是空的".into());
    }
    if len > MAX_IMAGE_BYTES {
        return Some("这张图片超过 30MB，换一张小一点的".into());
    }
    None
}

fn normalize_ext(ext: &str) -> String {
    ext.trim().trim_start_matches('.').to_ascii_lowercase()
}

fn shown_ext(ext: &str) -> String {
    if ext.is_empty() { "这个".into() } else { format!(".{ext}") }
}

pub fn extension_of(path: &Path) -> String {
    path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase()
}

pub fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let energy = samples.iter().map(|s| (*s as f64) * (*s as f64)).sum::<f64>() / samples.len() as f64;
    energy.sqrt() as f32
}

/// 把一段声音的响度对齐到 [`TARGET_RMS`]，返回要加的分贝。太安静时最多抬 24 分贝。
pub fn match_gain_db(level: f32) -> f32 {
    if level <= 1.0e-5 {
        return 0.0;
    }
    let db = 20.0 * (TARGET_RMS / level).log10();
    db.clamp(-24.0, 24.0)
}

pub fn gain_factor(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}

pub fn apply_gain(samples: &mut [f32], db: f32) {
    let g = gain_factor(db);
    for s in samples {
        *s = (*s * g).clamp(-1.0, 1.0);
    }
}

/// 长音频回到开头时，结尾和开头按这个比例叠在一起。`t` 从 0 到接近 1。
pub fn mix_loop_point(tail: &[f32], head: &[f32]) -> Vec<f32> {
    let n = tail.len().min(head.len());
    (0..n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let tail_s = tail[tail.len() - n + i];
            tail_s * (1.0 - t) + head[i] * t
        })
        .collect()
}

/// 缩小、模糊、压暗，让白色的时间在照片上仍然看得清。
pub fn prepare_backdrop(src: &RgbaImage) -> RgbaImage {
    let (w, h) = src.dimensions();
    let long = w.max(h).max(1);
    let fitted = if long > BACKDROP_EDGE {
        let scale = BACKDROP_EDGE as f32 / long as f32;
        image::imageops::resize(src, ((w as f32) * scale) as u32, ((h as f32) * scale) as u32, FilterType::Triangle)
    } else {
        src.clone()
    };
    let mut out = image::imageops::blur(&fitted, 1.2);
    for pixel in out.pixels_mut() {
        pixel[0] = (pixel[0] as u16 * 45 / 100) as u8;
        pixel[1] = (pixel[1] as u16 * 45 / 100) as u8;
        pixel[2] = (pixel[2] as u16 * 45 / 100) as u8;
    }
    out
}

pub fn clean_name(raw: &str) -> AppResult<String> {
    let name = raw.trim();
    let n = name.chars().count();
    if n == 0 {
        return invalid("名字不能是空的");
    }
    if n > NAME_MAX {
        return invalid("名字最多 20 个字");
    }
    Ok(name.to_string())
}

fn file_stem_name(path: &Path) -> String {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("未命名");
    let name: String = stem.trim().chars().take(NAME_MAX).collect();
    if name.is_empty() { "未命名".into() } else { name }
}

struct Probe {
    duration_ms: i64,
    level: f32,
}

fn open_decoder(path: &Path) -> AppResult<rodio::Decoder<BufReader<File>>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let decoded = if extension_of(path) == "m4a" {
        rodio::Decoder::new_mp4(reader, rodio::decoder::Mp4Type::M4a)
    } else {
        rodio::Decoder::new(reader)
    };
    decoded.map_err(|_| {
        crate::error::AppError::Invalid("这个文件读不出来，换一个 mp3、m4a、wav 或 flac".into())
    })
}

/// 读出时长和响度。超过十分钟就不再把采样留在内存里，只把前 30 秒拿来测响度。
fn probe(path: &Path) -> AppResult<Probe> {
    let mut dec = open_decoder(path)?;
    let channels = dec.channels().max(1) as usize;
    let rate = dec.sample_rate().max(1) as i64;
    let limit = rate * 10 * 60;
    let rms_cap = (rate * 30) as usize;
    let mut frames = 0i64;
    let mut energy = 0.0f64;
    let mut counted = 0usize;
    loop {
        let mut acc = 0.0f32;
        let mut got = 0;
        for _ in 0..channels {
            let Some(sample) = dec.next() else {
                if got == 0 && frames == 0 {
                    return invalid("这个文件里没有声音");
                }
                if got == 0 {
                    let level = if counted == 0 { 0.0 } else { (energy / counted as f64).sqrt() as f32 };
                    let duration_ms = frames * 1000 / rate;
                    return Ok(Probe { duration_ms, level });
                }
                break;
            };
            acc += sample as f32 / 32768.0;
            got += 1;
        }
        if got == 0 {
            break;
        }
        frames += 1;
        if counted < rms_cap {
            let sample = acc / got as f32;
            energy += (sample as f64) * (sample as f64);
            counted += 1;
        }
        if frames > limit {
            let level = (energy / counted.max(1) as f64).sqrt() as f32;
            return Ok(Probe { duration_ms: TEN_MIN_MS + 1, level });
        }
    }
    if frames == 0 {
        return invalid("这个文件里没有声音");
    }
    let level = if counted == 0 { 0.0 } else { (energy / counted as f64).sqrt() as f32 };
    Ok(Probe { duration_ms: frames * 1000 / rate, level })
}

/// 整段解成 44100 单声道。只用于十分钟以内的声音。
pub fn decode_loop(path: &Path, gain_db: f32) -> AppResult<Vec<f32>> {
    let mut dec = open_decoder(path)?;
    let channels = dec.channels().max(1) as usize;
    let rate = dec.sample_rate().max(1);
    let mut native = Vec::new();
    loop {
        let mut acc = 0.0f32;
        let mut got = 0;
        for _ in 0..channels {
            let Some(sample) = dec.next() else { break };
            acc += sample as f32 / 32768.0;
            got += 1;
        }
        if got == 0 {
            break;
        }
        native.push(acc / got as f32);
        if native.len() > SAMPLE_RATE as usize * 10 * 60 + 1 {
            return invalid("这段声音超过 10 分钟，要边播边读");
        }
    }
    if native.is_empty() {
        return invalid("这个文件里没有声音");
    }
    let mut samples = resample(&native, rate, SAMPLE_RATE);
    apply_gain(&mut samples, gain_db);
    Ok(samples)
}

pub fn resample(samples: &[f32], from: u32, to: u32) -> Vec<f32> {
    if samples.is_empty() || from == 0 || to == 0 || from == to {
        return samples.to_vec();
    }
    let out_len = ((samples.len() as u64) * (to as u64) / (from as u64)).max(1) as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src = i as f64 * from as f64 / to as f64;
        let i0 = (src.floor() as usize).min(samples.len() - 1);
        let i1 = (i0 + 1).min(samples.len() - 1);
        let t = (src - i0 as f64) as f32;
        out.push(samples[i0] * (1.0 - t) + samples[i1] * t);
    }
    out
}

fn fresh_rel(folder: &str, ext: &str) -> String {
    static N: AtomicU64 = AtomicU64::new(0);
    format!(
        "{folder}/{}-{}-{}.{ext}",
        std::process::id(),
        crate::store::now_ms(),
        N.fetch_add(1, Ordering::Relaxed)
    )
}

fn copy_into(data_dir: &Path, folder: &str, src: &Path, ext: &str) -> AppResult<String> {
    let dir = data_dir.join(folder);
    std::fs::create_dir_all(&dir)?;
    let rel = fresh_rel(folder, ext);
    std::fs::copy(src, data_dir.join(&rel))?;
    Ok(rel)
}

pub fn import_sound(store: &Store, data_dir: &Path, src: &Path, name: Option<&str>, match_loudness: bool) -> AppResult<media::Sound> {
    let ext = extension_of(src);
    let len = std::fs::metadata(src).map(|m| m.len()).unwrap_or(0);
    if let Some(msg) = sound_error(&ext, len) {
        return invalid(msg);
    }
    let name = match name {
        Some(name) => clean_name(name)?,
        None => file_stem_name(src),
    };
    let probed = probe(src)?;
    let gain_db = if match_loudness { match_gain_db(probed.level) as f64 } else { 0.0 };
    let rel = copy_into(data_dir, "sounds", src, &ext)?;
    let id = match store.write(|c| media::insert_sound(c, &name, &rel, gain_db, probed.duration_ms)) {
        Ok(id) => id,
        Err(err) => {
            let _ = std::fs::remove_file(data_dir.join(&rel));
            return Err(err);
        }
    };
    Ok(media::Sound {
        id,
        name,
        path: data_dir.join(&rel).to_string_lossy().into_owned(),
        gain_db,
        duration_ms: probed.duration_ms,
    })
}

pub fn import_picture(store: &Store, data_dir: &Path, src: &Path, name: Option<&str>) -> AppResult<media::Picture> {
    let ext = extension_of(src);
    let len = std::fs::metadata(src).map(|m| m.len()).unwrap_or(0);
    if let Some(msg) = image_error(&ext, len) {
        return invalid(msg);
    }
    let name = match name {
        Some(name) => clean_name(name)?,
        None => file_stem_name(src),
    };
    let opened = image::open(src).map_err(|_| crate::error::AppError::Invalid("这张图片读不出来".into()))?;
    let prepared = prepare_backdrop(&opened.to_rgba8());
    let dir = data_dir.join("scenes");
    std::fs::create_dir_all(&dir)?;
    let rel = fresh_rel("scenes", "png");
    let dest = data_dir.join(&rel);
    prepared.save(&dest).map_err(|e| crate::error::AppError::Io(std::io::Error::other(e)))?;
    let id = match store.write(|c| media::insert_picture(c, &name, &rel)) {
        Ok(id) => id,
        Err(err) => {
            let _ = std::fs::remove_file(&dest);
            return Err(err);
        }
    };
    Ok(media::Picture { id, name, path: dest.to_string_lossy().into_owned() })
}

pub fn user_sound_id(ambient: &str) -> Option<i64> {
    ambient.strip_prefix("s:").and_then(|rest| rest.parse().ok())
}

pub fn remove_stored_file(data_dir: &Path, rel: &str) {
    let path = data_dir.join(rel);
    if path.starts_with(data_dir) {
        let _ = std::fs::remove_file(path);
    }
}

pub fn picture_bytes(path: &PathBuf) -> AppResult<Vec<u8>> {
    Ok(std::fs::read(path)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ten_minutes_loops_in_memory_and_anything_longer_streams() {
        assert_eq!(loop_mode(TEN_MIN_MS), LoopMode::Seamless);
        assert_eq!(loop_mode(TEN_MIN_MS - 1), LoopMode::Seamless);
        assert_eq!(loop_mode(TEN_MIN_MS + 1), LoopMode::Stream);
    }

    #[test]
    fn unsupported_or_huge_files_say_why() {
        assert!(sound_error("ape", 10).unwrap().contains("不支持 .ape"));
        assert!(sound_error("wav", MAX_SOUND_BYTES).is_none());
        assert!(sound_error("wav", MAX_SOUND_BYTES + 1).unwrap().contains("200MB"));
        assert!(sound_error("mp3", 0).unwrap().contains("空的"));
        assert!(image_error("gif", 10).unwrap().contains("不支持 .gif"));
        assert!(image_error("jpg", 10).is_none());
        assert!(image_error("JPEG", 10).is_none());
    }

    #[test]
    fn a_loud_clip_is_turned_down_and_a_quiet_one_is_lifted() {
        let loud = rms(&[0.5; 1000]);
        let quiet = rms(&[0.005; 1000]);
        assert!(match_gain_db(loud) < 0.0);
        assert!(match_gain_db(quiet) > 0.0);
        let mut samples = vec![0.5; 8];
        apply_gain(&mut samples, match_gain_db(loud));
        assert!((samples[0] - TARGET_RMS).abs() < 0.01, "{}", samples[0]);
    }

    #[test]
    fn the_loop_point_starts_at_the_tail_and_ends_near_the_head() {
        let mixed = mix_loop_point(&[0.0, 0.0], &[1.0, 1.0]);
        assert!((mixed[0] - 0.0).abs() < 0.001);
        assert!(mixed[1] > 0.4 && mixed[1] < 0.6, "{mixed:?}");
    }

    #[test]
    fn a_white_picture_becomes_dark_enough_to_read_white_text() {
        let src = image::RgbaImage::from_pixel(8, 8, image::Rgba([255, 255, 255, 255]));
        let out = prepare_backdrop(&src);
        let pixel = out.get_pixel(4, 4);
        assert!(pixel[0] < 160 && pixel[1] < 160 && pixel[2] < 160, "{pixel:?}");
        assert!(pixel[0] > 40, "{pixel:?}");
    }

    #[test]
    fn names_reject_blank_and_anything_past_twenty_characters() {
        assert!(clean_name("   ").is_err());
        assert!(clean_name("一二三四五六七八九十一二三四五六七八九十一").is_err());
        assert_eq!(clean_name(" 老电扇 ").unwrap(), "老电扇");
    }
}
