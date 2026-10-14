use std::sync::Arc;

use aside_lib::app::{Aside, StartArgs};
use aside_lib::clock::ManualClock;
use aside_lib::events::RecordingSink;
use aside_lib::shell::RecordingShell;
use aside_lib::store::subjects;
use aside_lib::timer::{Display, Mode};
use serde_json::json;

fn rig() -> (tempfile::TempDir, Arc<Aside>, Arc<RecordingShell>, Arc<RecordingSink>) {
    let dir = tempfile::tempdir().unwrap();
    let shell = Arc::new(RecordingShell::default());
    let events = Arc::new(RecordingSink::default());
    let app = Aside::open(
        dir.path().to_path_buf(),
        Arc::new(ManualClock::new(1_760_000_000_000)),
        events.clone(),
        shell.clone(),
    )
    .unwrap();
    (dir, app, shell, events)
}

fn start(app: &Aside) {
    let id = app.store.read(|c| subjects::list(c, false)).unwrap().into_iter().find(|s| s.is_fallback).unwrap().id;
    app.timer_start(StartArgs {
        subject_id: id,
        mode: Mode::Countdown,
        planned_ms: Some(45 * 60_000),
        task: String::new(),
        display: Display::Fullscreen,
    })
    .unwrap();
}

#[test]
fn a_session_runs_the_open_shortcut_and_then_the_close_one() {
    let (_dir, app, shell, events) = rig();
    app.settings_update(json!({ "dnd_on": "打开专注模式", "dnd_off": "关闭专注模式" })).unwrap();
    start(&app);
    let calls = shell.calls();
    if std::env::consts::OS == "macos" {
        assert!(calls.iter().any(|c| c == "focus:打开专注模式"), "{calls:?}");
        app.timer_finish().unwrap();
        let calls = shell.calls();
        assert!(calls.iter().any(|c| c == "focus:关闭专注模式"), "{calls:?}");
    } else if std::env::consts::OS == "windows" {
        let hint = events.last("dnd:hint").unwrap();
        assert_eq!(hint["message"], "可以按 Win+N 打开勿扰");
        app.settings_update(json!({ "dnd_hint": false })).unwrap();
        events.events.lock().unwrap().clear();
        app.timer_finish().unwrap();
        start(&app);
        assert!(events.last("dnd:hint").is_none());
    } else {
        assert!(!calls.iter().any(|c| c.starts_with("focus:")));
    }
}

#[test]
fn a_missing_shortcut_is_told_on_the_start_page() {
    let (_dir, app, shell, events) = rig();
    shell.fail_focus();
    app.settings_update(json!({ "dnd_on": "打开专注模式", "dnd_off": "关闭专注模式" })).unwrap();
    start(&app);
    if std::env::consts::OS == "macos" {
        let message = events.last("dnd:error").unwrap()["message"].as_str().unwrap().to_string();
        assert!(message.contains("找不到名为‘打开专注模式’的快捷指令"), "{message}");
    } else {
        assert!(events.last("dnd:error").is_none());
    }
}
