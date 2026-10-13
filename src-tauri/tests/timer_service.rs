use std::sync::Arc;

use aside_lib::app::{Aside, StartArgs};
use aside_lib::clock::{Clock, ManualClock};
use aside_lib::error::AppError;
use aside_lib::events::{RecordingSink, TIMER_FINISHED};
use aside_lib::shell::RecordingShell;
use aside_lib::store::{sessions, subjects};
use aside_lib::timer::{Display, EndReason, Mode, Phase, Segment};
use tempfile::TempDir;

const MIN: i64 = 60_000;
const T0: i64 = 1_760_000_000_000;

struct Rig {
    dir: TempDir,
    clock: Arc<ManualClock>,
    events: Arc<RecordingSink>,
    shell: Arc<RecordingShell>,
    app: Arc<Aside>,
}

fn rig() -> Rig {
    let dir = tempfile::tempdir().unwrap();
    let clock = Arc::new(ManualClock::new(T0));
    let events = Arc::new(RecordingSink::default());
    let shell = Arc::new(RecordingShell::default());
    let app = Aside::open(dir.path().to_path_buf(), clock.clone(), events.clone(), shell.clone()).unwrap();
    Rig { dir, clock, events, shell, app }
}

/// 模拟 app 意外退出后重新打开：同一个数据目录，新的 Aside。
fn reopen(r: &Rig) -> Arc<Aside> {
    Aside::open(r.dir.path().to_path_buf(), r.clock.clone(), r.events.clone(), r.shell.clone()).unwrap()
}

fn subject_id(app: &Aside, name: &str) -> i64 {
    app.store
        .read(|c| subjects::list(c, true))
        .unwrap()
        .into_iter()
        .find(|s| s.name == name)
        .unwrap()
        .id
}

fn start(app: &Aside, subject: &str, minutes: i64) {
    app.timer_start(StartArgs {
        subject_id: subject_id(app, subject),
        mode: Mode::Countdown,
        planned_ms: Some(minutes * MIN),
        task: "二重积分 第三章习题".into(),
        display: Display::Fullscreen,
    })
    .unwrap();
}

fn session_count(app: &Aside) -> i64 {
    app.store.read(sessions::count).unwrap()
}

fn has_active_row(app: &Aside) -> bool {
    app.store.read(|c| Ok(sessions::load_active(c)?.is_some())).unwrap()
}

/// 每秒调用一次 tick，直到 `until`。
fn run_until(r: &Rig, until: i64) {
    while r.clock.now_ms() < until {
        r.clock.advance(1_000);
        r.app.tick();
    }
}

#[test]
fn finished_session_is_written_with_its_segments() {
    let r = rig();
    start(&r.app, "数学", 45);
    r.clock.set(T0 + 10 * MIN);
    r.app.timer_pause().unwrap();
    r.clock.set(T0 + 15 * MIN);
    r.app.timer_resume().unwrap();
    r.clock.set(T0 + 40 * MIN);
    let outcome = r.app.timer_finish().unwrap();

    assert!(outcome.recorded);
    let s = r.app.store.read(|c| sessions::get(c, outcome.session_id.unwrap())).unwrap();
    assert_eq!(s.subject_id, subject_id(&r.app, "数学"));
    assert_eq!((s.mode.as_str(), s.source.as_str()), ("countdown", "timer"));
    assert_eq!((s.planned_ms, s.focused_ms), (Some(45 * MIN), 35 * MIN));
    assert_eq!((s.started_at, s.ended_at), (T0, T0 + 40 * MIN));
    assert_eq!(s.task, "二重积分 第三章习题");
    assert_eq!(s.note, "");
    assert_eq!(
        s.segments,
        vec![
            Segment { start: T0, end: T0 + 10 * MIN },
            Segment { start: T0 + 15 * MIN, end: T0 + 40 * MIN },
        ]
    );
}

#[test]
fn under_one_minute_is_not_recorded_and_exactly_one_minute_is() {
    let r = rig();
    start(&r.app, "其他", 25);
    r.clock.set(T0 + MIN - 1);
    let short = r.app.timer_finish().unwrap();
    assert!(!short.recorded);
    assert_eq!(short.session_id, None);
    assert_eq!(session_count(&r.app), 0);

    r.app.timer_close(false).unwrap();
    r.clock.set(T0 + 10 * MIN);
    start(&r.app, "其他", 25);
    r.clock.set(T0 + 11 * MIN);
    assert!(r.app.timer_finish().unwrap().recorded);
    assert_eq!(session_count(&r.app), 1);
}

#[test]
fn countdown_reaching_zero_records_the_planned_length_and_announces_it() {
    let r = rig();
    start(&r.app, "英语", 2);
    run_until(&r, T0 + 2 * MIN + 3_000);

    assert_eq!(r.app.timer_view().snapshot.phase, Phase::Finished);
    let finished = r.events.last(TIMER_FINISHED).expect("应该广播 timer:finished");
    assert_eq!(finished["focused_ms"], 2 * MIN);
    assert_eq!(finished["reason"], "completed");
    let id = finished["session_id"].as_i64().unwrap();
    assert_eq!(r.app.store.read(|c| sessions::get(c, id)).unwrap().focused_ms, 2 * MIN);
    assert_eq!(session_count(&r.app), 1);
}

#[test]
fn heartbeat_row_lives_only_while_a_session_is_active() {
    let r = rig();
    assert!(!has_active_row(&r.app));
    start(&r.app, "数学", 45);
    assert!(has_active_row(&r.app));
    run_until(&r, T0 + 31_000);
    let (_, heartbeat) = r.app.store.read(sessions::load_active).unwrap().unwrap();
    assert!(heartbeat >= T0 + 30_000, "心跳应该每 30 秒更新一次");
    r.app.timer_finish().unwrap();
    assert!(!has_active_row(&r.app));
}

#[test]
fn crash_while_running_is_recovered_at_the_last_heartbeat() {
    let r = rig();
    start(&r.app, "数学", 90);
    run_until(&r, T0 + 20 * MIN + 10_000);
    r.clock.set(T0 + 5 * 60 * MIN);

    let app = reopen(&r);
    let view = app.timer_view();
    assert_eq!(view.snapshot.phase, Phase::Idle);
    let recovered = view.recovered.expect("应该提示上次有一段专注已记下");
    assert_eq!(recovered.reason, EndReason::Recovered);
    assert_eq!(recovered.focused_ms, 20 * MIN);
    assert!(recovered.recorded);
    assert_eq!(session_count(&app), 1);
    assert!(!has_active_row(&app));
}

#[test]
fn crash_while_paused_counts_only_up_to_the_pause() {
    let r = rig();
    start(&r.app, "数学", 90);
    r.clock.set(T0 + 12 * MIN);
    r.app.timer_pause().unwrap();
    run_until(&r, T0 + 40 * MIN);

    let app = reopen(&r);
    assert_eq!(app.timer_view().recovered.unwrap().focused_ms, 12 * MIN);
}

#[test]
fn recovered_session_under_one_minute_is_dropped() {
    let r = rig();
    start(&r.app, "数学", 90);
    run_until(&r, T0 + 40_000);

    let app = reopen(&r);
    let recovered = app.timer_view().recovered.unwrap();
    assert!(!recovered.recorded);
    assert_eq!(session_count(&app), 0);
    assert!(!has_active_row(&app));
}

#[test]
fn recovery_notice_is_shown_once() {
    let r = rig();
    start(&r.app, "数学", 90);
    run_until(&r, T0 + 3 * MIN);
    let app = reopen(&r);
    assert!(app.timer_view().recovered.is_some());
    app.timer_dismiss_recovered();
    assert!(app.timer_view().recovered.is_none());
}

#[test]
fn dismissing_the_recovery_notice_tells_every_window() {
    let r = rig();
    start(&r.app, "数学", 90);
    run_until(&r, T0 + 3 * MIN);
    let app = reopen(&r);
    let before = r.events.names().len();
    app.timer_dismiss_recovered();
    let names = r.events.names();
    assert_eq!(&names[before..], ["timer:tick"], "应该广播一次新的计时状态");
    assert!(r.events.last("timer:tick").unwrap()["recovered"].is_null());
}

#[test]
fn starting_with_an_archived_or_unknown_subject_is_rejected() {
    let r = rig();
    let english = subject_id(&r.app, "英语");
    r.app.store.write(|c| subjects::archive(c, english)).unwrap();
    let args = |subject_id| StartArgs {
        subject_id,
        mode: Mode::Countdown,
        planned_ms: Some(25 * MIN),
        task: String::new(),
        display: Display::Fullscreen,
    };
    assert!(matches!(r.app.timer_start(args(english)), Err(AppError::Invalid(_))));
    assert!(matches!(r.app.timer_start(args(999)), Err(AppError::Invalid(_))));
    assert_eq!(r.app.timer_view().snapshot.phase, Phase::Idle);
    assert!(!has_active_row(&r.app));
}

#[test]
fn starting_fullscreen_hides_main_and_opens_the_overlay() {
    let r = rig();
    start(&r.app, "数学", 45);
    assert_eq!(r.shell.calls(), vec!["open:overlay", "close:main"]);
}

#[test]
fn computer_stays_awake_through_pauses_until_the_session_ends() {
    let r = rig();
    assert!(!r.shell.awake());
    start(&r.app, "数学", 45);
    assert!(r.shell.awake());
    r.clock.set(T0 + 5 * MIN);
    r.app.timer_pause().unwrap();
    assert!(r.shell.awake());
    r.app.timer_finish().unwrap();
    assert!(!r.shell.awake());
}

#[test]
fn computer_may_sleep_again_after_an_automatic_finish() {
    let r = rig();
    start(&r.app, "数学", 1);
    run_until(&r, T0 + MIN + 2_000);
    assert_eq!(r.app.timer_view().snapshot.phase, Phase::Finished);
    assert!(!r.shell.awake());
}

#[test]
fn shrinking_to_the_floating_window_swaps_windows_and_survives_a_crash() {
    let r = rig();
    start(&r.app, "数学", 45);
    r.clock.set(T0 + 5 * MIN);
    let view = r.app.timer_switch_display(Display::Mini).unwrap();
    assert_eq!(view.snapshot.plan.unwrap().display, Display::Mini);
    assert_eq!(view.snapshot.focused_ms, 5 * MIN);
    assert_eq!(r.shell.calls()[2..], ["open:mini", "close:overlay"]);

    let (timer, _) = r.app.store.read(sessions::load_active).unwrap().unwrap();
    assert_eq!(timer.plan().unwrap().display, Display::Mini);
}

#[test]
fn switching_to_the_same_display_changes_nothing() {
    let r = rig();
    start(&r.app, "数学", 45);
    r.app.timer_switch_display(Display::Fullscreen).unwrap();
    assert_eq!(r.shell.calls(), ["open:overlay", "close:main"]);
}

#[test]
fn toggle_flips_between_fullscreen_and_the_floating_window() {
    let r = rig();
    start(&r.app, "数学", 45);
    let mini = r.app.timer_toggle_display().unwrap();
    assert_eq!(mini.snapshot.plan.unwrap().display, Display::Mini);
    let back = r.app.timer_toggle_display().unwrap();
    assert_eq!(back.snapshot.plan.unwrap().display, Display::Fullscreen);
    assert_eq!(r.shell.calls()[2..], ["open:mini", "close:overlay", "open:overlay", "close:mini"]);
}

#[test]
fn switching_display_needs_a_session_in_progress() {
    let r = rig();
    assert!(matches!(r.app.timer_switch_display(Display::Mini), Err(AppError::Invalid(_))));
    assert!(matches!(r.app.timer_toggle_display(), Err(AppError::Invalid(_))));
    assert!(r.shell.calls().is_empty());
}

#[test]
fn switching_mid_session_keeps_the_start_page_choice() {
    let r = rig();
    start(&r.app, "数学", 45);
    r.app.timer_switch_display(Display::Mini).unwrap();
    assert_eq!(r.app.settings().last_display, Display::Fullscreen);
}

#[test]
fn finishing_in_the_floating_window_shows_the_end_page_fullscreen() {
    let r = rig();
    start(&r.app, "数学", 45);
    r.app.timer_switch_display(Display::Mini).unwrap();
    r.clock.set(T0 + 5 * MIN);
    r.app.timer_finish().unwrap();
    assert_eq!(r.shell.calls()[4..], ["open:overlay", "close:mini"]);
}

#[test]
fn floating_window_position_is_remembered() {
    let r = rig();
    assert_eq!(r.app.settings().mini_pos, None);
    r.app.remember_mini_pos(1180.0, 64.5).unwrap();
    let reopened = reopen(&r);
    let pos = reopened.settings().mini_pos.unwrap();
    assert_eq!((pos.x, pos.y), (1180.0, 64.5));
}

fn start_mock_in_mini(app: &Aside, subject: &str) {
    app.timer_start(StartArgs {
        subject_id: subject_id(app, subject),
        mode: Mode::Mock,
        planned_ms: Some(150 * MIN),
        task: "2023 真题".into(),
        display: Display::Mini,
    })
    .unwrap();
}

#[test]
fn quick_start_reuses_the_last_subject_mode_length_and_display() {
    let r = rig();
    start_mock_in_mini(&r.app, "数学");
    r.clock.set(T0 + 5 * MIN);
    r.app.timer_finish().unwrap();
    r.app.timer_close(false).unwrap();

    let view = r.app.timer_quick_start().unwrap();
    let plan = view.snapshot.plan.unwrap();
    assert_eq!(plan.subject_id, subject_id(&r.app, "数学"));
    assert_eq!((plan.mode, plan.planned_ms, plan.display), (Mode::Mock, Some(150 * MIN), Display::Mini));
    assert_eq!(plan.task, "");
    assert_eq!(view.snapshot.phase, Phase::Running);
}

#[test]
fn quick_start_uses_other_before_any_session_or_after_archiving() {
    let r = rig();
    let first = r.app.timer_quick_start().unwrap().snapshot.plan.unwrap();
    assert_eq!(first.subject_id, subject_id(&r.app, "其他"));
    assert_eq!((first.mode, first.planned_ms), (Mode::Countdown, Some(45 * MIN)));
    r.app.timer_finish().unwrap();

    start(&r.app, "英语", 45);
    r.app.timer_finish().unwrap();
    r.app.subject_archive(subject_id(&r.app, "英语")).unwrap();
    let after = r.app.timer_quick_start().unwrap().snapshot.plan.unwrap();
    assert_eq!(after.subject_id, subject_id(&r.app, "其他"));
}

#[test]
fn toggle_ends_a_session_in_progress_and_starts_one_otherwise() {
    let r = rig();
    assert_eq!(r.app.timer_toggle().unwrap().snapshot.phase, Phase::Running);
    r.clock.set(T0 + 5 * MIN);
    let ended = r.app.timer_toggle().unwrap();
    assert_eq!(ended.snapshot.phase, Phase::Finished);
    assert_eq!(ended.last.unwrap().focused_ms, 5 * MIN);
    assert_eq!(session_count(&r.app), 1);
}

#[test]
fn starting_again_from_the_end_page_in_another_display_closes_it() {
    let r = rig();
    start(&r.app, "数学", 45);
    r.clock.set(T0 + 5 * MIN);
    r.app.timer_finish().unwrap();
    start_mock_in_mini(&r.app, "数学");
    assert_eq!(r.shell.calls()[2..], ["open:mini", "close:main", "close:overlay"]);
}

#[test]
fn quitting_records_a_session_in_progress_without_opening_windows() {
    let r = rig();
    start(&r.app, "数学", 45);
    r.clock.set(T0 + 20 * MIN);
    r.app.shutdown();
    assert_eq!(session_count(&r.app), 1);
    assert!(!has_active_row(&r.app));
    assert!(!r.shell.awake());
    assert_eq!(r.shell.calls(), ["open:overlay", "close:main"]);
    assert!(reopen(&r).timer_view().recovered.is_none());
}

#[test]
fn quitting_while_idle_changes_nothing() {
    let r = rig();
    r.app.shutdown();
    assert_eq!(session_count(&r.app), 0);
    assert!(r.shell.calls().is_empty());
}

#[test]
fn closing_the_end_page_returns_to_the_main_window() {
    let r = rig();
    start(&r.app, "数学", 45);
    r.clock.set(T0 + 5 * MIN);
    r.app.timer_finish().unwrap();
    r.app.timer_close(false).unwrap();
    assert_eq!(r.app.timer_view().snapshot.phase, Phase::Idle);
    assert_eq!(r.shell.calls()[2..], ["open:main", "close:overlay"]);
}

#[test]
fn view_after_finishing_keeps_the_outcome_for_the_end_page() {
    let r = rig();
    start(&r.app, "数学", 45);
    r.clock.set(T0 + 5 * MIN);
    let outcome = r.app.timer_finish().unwrap();
    let view = r.app.timer_view();
    assert_eq!(view.last.unwrap().session_id, outcome.session_id);
    assert_eq!(view.subject.unwrap().name, "数学");
}

#[test]
fn note_is_saved_on_the_session_and_limited_to_200_characters() {
    let r = rig();
    start(&r.app, "数学", 45);
    r.clock.set(T0 + 5 * MIN);
    let id = r.app.timer_finish().unwrap().session_id.unwrap();

    r.app.session_set_note(id, "  第 3 题没想通，明天问 ").unwrap();
    assert_eq!(r.app.store.read(|c| sessions::get(c, id)).unwrap().note, "第 3 题没想通，明天问");

    let too_long = "字".repeat(201);
    assert!(matches!(r.app.session_set_note(id, &too_long), Err(AppError::Invalid(_))));
    r.app.session_set_note(id, &"字".repeat(200)).unwrap();
    assert!(matches!(r.app.session_set_note(999, "x"), Err(AppError::Invalid(_))));
}

#[test]
fn starting_remembers_mode_length_and_display_for_next_time() {
    let r = rig();
    r.app
        .timer_start(StartArgs {
            subject_id: subject_id(&r.app, "数学"),
            mode: Mode::Countdown,
            planned_ms: Some(90 * MIN),
            task: String::new(),
            display: Display::Mini,
        })
        .unwrap();
    let s = r.app.settings();
    assert_eq!((s.last_mode, s.last_countdown_ms, s.last_display), (Mode::Countdown, 90 * MIN, Display::Mini));
}
