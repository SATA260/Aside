use std::sync::Arc;

use aside_lib::app::{Aside, StartArgs};
use aside_lib::clock::ManualClock;
use aside_lib::events::RecordingSink;
use aside_lib::shell::RecordingShell;
use aside_lib::store::subjects;
use aside_lib::timer::{Display, Mode};
use chrono::{Local, TimeZone};

const MIN: i64 = 60_000;

fn local(d: u32, h: u32, m: u32) -> i64 {
    Local.with_ymd_and_hms(2026, 10, d, h, m, 0).unwrap().timestamp_millis()
}

fn setup(now: i64) -> (tempfile::TempDir, Arc<ManualClock>, Arc<Aside>) {
    let dir = tempfile::tempdir().unwrap();
    let clock = Arc::new(ManualClock::new(now));
    let app = Aside::open(
        dir.path().to_path_buf(),
        clock.clone(),
        Arc::new(RecordingSink::default()),
        Arc::new(RecordingShell::default()),
    )
    .unwrap();
    (dir, clock, app)
}

fn countup(app: &Aside) {
    let other = app.store.read(subjects::fallback).unwrap().id;
    app.timer_start(StartArgs {
        subject_id: other,
        mode: Mode::Countup,
        planned_ms: None,
        task: String::new(),
        display: Display::Fullscreen,
    })
    .unwrap();
}

#[test]
fn today_counts_finished_sessions_and_the_running_one() {
    let (_dir, clock, app) = setup(local(13, 10, 0));
    countup(&app);
    clock.set(local(13, 10, 30));
    app.timer_finish().unwrap();
    app.timer_close(false).unwrap();

    clock.set(local(13, 11, 50));
    countup(&app);
    clock.set(local(13, 12, 0));
    assert_eq!(app.timer_view().today_ms, 40 * MIN);
}

#[test]
fn a_session_across_midnight_only_counts_its_part_after_midnight() {
    let (_dir, clock, app) = setup(local(12, 23, 30));
    countup(&app);
    clock.set(local(13, 0, 30));
    app.timer_finish().unwrap();
    app.timer_close(false).unwrap();

    clock.set(local(13, 9, 0));
    assert_eq!(app.timer_view().today_ms, 30 * MIN);
}

#[test]
fn again_flag_is_set_by_closing_with_again_and_cleared_by_the_next_start() {
    let (_dir, clock, app) = setup(local(13, 10, 0));
    countup(&app);
    clock.set(local(13, 10, 5));
    app.timer_finish().unwrap();

    let view = app.timer_close(true).unwrap();
    assert!(view.again);
    assert!(view.last.is_some(), "再来一段要知道上一段的科目");

    countup(&app);
    assert!(!app.timer_view().again);
}
