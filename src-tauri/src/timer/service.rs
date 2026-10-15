//! 把计时状态机接到时钟、数据库、事件和窗口上：开始、暂停、结束、每秒的 tick、
//! 每 30 秒一次的心跳，以及异常退出后的恢复。

use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;

use crate::app::{Aside, StartArgs};
use crate::error::{AppResult, invalid};
use crate::events::{TIMER_FINISHED, TIMER_TICK};
use crate::stats;
use crate::store::subjects::{self, Subject};
use crate::store::sessions;
use crate::timer::{Display, EndReason, Finished, MINUTE_MS, Mode, Phase, Plan, Snapshot, Timer, TimerError};

const HEARTBEAT_MS: i64 = 30_000;

#[derive(Debug, Clone, Serialize)]
pub struct Outcome {
    pub session_id: Option<i64>,
    /// 不到 1 分钟的不记录
    pub recorded: bool,
    pub focused_ms: i64,
    pub reason: EndReason,
    pub plan: Plan,
    pub started_at: i64,
    pub ended_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TimerView {
    #[serde(flatten)]
    pub snapshot: Snapshot,
    pub subject: Option<Subject>,
    /// 刚结束的那一段，结束页用
    pub last: Option<Outcome>,
    /// 上次异常退出后补记的那一段，开始页提示一次
    pub recovered: Option<Outcome>,
    /// 用户点了“再来一段”：开始页沿用上一段的科目
    pub again: bool,
    /// 今天一共专注了多久，包括正在进行的这一段
    pub today_ms: i64,
    pub now: i64,
}

#[derive(Default)]
pub struct TimerState {
    timer: Timer,
    subject: Option<Subject>,
    last: Option<Outcome>,
    recovered: Option<Outcome>,
    again: bool,
    heartbeat_at: i64,
}

pub fn view_name(display: Display) -> &'static str {
    match display {
        Display::Fullscreen | Display::Lecture => "overlay",
        Display::Mini => "mini",
    }
}

impl Aside {
    pub fn timer_view(&self) -> TimerView {
        let st = self.timer_state();
        self.view_of(&st, self.clock.now_ms())
    }

    fn view_of(&self, st: &TimerState, now: i64) -> TimerView {
        let (from, to) = stats::day_bounds(now);
        let recorded = self.store.read(|c| sessions::focused_between(c, from, to)).unwrap_or(0);
        let live: i64 = st.timer.live_segments(now).iter().map(|s| stats::clipped(s, from, to)).sum();
        TimerView {
            snapshot: st.timer.snapshot(now),
            subject: st.subject.clone(),
            last: st.last.clone(),
            recovered: st.recovered.clone(),
            again: st.again,
            today_ms: recorded + live,
            now,
        }
    }

    fn broadcast(&self, view: &TimerView) {
        if let Ok(payload) = serde_json::to_value(view) {
            self.events.emit(TIMER_TICK, payload);
        }
    }

    pub fn timer_start(&self, args: StartArgs) -> AppResult<TimerView> {
        let subject = self.store.read(|c| subjects::get(c, args.subject_id))?;
        if subject.archived {
            return invalid("这个科目已经归档了，先在科目页恢复它");
        }
        let plan = Plan {
            subject_id: subject.id,
            mode: args.mode,
            planned_ms: args.planned_ms,
            task: args.task,
            display: args.display,
        };
        let now = self.clock.now_ms();
        let (view, end_page_open) = {
            let mut st = self.timer_state();
            let end_page_open = st.timer.phase() == Phase::Finished;
            let mut timer = st.timer.clone();
            timer.start(plan.clone(), now)?;
            self.store.write(|c| sessions::save_active(c, &timer, now))?;
            st.timer = timer;
            st.subject = Some(subject);
            st.last = None;
            st.again = false;
            st.heartbeat_at = now;
            (self.view_of(&st, now), end_page_open)
        };
        self.edit_settings(|s| {
            s.last_subject_id = Some(plan.subject_id);
            s.last_mode = plan.mode;
            s.last_display = plan.display;
            match (plan.mode, plan.planned_ms) {
                (Mode::Countdown, Some(ms)) => s.last_countdown_ms = ms,
                (Mode::Mock, Some(ms)) => s.last_mock_ms = ms,
                _ => {}
            }
        })?;
        self.broadcast(&view);
        self.shell.keep_awake(true);
        self.apply_focus(true);
        self.shell.open_view(view_name(plan.display));
        self.shell.close_view("main");
        if end_page_open && view_name(plan.display) != "overlay" {
            self.shell.close_view("overlay");
        }
        Ok(view)
    }

    /// 快捷键和托盘用：不打开开始页，直接沿用上一次的科目、方式、时长和显示方式。
    /// 上一次的科目已经归档或从没开始过时用“其他”。
    pub fn timer_quick_start(&self) -> AppResult<TimerView> {
        let s = self.settings();
        let active = self.store.read(|c| subjects::list(c, false))?;
        let subject = active
            .iter()
            .find(|x| Some(x.id) == s.last_subject_id)
            .or_else(|| active.iter().find(|x| x.is_fallback))
            .ok_or_else(|| crate::error::AppError::Invalid("没有可用的科目".into()))?;
        let planned_ms = match s.last_mode {
            Mode::Countdown => Some(s.last_countdown_ms),
            Mode::Mock => Some(s.last_mock_ms),
            Mode::Countup => None,
        };
        self.timer_start(StartArgs {
            subject_id: subject.id,
            mode: s.last_mode,
            planned_ms,
            task: String::new(),
            display: s.last_display,
        })
    }

    pub fn timer_is_active(&self) -> bool {
        self.timer_state().timer.is_active()
    }

    /// 开始或结束：正在专注就结束这一段，否则按上一次的选择开始。
    pub fn timer_toggle(&self) -> AppResult<TimerView> {
        if self.timer_is_active() {
            self.timer_finish()?;
            Ok(self.timer_view())
        } else {
            self.timer_quick_start()
        }
    }

    /// 退出 app 时调用：进行中的这一段按现在结束并记下，不再打开任何窗口。
    pub fn shutdown(&self) {
        let now = self.clock.now_ms();
        let mut st = self.timer_state();
        if !st.timer.is_active() {
            return;
        }
        match st.timer.finish(now).map_err(Into::into).and_then(|done| self.complete(&mut st, done)) {
            Ok(_) => self.shell.keep_awake(false),
            Err(e) => eprintln!("退出时记下这一段失败：{e}"),
        }
    }

    pub fn timer_pause(&self) -> AppResult<TimerView> {
        self.change(|t, now| t.pause(now))
    }

    pub fn timer_resume(&self) -> AppResult<TimerView> {
        self.change(|t, now| t.resume(now))
    }

    fn change(&self, f: impl FnOnce(&mut Timer, i64) -> Result<(), crate::timer::TimerError>) -> AppResult<TimerView> {
        let now = self.clock.now_ms();
        let view = {
            let mut st = self.timer_state();
            let mut timer = st.timer.clone();
            f(&mut timer, now)?;
            self.store.write(|c| sessions::save_active(c, &timer, now))?;
            st.timer = timer;
            st.heartbeat_at = now;
            self.view_of(&st, now)
        };
        self.broadcast(&view);
        Ok(view)
    }

    pub fn timer_finish(&self) -> AppResult<Outcome> {
        let now = self.clock.now_ms();
        let (outcome, view) = {
            let mut st = self.timer_state();
            let done = st.timer.finish(now)?;
            let outcome = self.complete(&mut st, done)?;
            (outcome, self.view_of(&st, now))
        };
        self.after_finish(&outcome, &view);
        Ok(outcome)
    }

    /// 关掉结束页回到开始页。`again` 表示用户点的是“再来一段”，开始页沿用上一段的科目。
    pub fn timer_close(&self, again: bool) -> AppResult<TimerView> {
        let now = self.clock.now_ms();
        let view = {
            let mut st = self.timer_state();
            if st.timer.is_active() {
                return invalid("专注还在进行中");
            }
            st.timer.reset();
            st.again = again && st.last.is_some();
            self.view_of(&st, now)
        };
        self.broadcast(&view);
        self.silence_alarm();
        self.shell.open_view("main");
        self.shell.close_view("overlay");
        Ok(view)
    }

    /// 专注中换一种显示方式：先打开新窗口，再关掉旧的。只影响这一段，不改开始页上次的选择。
    pub fn timer_switch_display(&self, display: Display) -> AppResult<TimerView> {
        let now = self.clock.now_ms();
        let (from, view) = {
            let mut st = self.timer_state();
            let mut timer = st.timer.clone();
            let from = timer.plan().map(|p| p.display);
            timer.set_display(display)?;
            if from != Some(display) {
                self.store.write(|c| sessions::save_active(c, &timer, now))?;
                st.timer = timer;
            }
            (from, self.view_of(&st, now))
        };
        if let Some(from) = from.filter(|f| *f != display) {
            self.broadcast(&view);
            self.shell.open_view(view_name(display));
            self.shell.close_view(view_name(from));
        }
        Ok(view)
    }

    /// 全屏和浮窗互相切换，快捷键用。
    pub fn timer_toggle_display(&self) -> AppResult<TimerView> {
        let current = {
            let st = self.timer_state();
            st.timer.plan().filter(|_| st.timer.is_active()).map(|p| p.display)
        };
        match current {
            Some(Display::Fullscreen | Display::Lecture) => self.timer_switch_display(Display::Mini),
            Some(Display::Mini) => self.timer_switch_display(Display::Fullscreen),
            None => Err(TimerError::NotActive.into()),
        }
    }

    pub fn timer_dismiss_recovered(&self) -> TimerView {
        let view = {
            let mut st = self.timer_state();
            st.recovered = None;
            self.view_of(&st, self.clock.now_ms())
        };
        self.broadcast(&view);
        view
    }

    /// 每秒调用一次：到点自动结束，按时写心跳，向所有窗口广播当前时间。
    pub fn tick(&self) {
        let now = self.clock.now_ms();
        let mut finished = None;
        let view = {
            let mut st = self.timer_state();
            if let Some(done) = st.timer.tick(now) {
                match self.complete(&mut st, done) {
                    Ok(outcome) => finished = Some(outcome),
                    Err(e) => eprintln!("写入专注记录失败：{e}"),
                }
            } else if st.timer.is_active() && now - st.heartbeat_at >= HEARTBEAT_MS {
                match self.store.write(|c| sessions::save_active(c, &st.timer, now)) {
                    Ok(()) => st.heartbeat_at = now,
                    Err(e) => eprintln!("写入心跳失败：{e}"),
                }
            }
            if !st.timer.is_active() && finished.is_none() {
                return;
            }
            self.view_of(&st, now)
        };
        match finished {
            Some(outcome) => self.after_finish(&outcome, &view),
            None => self.broadcast(&view),
        }
    }

    pub fn spawn_ticker(app: Arc<Aside>) {
        std::thread::spawn(move || {
            loop {
                let wait = 1_000 - app.clock.now_ms().rem_euclid(1_000);
                std::thread::sleep(Duration::from_millis(wait as u64 + 5));
                app.tick();
            }
        });
    }

    /// 写入记录（够 1 分钟的话）并清掉心跳，两步在同一个事务里。
    fn complete(&self, st: &mut TimerState, done: Finished) -> AppResult<Outcome> {
        let recorded = done.focused_ms >= MINUTE_MS;
        let session_id = self.store.write(|c| {
            sessions::clear_active(c)?;
            Ok(if recorded { Some(sessions::insert_from_timer(c, &done)?) } else { None })
        })?;
        let outcome = Outcome {
            session_id,
            recorded,
            focused_ms: done.focused_ms,
            reason: done.reason,
            plan: done.plan,
            started_at: done.started_at,
            ended_at: done.ended_at,
        };
        st.last = Some(outcome.clone());
        Ok(outcome)
    }

    fn after_finish(&self, outcome: &Outcome, view: &TimerView) {
        self.shell.keep_awake(false);
        self.apply_focus(false);
        self.broadcast(view);
        if let Ok(payload) = serde_json::to_value(outcome) {
            self.events.emit(TIMER_FINISHED, payload);
        }
        if outcome.recorded {
            self.data_changed();
        }
        if outcome.reason == crate::timer::EndReason::Completed {
            self.celebrate();
        }
        if outcome.plan.display != Display::Fullscreen {
            self.shell.open_view("overlay");
            self.shell.close_view(view_name(outcome.plan.display));
        }
    }

    /// 启动时调用：上次有没结束的专注，就按最后一次心跳结束并记下。
    pub(crate) fn recover_timer(&self) -> AppResult<()> {
        let Some((mut timer, heartbeat_at)) = self.store.read(sessions::load_active)? else {
            return Ok(());
        };
        let mut st = self.timer_state();
        match timer.recover(heartbeat_at) {
            Ok(done) => {
                let outcome = self.complete(&mut st, done)?;
                st.last = None;
                st.recovered = Some(outcome);
                drop(st);
                self.apply_focus(false);
            }
            Err(_) => self.store.write(sessions::clear_active)?,
        }
        Ok(())
    }
}
