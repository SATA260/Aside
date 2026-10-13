use std::sync::Arc;

use aside_lib::app::Aside;
use aside_lib::clock::ManualClock;
use aside_lib::error::AppError;
use aside_lib::events::RecordingSink;
use aside_lib::shell::RecordingShell;
use aside_lib::timer::Phase;
use serde_json::json;
use tempfile::TempDir;

const T0: i64 = 1_760_000_000_000;

struct Rig {
    dir: TempDir,
    clock: Arc<ManualClock>,
    shell: Arc<RecordingShell>,
    app: Arc<Aside>,
}

fn rig() -> Rig {
    let dir = tempfile::tempdir().unwrap();
    let clock = Arc::new(ManualClock::new(T0));
    let shell = Arc::new(RecordingShell::default());
    let app = Aside::open(dir.path().to_path_buf(), clock.clone(), Arc::new(RecordingSink::default()), shell.clone()).unwrap();
    Rig { dir, clock, shell, app }
}

fn refused(result: Result<impl std::fmt::Debug, AppError>) -> String {
    match result {
        Err(AppError::Invalid(msg)) => msg,
        other => panic!("应该被拒绝，实际是 {other:?}"),
    }
}

#[test]
fn defaults_are_bound_at_startup() {
    let r = rig();
    r.app.shortcuts_bind_all();
    assert_eq!(r.shell.bound("toggle").as_deref(), Some("CommandOrControl+Alt+S"));
    assert_eq!(r.shell.bound("display").as_deref(), Some("CommandOrControl+Alt+M"));
}

#[test]
fn changing_a_shortcut_rebinds_it_and_is_remembered() {
    let r = rig();
    r.app.shortcuts_bind_all();
    let s = r.app.shortcut_set("toggle", " Control+Shift+F ").unwrap();
    assert_eq!(s.shortcut_toggle, "Control+Shift+F");
    assert_eq!(r.shell.bound("toggle").as_deref(), Some("Control+Shift+F"));

    let reopened = Aside::open(r.dir.path().to_path_buf(), r.clock.clone(), Arc::new(RecordingSink::default()), r.shell.clone()).unwrap();
    assert_eq!(reopened.settings().shortcut_toggle, "Control+Shift+F");
}

#[test]
fn shortcut_taken_by_another_app_is_refused_and_the_old_one_kept() {
    let r = rig();
    r.app.shortcuts_bind_all();
    r.shell.take("Control+Shift+F");
    let msg = refused(r.app.shortcut_set("toggle", "Control+Shift+F"));
    assert!(msg.contains("占用"), "{msg}");
    assert_eq!(r.app.settings().shortcut_toggle, "CommandOrControl+Alt+S");
    assert_eq!(r.shell.bound("toggle").as_deref(), Some("CommandOrControl+Alt+S"));
}

#[test]
fn both_actions_cannot_share_one_shortcut() {
    let r = rig();
    let msg = refused(r.app.shortcut_set("display", "CommandOrControl+Alt+S"));
    assert!(msg.contains("重复"), "{msg}");
    // 写法不同但其实是同一个组合
    refused(r.app.shortcut_set("display", "cmdorctrl+option+KeyS"));
    assert_eq!(r.app.settings().shortcut_display, "CommandOrControl+Alt+M");
}

#[test]
fn shortcut_needs_a_modifier_and_one_ordinary_key() {
    let r = rig();
    // 只按 Shift 的话，打大写字母时会被抢走
    for bad in ["S", "", "Alt+Shift", "Ctrl+", "Ctrl+Banana", "Alt+S+D", "Shift+S"] {
        refused(r.app.shortcut_set("toggle", bad));
    }
    refused(r.app.shortcut_set("lunch", "Alt+L"));
    assert_eq!(r.app.settings().shortcut_toggle, "CommandOrControl+Alt+S");
    r.app.shortcut_set("toggle", "Shift+Control+S").unwrap();
}

#[test]
fn shortcuts_rest_while_a_new_one_is_recorded() {
    let r = rig();
    r.app.shortcuts_bind_all();
    r.app.shortcuts_pause(true);
    assert!(r.shell.shortcuts_paused());
    r.app.shortcuts_pause(false);
    assert!(!r.shell.shortcuts_paused());

    r.app.shortcuts_pause(true);
    r.app.shortcut_set("display", "Alt+Shift+M").unwrap();
    assert!(!r.shell.shortcuts_paused(), "录好之后恢复");

    r.app.shortcuts_pause(true);
    r.shell.take("Alt+Shift+K");
    refused(r.app.shortcut_set("display", "Alt+Shift+K"));
    assert!(!r.shell.shortcuts_paused(), "没录成也恢复");
}

#[test]
fn settings_update_cannot_bypass_binding() {
    let r = rig();
    r.app.settings_update(json!({ "shortcut_toggle": "Alt+X", "sfx_volume": 0.2 })).unwrap();
    let s = r.app.settings();
    assert_eq!((s.shortcut_toggle.as_str(), s.sfx_volume), ("CommandOrControl+Alt+S", 0.2));
}

#[test]
fn pressing_toggle_starts_with_the_last_choices_and_ends() {
    let r = rig();
    r.app.shortcut_pressed("toggle").unwrap();
    assert_eq!(r.app.timer_view().snapshot.phase, Phase::Running);
    r.clock.set(T0 + 5 * 60_000);
    r.app.shortcut_pressed("toggle").unwrap();
    assert_eq!(r.app.timer_view().snapshot.phase, Phase::Finished);
}

#[test]
fn pressing_display_flips_only_during_a_session() {
    let r = rig();
    r.app.shortcut_pressed("display").unwrap();
    assert!(r.shell.calls().is_empty());
    r.app.shortcut_pressed("toggle").unwrap();
    r.app.shortcut_pressed("display").unwrap();
    assert_eq!(r.shell.calls()[2..], ["open:mini", "close:overlay"]);
}
