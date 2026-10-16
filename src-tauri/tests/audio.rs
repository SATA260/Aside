use aside_lib::audio::{CROSSFADE_SECS, Engine, SAMPLE_RATE};

fn clip(value: f32, len: usize) -> Vec<f32> {
    vec![value; len]
}

#[test]
fn a_clip_loops_seamlessly() {
    let mut engine = Engine::new();
    engine.set_clip(clip(0.0, 1).into_iter().chain([0.5, 1.0]).collect());
    let mut out = [0.0; 6];
    engine.render(&mut out);
    assert_eq!(out, [0.0, 0.5, 1.0, 0.0, 0.5, 1.0]);
}

#[test]
fn volume_scales_the_output() {
    let mut engine = Engine::new();
    engine.set_clip(clip(1.0, 4));
    engine.set_volume(0.5);
    let mut out = [0.0; 2];
    engine.render(&mut out);
    assert_eq!(out, [0.5, 0.5]);
}

#[test]
fn switching_clips_crossfades_over_two_seconds() {
    let mut engine = Engine::new();
    engine.set_clip(clip(1.0, 8));
    engine.switch_to(clip(0.0, 8));
    let mut out = [0.0; 4];
    engine.render(&mut out);
    let fade = (SAMPLE_RATE as f32 * CROSSFADE_SECS) as usize;
    let expected: Vec<f32> = (0..4).map(|i| 1.0 - i as f32 / fade as f32).collect();
    for (got, want) in out.iter().zip(&expected) {
        assert!((got - want).abs() < 1e-4, "{got} != {want}");
    }
}

#[test]
fn the_new_clip_plays_alone_once_the_fade_finishes() {
    let mut engine = Engine::new();
    engine.set_clip(clip(1.0, 8));
    engine.switch_to(clip(-0.5, 8));
    let fade = (SAMPLE_RATE as f32 * CROSSFADE_SECS) as usize;
    let mut skip = vec![0.0; fade];
    engine.render(&mut skip);
    let mut out = [0.0; 4];
    engine.render(&mut out);
    assert_eq!(out, [-0.5, -0.5, -0.5, -0.5]);
}

#[test]
fn a_fade_that_ends_mid_buffer_hands_over_cleanly() {
    let mut engine = Engine::new();
    engine.set_clip(clip(1.0, 8));
    engine.switch_to(clip(0.0, 8));
    let fade = (SAMPLE_RATE as f32 * CROSSFADE_SECS) as usize;
    let mut out = vec![0.0; fade + 3];
    engine.render(&mut out);
    assert!(out[fade - 1] > 0.0 && out[fade - 1] < 0.01, "{}", out[fade - 1]);
    assert_eq!(&out[fade..], &[0.0, 0.0, 0.0]);
}

#[test]
fn silence_before_any_clip_and_an_empty_clip_stays_silent() {
    let mut engine = Engine::new();
    let mut out = [1.0; 4];
    engine.render(&mut out);
    assert_eq!(out, [0.0; 4]);
    engine.set_clip(Vec::new());
    engine.render(&mut out);
    assert_eq!(out, [0.0; 4]);
}

#[test]
fn clips_keep_looping_while_fading() {
    let mut engine = Engine::new();
    engine.set_clip(vec![1.0, 0.0]);
    engine.switch_to(vec![0.0, 1.0]);
    let mut out = [0.0; 4];
    engine.render(&mut out);
    let fade = (SAMPLE_RATE as f32 * CROSSFADE_SECS) as usize;
    // 旧的一段在偶数采样点响，新的一段在奇数采样点响
    assert!(out[0] > 0.9, "{}", out[0]);
    assert!(out[1] < 0.01, "{}", out[1]);
    assert!((out[3] - 3.0 / fade as f32).abs() < 1e-4, "{}", out[3]);
}

#[test]
fn silencing_the_alarm_leaves_the_loop_playing() {
    let mut engine = Engine::new();
    engine.set_volume(1.0);
    engine.set_clip(vec![0.25]);
    engine.play_once(vec![0.5, 0.5]);
    let mut out = [0.0; 2];
    engine.render(&mut out);
    assert!((out[0] - 0.75).abs() < 1e-4, "{}", out[0]);
    engine.silence();
    engine.render(&mut out);
    assert!((out[0] - 0.25).abs() < 1e-4, "{}", out[0]);
    assert!((out[1] - 0.25).abs() < 1e-4, "{}", out[1]);
}

#[test]
fn every_builtin_sound_decodes_to_audible_audio() {
    for sound in aside_lib::audio::builtin::list() {
        let samples = aside_lib::audio::builtin::samples(sound);
        let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
        assert!(samples.len() > SAMPLE_RATE as usize, "{} 太短", sound.name);
        assert!(rms > 0.02, "{} 听起来是空的（响度 {rms}）", sound.name);
    }
}

#[test]
fn the_chime_grows_louder_and_the_alarm_rings_three_times() {
    let tone = aside_lib::audio::chime();
    let rms = |slice: &[f32]| (slice.iter().map(|s| s * s).sum::<f32>() / slice.len() as f32).sqrt();
    let early = rms(&tone[..SAMPLE_RATE as usize]);
    let late = rms(&tone[SAMPLE_RATE as usize * 6..SAMPLE_RATE as usize * 7]);
    assert!(late > early * 2.0, "{late} should be much louder than {early}");
    assert_eq!(aside_lib::audio::alarm().len(), tone.len() * 3);
}

#[test]
fn the_quack_is_two_short_calls() {
    let tone = aside_lib::audio::quack();
    let rms = |slice: &[f32]| (slice.iter().map(|s| s * s).sum::<f32>() / slice.len() as f32).sqrt();
    let first = rms(&tone[..(SAMPLE_RATE as f32 * 0.12) as usize]);
    let gap = rms(&tone[(SAMPLE_RATE as f32 * 0.18) as usize..(SAMPLE_RATE as f32 * 0.22) as usize]);
    let second = rms(&tone[(SAMPLE_RATE as f32 * 0.28) as usize..(SAMPLE_RATE as f32 * 0.38) as usize]);
    assert!(first > gap * 3.0, "{first} should be louder than the gap {gap}");
    assert!(second > gap * 3.0, "{second} should be louder than the gap {gap}");
    assert!(aside_lib::settings::Settings::default().quack == false);
}

fn opening_the_player_works_even_without_a_sound_device() {
    let player = aside_lib::audio::Player::open();
    player.set_volume(0.4);
    player.switch_to(clip(0.2, 1000));
    player.stop();
}
