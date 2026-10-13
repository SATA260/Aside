use aside_lib::timer::{Display, EndReason, Mode, Phase, Plan, Segment, Timer, TimerError};

const MIN: i64 = 60_000;
const HOUR: i64 = 60 * MIN;
const T0: i64 = 1_760_000_000_000;

fn countdown(minutes: i64) -> Plan {
    Plan {
        subject_id: 1,
        mode: Mode::Countdown,
        planned_ms: Some(minutes * MIN),
        task: String::new(),
        display: Display::Fullscreen,
    }
}

fn countup() -> Plan {
    Plan { mode: Mode::Countup, planned_ms: None, ..countdown(1) }
}

fn started(plan: Plan) -> Timer {
    let mut t = Timer::default();
    t.start(plan, T0).unwrap();
    t
}

#[test]
fn new_timer_is_idle_with_nothing_focused() {
    let t = Timer::default();
    assert_eq!(t.phase(), Phase::Idle);
    assert_eq!(t.focused_ms(T0), 0);
}

#[test]
fn running_time_accumulates_from_the_start() {
    let t = started(countdown(45));
    assert_eq!(t.phase(), Phase::Running);
    assert_eq!(t.focused_ms(T0 + 12 * MIN), 12 * MIN);
}

#[test]
fn paused_time_is_not_counted() {
    let mut t = started(countdown(45));
    t.pause(T0 + 10 * MIN).unwrap();
    assert_eq!(t.phase(), Phase::Paused);
    assert_eq!(t.focused_ms(T0 + 25 * MIN), 10 * MIN);
    t.resume(T0 + 25 * MIN).unwrap();
    assert_eq!(t.focused_ms(T0 + 30 * MIN), 15 * MIN);
}

#[test]
fn time_while_asleep_counts_when_not_paused() {
    let t = started(countup());
    assert_eq!(t.focused_ms(T0 + 3 * HOUR), 3 * HOUR);
}

#[test]
fn finishing_returns_every_running_segment() {
    let mut t = started(countdown(90));
    t.pause(T0 + 10 * MIN).unwrap();
    t.resume(T0 + 15 * MIN).unwrap();
    let done = t.finish(T0 + 40 * MIN).unwrap();
    assert_eq!(
        done.segments,
        vec![
            Segment { start: T0, end: T0 + 10 * MIN },
            Segment { start: T0 + 15 * MIN, end: T0 + 40 * MIN },
        ]
    );
    assert_eq!(done.focused_ms, 35 * MIN);
    assert_eq!(done.reason, EndReason::Manual);
    assert_eq!((done.started_at, done.ended_at), (T0, T0 + 40 * MIN));
    assert_eq!(t.phase(), Phase::Finished);
}

#[test]
fn finishing_while_paused_ends_at_the_pause() {
    let mut t = started(countdown(90));
    t.pause(T0 + 20 * MIN).unwrap();
    let done = t.finish(T0 + 50 * MIN).unwrap();
    assert_eq!(done.focused_ms, 20 * MIN);
    assert_eq!(done.ended_at, T0 + 20 * MIN);
}

#[test]
fn countdown_finishes_by_itself_exactly_at_the_planned_length() {
    let mut t = started(countdown(45));
    assert!(t.tick(T0 + 44 * MIN).is_none());
    let done = t.tick(T0 + 45 * MIN + 700).unwrap();
    assert_eq!(done.focused_ms, 45 * MIN);
    assert_eq!(done.ended_at, T0 + 45 * MIN);
    assert_eq!(done.reason, EndReason::Completed);
    assert_eq!(t.phase(), Phase::Finished);
}

#[test]
fn countdown_after_a_long_sleep_still_records_only_the_planned_length() {
    let mut t = started(countdown(45));
    t.pause(T0 + 20 * MIN).unwrap();
    t.resume(T0 + 30 * MIN).unwrap();
    let done = t.tick(T0 + 5 * HOUR).unwrap();
    assert_eq!(done.focused_ms, 45 * MIN);
    assert_eq!(done.ended_at, T0 + 55 * MIN);
}

#[test]
fn ending_by_hand_after_the_countdown_ran_out_is_capped_too() {
    let mut t = started(countdown(30));
    let done = t.finish(T0 + 2 * HOUR).unwrap();
    assert_eq!(done.focused_ms, 30 * MIN);
    assert_eq!(done.reason, EndReason::Completed);
}

#[test]
fn countup_stops_by_itself_at_24_hours() {
    let mut t = started(countup());
    assert!(t.tick(T0 + 23 * HOUR).is_none());
    let done = t.tick(T0 + 30 * HOUR).unwrap();
    assert_eq!(done.focused_ms, 24 * HOUR);
    assert_eq!(done.reason, EndReason::Completed);
}

#[test]
fn snapshot_shows_remaining_time_for_countdown() {
    let mut t = started(countdown(45));
    t.pause(T0 + 12 * MIN).unwrap();
    let s = t.snapshot(T0 + 20 * MIN);
    assert_eq!(s.phase, Phase::Paused);
    assert_eq!(s.focused_ms, 12 * MIN);
    assert_eq!(s.planned_ms, Some(45 * MIN));
    assert_eq!(s.remaining_ms, Some(33 * MIN));
    assert_eq!(s.started_at, Some(T0));
}

#[test]
fn countup_snapshot_has_no_remaining_time() {
    let s = started(countup()).snapshot(T0 + MIN);
    assert_eq!((s.planned_ms, s.remaining_ms), (None, None));
}

#[test]
fn only_one_session_at_a_time() {
    let mut t = started(countdown(45));
    assert_eq!(t.start(countdown(25), T0 + MIN), Err(TimerError::AlreadyActive));
    t.pause(T0 + 2 * MIN).unwrap();
    assert_eq!(t.start(countdown(25), T0 + 3 * MIN), Err(TimerError::AlreadyActive));
}

#[test]
fn a_new_session_can_start_after_finishing() {
    let mut t = started(countdown(45));
    t.finish(T0 + 10 * MIN).unwrap();
    t.start(countdown(25), T0 + 11 * MIN).unwrap();
    assert_eq!(t.phase(), Phase::Running);
    assert_eq!(t.focused_ms(T0 + 12 * MIN), MIN);
}

#[test]
fn reset_returns_to_idle() {
    let mut t = started(countdown(45));
    t.finish(T0 + MIN).unwrap();
    t.reset();
    assert_eq!(t.phase(), Phase::Idle);
    assert_eq!(t.snapshot(T0 + 2 * MIN).plan, None);
}

#[test]
fn wrong_order_of_actions_is_rejected() {
    let mut t = Timer::default();
    assert_eq!(t.pause(T0), Err(TimerError::NotRunning));
    assert_eq!(t.resume(T0), Err(TimerError::NotPaused));
    assert!(matches!(t.finish(T0), Err(TimerError::NotActive)));
    t.start(countdown(45), T0).unwrap();
    assert_eq!(t.resume(T0 + MIN), Err(TimerError::NotPaused));
    t.pause(T0 + MIN).unwrap();
    assert_eq!(t.pause(T0 + 2 * MIN), Err(TimerError::NotRunning));
}

#[test]
fn countdown_length_must_be_between_one_minute_and_24_hours() {
    let mut t = Timer::default();
    let with = |ms: Option<i64>| Plan { planned_ms: ms, ..countdown(1) };
    assert_eq!(t.start(with(Some(MIN - 1)), T0), Err(TimerError::InvalidDuration));
    assert_eq!(t.start(with(Some(0)), T0), Err(TimerError::InvalidDuration));
    assert_eq!(t.start(with(Some(-MIN)), T0), Err(TimerError::InvalidDuration));
    assert_eq!(t.start(with(Some(24 * HOUR + 1)), T0), Err(TimerError::InvalidDuration));
    assert_eq!(t.start(with(None), T0), Err(TimerError::InvalidDuration));
    t.start(with(Some(24 * HOUR)), T0).unwrap();
}

#[test]
fn mock_exam_needs_a_length_too() {
    let mut t = Timer::default();
    let plan = Plan { mode: Mode::Mock, planned_ms: None, ..countdown(1) };
    assert_eq!(t.start(plan, T0), Err(TimerError::InvalidDuration));
}

#[test]
fn countup_ignores_a_planned_length() {
    let mut t = Timer::default();
    t.start(Plan { planned_ms: Some(10 * MIN), ..countup() }, T0).unwrap();
    assert_eq!(t.snapshot(T0).planned_ms, None);
}

#[test]
fn task_is_trimmed_and_limited_to_50_characters() {
    let mut t = Timer::default();
    let long = "题".repeat(51);
    assert_eq!(t.start(Plan { task: long, ..countdown(45) }, T0), Err(TimerError::TaskTooLong));
    t.start(Plan { task: format!("  {}  ", "题".repeat(50)), ..countdown(45) }, T0).unwrap();
    assert_eq!(t.snapshot(T0).plan.unwrap().task, "题".repeat(50));
}

#[test]
fn clock_going_backwards_never_produces_negative_time() {
    let mut t = started(countup());
    assert_eq!(t.focused_ms(T0 - 5 * MIN), 0);
    t.pause(T0 - 5 * MIN).unwrap();
    let done = t.finish(T0).unwrap();
    assert_eq!(done.focused_ms, 0);
    assert!(done.segments.iter().all(|s| s.end >= s.start));
}

#[test]
fn display_can_change_mid_session_without_touching_the_time() {
    let mut t = started(countdown(45));
    t.pause(T0 + 5 * MIN).unwrap();
    t.set_display(Display::Mini).unwrap();
    assert_eq!(t.plan().unwrap().display, Display::Mini);
    assert_eq!((t.phase(), t.focused_ms(T0 + 9 * MIN)), (Phase::Paused, 5 * MIN));

    let mut idle = Timer::default();
    assert_eq!(idle.set_display(Display::Mini), Err(TimerError::NotActive));
    t.finish(T0 + 9 * MIN).unwrap();
    assert_eq!(t.set_display(Display::Fullscreen), Err(TimerError::NotActive));
}

#[test]
fn timer_survives_a_round_trip_through_json() {
    let mut t = started(countdown(45));
    t.pause(T0 + 5 * MIN).unwrap();
    let json = serde_json::to_string(&t).unwrap();
    let back: Timer = serde_json::from_str(&json).unwrap();
    assert_eq!(back.snapshot(T0 + 9 * MIN), t.snapshot(T0 + 9 * MIN));
}
