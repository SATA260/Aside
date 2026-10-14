use std::sync::Arc;

use aside_lib::app::Aside;
use aside_lib::clock::ManualClock;
use aside_lib::events::RecordingSink;
use aside_lib::records::ManualEntry;
use aside_lib::shell::RecordingShell;
use aside_lib::stats::due_week_prompt;
use aside_lib::store::subjects;
use chrono::{Datelike, Local, NaiveDate, TimeZone, Weekday};

fn at(d: u32, h: u32, m: u32) -> i64 {
    Local.with_ymd_and_hms(2026, 10, d, h, m, 0).unwrap().timestamp_millis()
}

fn rig(now: i64) -> (tempfile::TempDir, Arc<Aside>) {
    let dir = tempfile::tempdir().unwrap();
    let app = Aside::open(
        dir.path().to_path_buf(),
        Arc::new(ManualClock::new(now)),
        Arc::new(RecordingSink::default()),
        Arc::new(RecordingShell::default()),
    )
    .unwrap();
    (dir, app)
}

fn study(app: &Aside, day: u32) {
    let id = app.store.read(|c| subjects::list(c, false)).unwrap().into_iter().find(|s| s.name == "数学").unwrap().id;
    let start = at(day, 10, 0);
    app.session_add(ManualEntry {
        subject_id: id,
        started_at: start,
        ended_at: start + 30 * 60_000,
        task: String::new(),
        note: String::new(),
    })
    .unwrap();
}

#[test]
fn the_prompt_is_only_due_on_monday_when_last_week_has_time() {
    let monday = NaiveDate::from_ymd_opt(2026, 10, 12).unwrap();
    assert_eq!(monday.weekday(), Weekday::Mon);
    assert!(due_week_prompt(monday, "", 60_000));
    assert!(!due_week_prompt(monday, "2026-10-12", 60_000));
    assert!(!due_week_prompt(monday, "", 0));
    assert!(!due_week_prompt(monday.succ_opt().unwrap(), "", 60_000));
}

#[test]
fn monday_morning_offers_last_week_once() {
    let (_dir, app) = rig(at(12, 9, 0));
    study(&app, 7);
    let prompt = app.week_prompt().unwrap().unwrap();
    assert_eq!(prompt.from, NaiveDate::from_ymd_opt(2026, 10, 5).unwrap());
    assert_eq!(prompt.to, NaiveDate::from_ymd_opt(2026, 10, 11).unwrap());
    assert!(app.week_prompt().unwrap().is_none());

    let (_dir, quiet) = rig(at(12, 9, 0));
    assert!(quiet.week_prompt().unwrap().is_none());
    assert_eq!(quiet.settings().week_prompted, "");

    let (_dir, tuesday) = rig(at(13, 9, 0));
    study(&tuesday, 7);
    assert!(tuesday.week_prompt().unwrap().is_none());
}
