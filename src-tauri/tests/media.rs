use std::path::Path;
use std::sync::Arc;

use aside_lib::app::Aside;
use aside_lib::audio::fileloop::render_loop;
use aside_lib::clock::ManualClock;
use aside_lib::error::AppError;
use aside_lib::events::RecordingSink;
use aside_lib::media::{MAX_SOUND_BYTES, TARGET_RMS};
use aside_lib::shell::RecordingShell;
use aside_lib::store::media;
use serde_json::json;

fn rig() -> (tempfile::TempDir, Arc<Aside>) {
    let dir = tempfile::tempdir().unwrap();
    let app = Aside::open(
        dir.path().to_path_buf(),
        Arc::new(ManualClock::new(1_760_000_000_000)),
        Arc::new(RecordingSink::default()),
        Arc::new(RecordingShell::default()),
    )
    .unwrap();
    (dir, app)
}

fn write_wav(path: &Path, samples: &[f32]) {
    let pcm: Vec<i16> = samples.iter().map(|s| (s.clamp(-1.0, 1.0) * 32767.0) as i16).collect();
    let data_len = (pcm.len() * 2) as u32;
    let mut bytes = Vec::new();
    bytes.extend(b"RIFF");
    bytes.extend(&(36 + data_len).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(&16u32.to_le_bytes());
    bytes.extend(&1u16.to_le_bytes());
    bytes.extend(&1u16.to_le_bytes());
    bytes.extend(&44100u32.to_le_bytes());
    bytes.extend(&(44100u32 * 2).to_le_bytes());
    bytes.extend(&2u16.to_le_bytes());
    bytes.extend(&16u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend(&data_len.to_le_bytes());
    for sample in pcm {
        bytes.extend(&sample.to_le_bytes());
    }
    std::fs::write(path, bytes).unwrap();
}

fn assert_invalid(result: Result<impl std::fmt::Debug, AppError>, needle: &str) {
    match result {
        Err(AppError::Invalid(msg)) => assert!(msg.contains(needle), "提示“{msg}”里应该包含“{needle}”"),
        other => panic!("应该被拒绝，实际是 {other:?}"),
    }
}

#[test]
fn an_imported_sound_survives_deleting_the_original_and_can_be_renamed() {
    let (dir, app) = rig();
    let src = dir.path().join("老电扇.wav");
    write_wav(&src, &[0.5; 4410]);
    let sound = app.import_sound(&src, None).unwrap();
    assert_eq!(sound.name, "老电扇");
    assert!(sound.gain_db < 0.0, "响的录音应该被调低，实际 {}", sound.gain_db);
    assert!(sound.duration_ms >= 90 && sound.duration_ms <= 110, "{}", sound.duration_ms);
    std::fs::remove_file(&src).unwrap();
    assert!(Path::new(&sound.path).exists());
    assert_eq!(app.settings().ambient, format!("s:{}", sound.id));

    app.rename_sound(sound.id, "宿舍风扇").unwrap();
    assert_eq!(app.sounds().unwrap()[0].name, "宿舍风扇");
    assert_invalid(app.rename_sound(sound.id, " "), "空的");
    assert_invalid(app.rename_sound(sound.id, &"字".repeat(21)), "20");
    assert_eq!(app.sounds().unwrap()[0].name, "宿舍风扇");

    app.delete_sound(sound.id).unwrap();
    assert!(app.sounds().unwrap().is_empty());
    assert_eq!(app.settings().ambient, "");
    assert!(!Path::new(&sound.path).exists());
}

#[test]
fn turning_off_loudness_matching_keeps_the_original_level() {
    let (dir, app) = rig();
    app.settings_update(json!({ "loudness_match": false })).unwrap();
    let src = dir.path().join("quiet.wav");
    write_wav(&src, &[0.01; 2000]);
    let sound = app.import_sound(&src, Some("轻")).unwrap();
    assert_eq!(sound.gain_db, 0.0);
    let _ = TARGET_RMS;
}

#[test]
fn a_bad_extension_an_empty_file_and_a_broken_mp3_are_refused() {
    let (dir, app) = rig();
    let ape = dir.path().join("song.ape");
    std::fs::write(&ape, b"not audio").unwrap();
    assert_invalid(app.import_sound(&ape, None), "不支持 .ape");

    let empty = dir.path().join("empty.wav");
    std::fs::write(&empty, b"").unwrap();
    assert_invalid(app.import_sound(&empty, None), "空的");

    let junk = dir.path().join("junk.mp3");
    std::fs::write(&junk, b"this is not an mp3").unwrap();
    assert_invalid(app.import_sound(&junk, None), "读不出来");
    assert!(app.sounds().unwrap().is_empty());
    let _ = MAX_SOUND_BYTES;
}

#[test]
fn an_imported_photo_is_darkened_and_deleting_it_clears_the_background() {
    let (dir, app) = rig();
    let src = dir.path().join("教室.png");
    let white = image::RgbaImage::from_pixel(12, 12, image::Rgba([255, 255, 255, 255]));
    white.save(&src).unwrap();
    let picture = app.import_picture(&src, None).unwrap();
    assert_eq!(picture.name, "教室");
    assert_eq!(app.settings().scene_image, Some(picture.id));
    std::fs::remove_file(&src).unwrap();
    let saved = image::open(&picture.path).unwrap().to_rgba8();
    let pixel = saved.get_pixel(6, 6);
    assert!(pixel[0] < 160, "{pixel:?}");

    let gif = dir.path().join("anim.gif");
    std::fs::write(&gif, b"GIF89a").unwrap();
    assert_invalid(app.import_picture(&gif, None), "不支持 .gif");

    app.delete_picture(picture.id).unwrap();
    assert!(app.settings().scene_image.is_none());
    assert!(app.pictures().unwrap().is_empty());
}

#[test]
fn a_long_enough_clip_crossfades_its_tail_into_its_head() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ramp.wav");
    let samples: Vec<f32> = (0..1000).map(|i| i as f32 / 1000.0).collect();
    write_wav(&path, &samples);
    let out = render_loop(&path, 100, 1100).unwrap();
    assert!((out[899] - 0.899).abs() < 0.02, "淡化前应还是原来的采样 {}", out[899]);
    assert!(out[950] < 0.7, "交叉处应该明显低于原来的 0.95，实际 {}", out[950]);
    assert!((out[1000] - 0.1).abs() < 0.02, "绕回去之后应从第 100 个采样继续，实际 {}", out[1000]);
}

#[test]
fn the_copied_row_can_be_read_back_from_the_database() {
    let (dir, app) = rig();
    let src = dir.path().join("fire.wav");
    write_wav(&src, &[0.2; 1000]);
    let sound = app.import_sound(&src, Some("篝火二")).unwrap();
    let listed = app.store.read(|c| media::list_sounds(c, dir.path())).unwrap();
    assert_eq!(listed[0].id, sound.id);
    assert_eq!(listed[0].name, "篝火二");
    assert!(listed[0].path.contains("sounds"));
}
