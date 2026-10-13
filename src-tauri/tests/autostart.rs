use std::sync::Arc;

use aside_lib::app::Aside;
use aside_lib::autostart::launched_at_login;
use aside_lib::clock::ManualClock;
use aside_lib::error::AppError;
use aside_lib::events::RecordingSink;
use aside_lib::shell::RecordingShell;
use tempfile::TempDir;

struct Rig {
    _dir: TempDir,
    shell: Arc<RecordingShell>,
    app: Arc<Aside>,
}

fn rig() -> Rig {
    let dir = tempfile::tempdir().unwrap();
    let clock = Arc::new(ManualClock::new(1_760_000_000_000));
    let shell = Arc::new(RecordingShell::default());
    let app = Aside::open(dir.path().to_path_buf(), clock, Arc::new(RecordingSink::default()), shell.clone()).unwrap();
    Rig { _dir: dir, shell, app }
}

#[test]
fn autostart_is_off_until_turned_on() {
    let r = rig();
    assert!(!r.app.autostart_enabled());

    assert!(r.app.autostart_set(true).unwrap());
    assert!(r.shell.autostart());
    assert!(r.app.autostart_enabled());

    assert!(!r.app.autostart_set(false).unwrap());
    assert!(!r.shell.autostart());
}

#[test]
fn autostart_the_system_refuses_says_so_and_stays_as_it_was() {
    let r = rig();
    r.shell.refuse_autostart();
    match r.app.autostart_set(true) {
        Err(AppError::Invalid(msg)) => assert!(msg.contains("开机自启"), "{msg}"),
        other => panic!("应该提示没打开，实际是 {other:?}"),
    }
    assert!(!r.app.autostart_enabled());
}

#[test]
fn only_a_launch_at_login_stays_in_the_tray() {
    assert!(launched_at_login(["/Applications/Aside.app/Contents/MacOS/aside", "--autostart"]));
    assert!(!launched_at_login(["/Applications/Aside.app/Contents/MacOS/aside"]));
    assert!(!launched_at_login(["aside", "--autostarted"]));
}
