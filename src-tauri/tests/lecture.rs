use std::sync::Arc;

use aside_lib::app::Aside;
use aside_lib::clock::ManualClock;
use aside_lib::events::RecordingSink;
use aside_lib::shell::RecordingShell;

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
