use std::sync::Arc;

use aside_lib::app::{Aside, StartArgs};
use aside_lib::clock::ManualClock;
use aside_lib::events::RecordingSink;
use aside_lib::shell::RecordingShell;
use aside_lib::store::subjects;
use aside_lib::timer::{Display, Mode};
use chrono::{Local, NaiveDate, TimeZone};

const MIN: i64 = 60_000;

fn at(m: u32, d: u32, h: u32, min: u32) -> i64 {
    Local.with_ymd_and_hms(2026, m, d, h, min, 0).unwrap().timestamp_millis()
}

fn date(m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, m, d).unwrap()
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

fn subject(app: &Aside, name: &str) -> i64 {
    app.subjects_list(false).unwrap().into_iter().find(|s| s.name == name).unwrap().id
}

/// 用计时器记下一段从 `start` 开始、学了 `minutes` 分钟的专注。
fn record(app: &Aside, clock: &ManualClock, subject_id: i64, start: i64, minutes: i64) {
    clock.set(start);
    app.timer_start(StartArgs {
        subject_id,
        mode: Mode::Countup,
        planned_ms: None,
        task: String::new(),
        display: Display::Fullscreen,
    })
    .unwrap();
    clock.set(start + minutes * MIN);
    app.timer_finish().unwrap();
    app.timer_close(false).unwrap();
}

#[test]
fn overview_lists_sixteen_weeks_of_days_up_to_today() {
    let (_dir, _clock, app) = setup(at(10, 14, 20, 0));
    let o = app.stats_overview().unwrap();
    assert_eq!(o.today, date(10, 14));
    assert_eq!(o.days.first().unwrap().date, date(6, 29));
    assert_eq!(o.days.last().unwrap().date, date(10, 14));
    assert!(o.days.iter().all(|d| d.total_ms == 0));
    assert_eq!(o.balance, "这一周还没有记录。");
}

#[test]
fn overview_sums_the_week_and_the_month_and_counts_days_studied() {
    let (_dir, clock, app) = setup(at(10, 1, 8, 0));
    let math = subject(&app, "数学");
    let english = subject(&app, "英语");
    record(&app, &clock, math, at(10, 1, 9, 0), 60);
    record(&app, &clock, math, at(10, 12, 9, 0), 120);
    record(&app, &clock, english, at(10, 13, 9, 0), 30);
    record(&app, &clock, math, at(10, 14, 9, 0), 90);
    clock.set(at(10, 14, 20, 0));

    let o = app.stats_overview().unwrap();
    assert_eq!((o.week.from, o.week.to), (date(10, 12), date(10, 18)));
    assert_eq!(o.week.total_ms, 240 * MIN);
    assert_eq!(o.week.days_studied, 3);
    assert_eq!(o.month.total_ms, 300 * MIN);
    assert_eq!(o.month.days_studied, 4);
    assert_eq!(o.week.subjects[0].subject_id, math);
    assert_eq!(o.week.subjects[0].ms, 210 * MIN);
    assert_eq!(o.balance, "数学花的时间最多，英语这周少一些。");
    assert_eq!(o.days.last().unwrap().total_ms, 90 * MIN);
}

#[test]
fn a_session_across_midnight_counts_on_both_days() {
    let (_dir, clock, app) = setup(at(10, 13, 23, 0));
    let math = subject(&app, "数学");
    record(&app, &clock, math, at(10, 13, 23, 30), 60);
    clock.set(at(10, 14, 9, 0));

    let o = app.stats_overview().unwrap();
    let last_two: Vec<_> = o.days[o.days.len() - 2..].iter().map(|d| d.total_ms).collect();
    assert_eq!(last_two, vec![30 * MIN, 30 * MIN]);
}

#[test]
fn days_between_returns_each_day_of_the_range() {
    let (_dir, clock, app) = setup(at(10, 1, 8, 0));
    let math = subject(&app, "数学");
    record(&app, &clock, math, at(10, 3, 9, 0), 45);
    clock.set(at(10, 14, 9, 0));

    let days = app.stats_days(date(10, 1), date(10, 31)).unwrap();
    assert_eq!(days.len(), 31);
    assert_eq!(days[2].total_ms, 45 * MIN);
}

#[test]
fn days_between_refuses_a_reversed_or_huge_range() {
    let (_dir, _clock, app) = setup(at(10, 14, 9, 0));
    assert!(app.stats_days(date(10, 31), date(10, 1)).is_err());
    assert!(app.stats_days(date(1, 1), NaiveDate::from_ymd_opt(2028, 1, 1).unwrap()).is_err());
}

#[test]
fn the_fallback_subject_is_listed_by_name_in_the_balance() {
    let (_dir, clock, app) = setup(at(10, 12, 8, 0));
    let other = app.store.read(subjects::fallback).unwrap().id;
    record(&app, &clock, other, at(10, 12, 9, 0), 30);
    clock.set(at(10, 14, 9, 0));
    assert_eq!(app.stats_overview().unwrap().balance, "这一周的时间都给了其他。");
}
