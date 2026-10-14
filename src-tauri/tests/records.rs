use std::sync::Arc;

use aside_lib::app::{Aside, StartArgs};
use aside_lib::clock::ManualClock;
use aside_lib::events::RecordingSink;
use aside_lib::records::SessionPatch;
use aside_lib::shell::RecordingShell;
use aside_lib::timer::{Display, Mode};
use chrono::{Local, NaiveDate, TimeZone};

const MIN: i64 = 60_000;

fn at(d: u32, h: u32, m: u32) -> i64 {
    Local
        .with_ymd_and_hms(2026, 10, d, h, m, 0)
        .unwrap()
        .timestamp_millis()
}

fn date(d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, d).unwrap()
}

struct Env {
    _dir: tempfile::TempDir,
    clock: Arc<ManualClock>,
    events: Arc<RecordingSink>,
    app: Arc<Aside>,
}

fn setup(now: i64) -> Env {
    let dir = tempfile::tempdir().unwrap();
    let clock = Arc::new(ManualClock::new(now));
    let events = Arc::new(RecordingSink::default());
    let app = Aside::open(
        dir.path().to_path_buf(),
        clock.clone(),
        events.clone(),
        Arc::new(RecordingShell::default()),
    )
    .unwrap();
    Env {
        _dir: dir,
        clock,
        events,
        app,
    }
}

fn subject(app: &Aside, name: &str) -> i64 {
    app.subjects_list(true)
        .unwrap()
        .into_iter()
        .find(|s| s.name == name)
        .unwrap()
        .id
}

impl Env {
    fn start(&self, subject_id: i64, task: &str) {
        self.app
            .timer_start(StartArgs {
                subject_id,
                mode: Mode::Countup,
                planned_ms: None,
                task: task.into(),
                display: Display::Fullscreen,
            })
            .unwrap();
    }

    fn finish(&self) -> i64 {
        let id = self.app.timer_finish().unwrap().session_id.unwrap();
        self.app.timer_close(false).unwrap();
        id
    }

    /// 从 `start` 开始不停地学 `minutes` 分钟，返回记录的 id。
    fn record(&self, subject_id: i64, task: &str, start: i64, minutes: i64) -> i64 {
        self.clock.set(start);
        self.start(subject_id, task);
        self.clock.set(start + minutes * MIN);
        self.finish()
    }
}

#[test]
fn a_day_lists_its_sessions_in_time_order_with_subject_and_total() {
    let e = setup(at(13, 7, 0));
    let math = subject(&e.app, "数学");
    let english = subject(&e.app, "英语");
    e.record(english, "背单词", at(13, 7, 40), 50);
    e.record(math, "二重积分", at(13, 8, 45), 150);
    e.record(math, "", at(14, 9, 0), 30);

    let day = e.app.day_view(date(13)).unwrap();
    assert_eq!(day.date, date(13));
    assert_eq!((day.start, day.end), (at(13, 0, 0), at(14, 0, 0)));
    assert_eq!(day.total_ms, 200 * MIN);
    let rows: Vec<_> = day
        .entries
        .iter()
        .map(|r| (r.subject_name.as_str(), r.session.task.as_str(), r.day_ms))
        .collect();
    assert_eq!(
        rows,
        vec![
            ("英语", "背单词", 50 * MIN),
            ("数学", "二重积分", 150 * MIN)
        ]
    );
    assert_eq!(day.entries[0].subject_color, "#7FA3C7");
}

#[test]
fn a_session_across_midnight_shows_on_both_days_with_its_part_of_each() {
    let e = setup(at(13, 23, 0));
    let math = subject(&e.app, "数学");
    let id = e.record(math, "", at(13, 23, 30), 60);

    let before = e.app.day_view(date(13)).unwrap();
    let after = e.app.day_view(date(14)).unwrap();
    assert_eq!(before.entries[0].session.id, id);
    assert_eq!(after.entries[0].session.id, id);
    assert_eq!(
        (before.entries[0].day_ms, after.entries[0].day_ms),
        (30 * MIN, 30 * MIN)
    );
    assert_eq!(after.entries[0].session.focused_ms, 60 * MIN);
    assert_eq!(after.total_ms, 30 * MIN);
}

#[test]
fn a_pause_that_runs_past_midnight_does_not_put_the_session_on_the_next_day() {
    let e = setup(at(13, 23, 0));
    let math = subject(&e.app, "数学");
    e.start(math, "");
    e.clock.set(at(13, 23, 50));
    e.app.timer_pause().unwrap();
    e.clock.set(at(14, 0, 10));
    e.finish();

    assert_eq!(e.app.day_view(date(13)).unwrap().entries.len(), 1);
    assert!(e.app.day_view(date(14)).unwrap().entries.is_empty());
}

#[test]
fn an_entry_can_change_subject_task_and_note() {
    let e = setup(at(13, 9, 0));
    let math = subject(&e.app, "数学");
    let politics = subject(&e.app, "政治");
    let id = e.record(math, "二重积分", at(13, 9, 0), 40);

    let patch = SessionPatch {
        subject_id: Some(politics),
        task: Some("  马原 第一章 ".into()),
        note: Some("第 3 题没想通".into()),
        ..Default::default()
    };
    let s = e.app.session_update(id, patch).unwrap();
    assert_eq!(
        (s.subject_id, s.task.as_str(), s.note.as_str()),
        (politics, "马原 第一章", "第 3 题没想通")
    );
    assert_eq!(s.focused_ms, 40 * MIN);
    assert_eq!(
        e.app.day_view(date(13)).unwrap().entries[0].subject_name,
        "政治"
    );
    assert_eq!(
        e.events.names().last().map(String::as_str),
        Some("data:changed")
    );
}

#[test]
fn an_edit_leaves_fields_it_does_not_mention_alone() {
    let e = setup(at(13, 9, 0));
    let math = subject(&e.app, "数学");
    let id = e.record(math, "二重积分", at(13, 9, 0), 40);
    let s = e
        .app
        .session_update(
            id,
            SessionPatch {
                note: Some("ok".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        (s.subject_id, s.task.as_str(), s.note.as_str()),
        (math, "二重积分", "ok")
    );
}

#[test]
fn an_edit_refuses_a_long_task_a_long_note_or_an_unknown_subject() {
    let e = setup(at(13, 9, 0));
    let math = subject(&e.app, "数学");
    let id = e.record(math, "", at(13, 9, 0), 40);

    let long_task = SessionPatch {
        task: Some("字".repeat(51)),
        ..Default::default()
    };
    assert!(e.app.session_update(id, long_task).is_err());
    let long_note = SessionPatch {
        note: Some("字".repeat(201)),
        ..Default::default()
    };
    assert!(e.app.session_update(id, long_note).is_err());
    let unknown = SessionPatch {
        subject_id: Some(999),
        ..Default::default()
    };
    assert!(e.app.session_update(id, unknown).is_err());
    assert!(e.app.session_update(999, SessionPatch::default()).is_err());
}

#[test]
fn an_entry_cannot_move_to_an_archived_subject_but_may_keep_its_own() {
    let e = setup(at(13, 9, 0));
    let math = subject(&e.app, "数学");
    let politics = subject(&e.app, "政治");
    let id = e.record(math, "", at(13, 9, 0), 40);
    e.app.subject_archive(politics).unwrap();
    e.app.subject_archive(math).unwrap();

    assert!(
        e.app
            .session_update(
                id,
                SessionPatch {
                    subject_id: Some(politics),
                    ..Default::default()
                }
            )
            .is_err()
    );
    let keep = SessionPatch {
        subject_id: Some(math),
        note: Some("留着".into()),
        ..Default::default()
    };
    assert_eq!(e.app.session_update(id, keep).unwrap().note, "留着");
}

#[test]
fn moving_an_entry_makes_it_one_continuous_span_and_follows_the_new_day() {
    let e = setup(at(13, 10, 0));
    let math = subject(&e.app, "数学");
    e.clock.set(at(13, 9, 0));
    e.start(math, "习题");
    e.clock.set(at(13, 9, 20));
    e.app.timer_pause().unwrap();
    e.clock.set(at(13, 9, 40));
    e.app.timer_resume().unwrap();
    e.clock.set(at(13, 10, 0));
    let id = e.finish();
    e.clock.set(at(14, 12, 0));

    let moved = e
        .app
        .session_update(
            id,
            SessionPatch {
                started_at: Some(at(14, 8, 0)),
                ended_at: Some(at(14, 9, 0)),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        moved.segments,
        vec![aside_lib::timer::Segment {
            start: at(14, 8, 0),
            end: at(14, 9, 0)
        }]
    );
    assert_eq!(moved.focused_ms, 60 * MIN);
    assert!(e.app.day_view(date(13)).unwrap().entries.is_empty());
    assert_eq!(e.app.day_view(date(14)).unwrap().total_ms, 60 * MIN);
}

#[test]
fn an_entry_may_touch_its_neighbour_but_not_overlap_it() {
    let e = setup(at(13, 12, 0));
    let math = subject(&e.app, "数学");
    let english = subject(&e.app, "英语");
    e.record(english, "", at(13, 7, 40), 50);
    let id = e.record(math, "", at(13, 9, 0), 60);

    let touch = e
        .app
        .session_update(
            id,
            SessionPatch {
                started_at: Some(at(13, 8, 30)),
                ended_at: Some(at(13, 9, 30)),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(touch.started_at, at(13, 8, 30));

    let err = e
        .app
        .session_update(
            id,
            SessionPatch {
                started_at: Some(at(13, 8, 0)),
                ended_at: Some(at(13, 9, 0)),
                ..Default::default()
            },
        )
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("英语"), "{msg}");
    assert!(msg.contains("07:40"), "{msg}");
    assert_eq!(
        e.app.day_view(date(13)).unwrap().entries[1]
            .session
            .started_at,
        at(13, 8, 30)
    );
}

#[test]
fn a_rejected_time_edit_does_not_keep_the_other_fields_either() {
    let e = setup(at(13, 12, 0));
    let math = subject(&e.app, "数学");
    let id = e.record(math, "原来的任务", at(13, 9, 0), 40);
    let err = e
        .app
        .session_update(
            id,
            SessionPatch {
                task: Some("新任务".into()),
                started_at: Some(at(13, 11, 0)),
                ended_at: Some(at(13, 12, 30)),
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(err.to_string().contains("不能晚于当前时刻"));
    let day = e.app.day_view(date(13)).unwrap();
    assert_eq!(day.entries[0].session.task, "原来的任务");
    assert_eq!(day.entries[0].session.started_at, at(13, 9, 0));
}

#[test]
fn a_span_shorter_than_a_minute_or_longer_than_a_day_is_refused() {
    let e = setup(at(14, 12, 0));
    let math = subject(&e.app, "数学");
    let id = e.record(math, "", at(13, 9, 0), 40);
    let short = SessionPatch {
        started_at: Some(at(13, 8, 0)),
        ended_at: Some(at(13, 8, 0) + MIN - 1),
        ..Default::default()
    };
    assert!(
        e.app
            .session_update(id, short)
            .unwrap_err()
            .to_string()
            .contains("至少 1 分钟")
    );
    let long = SessionPatch {
        started_at: Some(at(13, 8, 0)),
        ended_at: Some(at(13, 8, 0) + 24 * 60 * MIN + MIN),
        ..Default::default()
    };
    assert!(
        e.app
            .session_update(id, long)
            .unwrap_err()
            .to_string()
            .contains("最长 24 小时")
    );
    let exact = SessionPatch {
        started_at: Some(at(13, 8, 0)),
        ended_at: Some(at(13, 8, 0) + MIN),
        ..Default::default()
    };
    assert_eq!(e.app.session_update(id, exact).unwrap().focused_ms, MIN);
}

#[test]
fn an_entry_cannot_move_onto_the_session_in_progress() {
    let e = setup(at(13, 9, 0));
    let math = subject(&e.app, "数学");
    let id = e.record(math, "", at(13, 9, 0), 40);
    e.clock.set(at(13, 14, 0));
    e.start(math, "");
    e.clock.set(at(13, 14, 30));
    let err = e
        .app
        .session_update(
            id,
            SessionPatch {
                started_at: Some(at(13, 13, 50)),
                ended_at: Some(at(13, 14, 10)),
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(err.to_string().contains("正在进行"));
    assert_eq!(
        e.app.day_view(date(13)).unwrap().entries[0]
            .session
            .started_at,
        at(13, 9, 0)
    );
}

#[test]
fn deleting_an_entry_removes_it_from_the_day_and_the_stats() {
    let e = setup(at(13, 9, 0));
    let math = subject(&e.app, "数学");
    let keep = e.record(math, "", at(13, 9, 0), 40);
    let gone = e.record(math, "", at(13, 10, 0), 30);

    e.app.session_delete(gone).unwrap();
    let day = e.app.day_view(date(13)).unwrap();
    assert_eq!(
        day.entries.iter().map(|r| r.session.id).collect::<Vec<_>>(),
        vec![keep]
    );
    assert_eq!(day.total_ms, 40 * MIN);
    assert_eq!(e.app.timer_view().today_ms, 40 * MIN);
    assert!(e.app.session_delete(gone).is_err());
}
