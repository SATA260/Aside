use std::sync::Arc;

use aside_lib::app::{Aside, StartArgs};
use aside_lib::clock::ManualClock;
use aside_lib::commands::dispatch;
use aside_lib::events::RecordingSink;
use aside_lib::shell::RecordingShell;
use aside_lib::store::subjects;
use aside_lib::timer::{Display, Mode, Phase};
use serde_json::json;

fn rig() -> (tempfile::TempDir, Arc<Aside>, Arc<RecordingShell>) {
    let dir = tempfile::tempdir().unwrap();
    let shell = Arc::new(RecordingShell::default());
    let app = Aside::open(
        dir.path().to_path_buf(),
        Arc::new(ManualClock::new(1_760_000_000_000)),
        Arc::new(RecordingSink::default()),
        shell.clone(),
    )
    .unwrap();
    (dir, app, shell)
}

#[test]
fn opening_a_course_remembers_the_address() {
    let (_dir, app, shell) = rig();
    let url = app.lecture_go("bilibili.com").unwrap();
    assert_eq!(url, "https://bilibili.com");
    assert_eq!(app.settings().lecture_url, "https://bilibili.com");
    assert!(shell.calls().iter().any(|c| c == "course:https://bilibili.com"));
    assert!(app.lecture_go("javascript:alert(1)").is_err());
    assert_eq!(app.settings().lecture_url, "https://bilibili.com");
}

#[test]
fn video_only_injects_a_script_that_can_be_cleared() {
    let (_dir, app, shell) = rig();
    app.lecture_video(true, "32:48");
    let script = shell.calls().into_iter().find(|c| c.starts_with("eval:")).unwrap();
    assert!(script.contains("32:48"));
    assert!(script.contains("这个页面上没有找到正在播放的视频"));
    assert!(app.is_video_only());
    app.lecture_video(false, "");
    assert!(!app.is_video_only());
    assert!(shell.calls().last().unwrap().contains("aside-video"));
}

#[test]
fn a_paused_video_pauses_the_timer_only_when_asked() {
    let (_dir, app, _shell) = rig();
    let id = app.store.read(|c| subjects::list(c, false)).unwrap().into_iter().find(|s| s.is_fallback).unwrap().id;
    app.timer_start(StartArgs {
        subject_id: id,
        mode: Mode::Countdown,
        planned_ms: Some(45 * 60_000),
        task: String::new(),
        display: Display::Fullscreen,
    })
    .unwrap();
    app.video_playing(Some(false), false).unwrap();
    assert_eq!(app.timer_view().snapshot.phase, Phase::Running);
    app.settings_update(json!({ "video_pause": true })).unwrap();
    app.video_playing(Some(false), false).unwrap();
    assert_eq!(app.timer_view().snapshot.phase, Phase::Paused);
    app.video_playing(Some(true), false).unwrap();
    assert_eq!(app.timer_view().snapshot.phase, Phase::Running);
    assert!(dispatch(&app, "timer_finish", serde_json::Value::Null, "course").is_err());
    assert!(dispatch(&app, "video_playing", json!({ "exit": true }), "course").is_ok());
    assert!(!app.is_video_only());
}
