//! 计时状态机：idle → running ⇄ paused → finished。
//! 只保存已经结束的运行段和当前这段的开始时刻，专注时长都由“当前时刻”推算，
//! 所以睡眠、锁屏、窗口被遮挡都不会让时间变慢。这里不碰时钟，时刻都由调用方传进来。

pub mod service;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::error::AppError;

pub const MINUTE_MS: i64 = 60_000;
pub const DAY_MS: i64 = 24 * 60 * MINUTE_MS;
const MAX_TASK_CHARS: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    #[default]
    Idle,
    Running,
    Paused,
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Countdown,
    Countup,
    Mock,
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Countdown => "countdown",
            Mode::Countup => "countup",
            Mode::Mock => "mock",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Display {
    Fullscreen,
    Mini,
    Lecture,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    pub subject_id: i64,
    pub mode: Mode,
    pub planned_ms: Option<i64>,
    #[serde(default)]
    pub task: String,
    pub display: Display,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Segment {
    pub start: i64,
    pub end: i64,
}

impl Segment {
    pub fn len(&self) -> i64 {
        (self.end - self.start).max(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndReason {
    /// 倒计时到点，或正计时满 24 小时
    Completed,
    /// 用户自己结束
    Manual,
    /// 异常退出后按最后一次心跳补上的
    Recovered,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Finished {
    pub plan: Plan,
    pub segments: Vec<Segment>,
    pub focused_ms: i64,
    pub reason: EndReason,
    pub started_at: i64,
    pub ended_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Snapshot {
    pub phase: Phase,
    pub plan: Option<Plan>,
    pub focused_ms: i64,
    pub planned_ms: Option<i64>,
    pub remaining_ms: Option<i64>,
    pub started_at: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum TimerError {
    #[error("已经有一段专注在进行中")]
    AlreadyActive,
    #[error("时长要在 1 分钟到 24 小时之间")]
    InvalidDuration,
    #[error("任务最多 50 个字")]
    TaskTooLong,
    #[error("现在没有在计时")]
    NotRunning,
    #[error("现在没有暂停")]
    NotPaused,
    #[error("现在没有进行中的专注")]
    NotActive,
}

impl From<TimerError> for AppError {
    fn from(e: TimerError) -> Self {
        AppError::Invalid(e.to_string())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Timer {
    phase: Phase,
    plan: Option<Plan>,
    segments: Vec<Segment>,
    run_start: Option<i64>,
}

impl Timer {
    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn plan(&self) -> Option<&Plan> {
        self.plan.as_ref()
    }

    pub fn is_active(&self) -> bool {
        matches!(self.phase, Phase::Running | Phase::Paused)
    }

    pub fn start(&mut self, mut plan: Plan, now: i64) -> Result<(), TimerError> {
        if self.is_active() {
            return Err(TimerError::AlreadyActive);
        }
        plan.task = plan.task.trim().to_string();
        if plan.task.chars().count() > MAX_TASK_CHARS {
            return Err(TimerError::TaskTooLong);
        }
        match plan.mode {
            Mode::Countup => plan.planned_ms = None,
            Mode::Countdown | Mode::Mock => match plan.planned_ms {
                Some(ms) if (MINUTE_MS..=DAY_MS).contains(&ms) => {}
                _ => return Err(TimerError::InvalidDuration),
            },
        }
        *self = Timer {
            phase: Phase::Running,
            plan: Some(plan),
            segments: Vec::new(),
            run_start: Some(now),
        };
        Ok(())
    }

    pub fn pause(&mut self, now: i64) -> Result<(), TimerError> {
        if self.phase != Phase::Running {
            return Err(TimerError::NotRunning);
        }
        self.close_run(now);
        self.phase = Phase::Paused;
        Ok(())
    }

    pub fn resume(&mut self, now: i64) -> Result<(), TimerError> {
        if self.phase != Phase::Paused {
            return Err(TimerError::NotPaused);
        }
        self.run_start = Some(now);
        self.phase = Phase::Running;
        Ok(())
    }

    pub fn finish(&mut self, now: i64) -> Result<Finished, TimerError> {
        self.finish_with(now, EndReason::Manual)
    }

    /// 异常退出后恢复：按最后一次心跳的时刻结束。
    pub fn recover(&mut self, heartbeat_at: i64) -> Result<Finished, TimerError> {
        self.finish_with(heartbeat_at, EndReason::Recovered)
    }

    /// 每秒调用一次；到点时自动结束并返回结果。
    pub fn tick(&mut self, now: i64) -> Option<Finished> {
        if self.phase == Phase::Running && self.closed_ms() + self.open_ms(now) >= self.limit_ms() {
            return self.finish(now).ok();
        }
        None
    }

    pub fn reset(&mut self) {
        *self = Timer::default();
    }

    pub fn focused_ms(&self, now: i64) -> i64 {
        match self.phase {
            Phase::Idle => 0,
            Phase::Finished => self.closed_ms(),
            Phase::Running | Phase::Paused => (self.closed_ms() + self.open_ms(now)).min(self.limit_ms()),
        }
    }

    pub fn snapshot(&self, now: i64) -> Snapshot {
        let focused_ms = self.focused_ms(now);
        let planned_ms = self.plan.as_ref().and_then(|p| p.planned_ms);
        Snapshot {
            phase: self.phase,
            plan: self.plan.clone(),
            focused_ms,
            planned_ms,
            remaining_ms: planned_ms.map(|p| (p - focused_ms).max(0)),
            started_at: self.segments.first().map(|s| s.start).or(self.run_start),
        }
    }

    /// 进行中这一段到 `now` 为止的所有运行段（含正在跑的那段），按天统计用。
    pub fn live_segments(&self, now: i64) -> Vec<Segment> {
        if !self.is_active() {
            return Vec::new();
        }
        let mut segments = self.segments.clone();
        if let Some(start) = self.run_start {
            let room = (self.limit_ms() - self.closed_ms()).max(0);
            segments.push(Segment { start, end: now.max(start).min(start + room) });
        }
        segments
    }

    fn finish_with(&mut self, now: i64, reason: EndReason) -> Result<Finished, TimerError> {
        if !self.is_active() {
            return Err(TimerError::NotActive);
        }
        if self.phase == Phase::Running {
            self.close_run(now);
        }
        let plan = self.plan.clone().ok_or(TimerError::NotActive)?;
        let focused_ms = self.closed_ms();
        let reason = if reason == EndReason::Manual && focused_ms >= self.limit_ms() {
            EndReason::Completed
        } else {
            reason
        };
        self.phase = Phase::Finished;
        let started_at = self.segments.first().map_or(now, |s| s.start);
        let ended_at = self.segments.last().map_or(now, |s| s.end);
        Ok(Finished { plan, segments: self.segments.clone(), focused_ms, reason, started_at, ended_at })
    }

    /// 结束当前运行段，结束时刻不超过“正好到点”的那一刻。
    fn close_run(&mut self, now: i64) {
        if let Some(start) = self.run_start.take() {
            let room = (self.limit_ms() - self.closed_ms()).max(0);
            let end = now.max(start).min(start + room);
            self.segments.push(Segment { start, end });
        }
    }

    fn closed_ms(&self) -> i64 {
        self.segments.iter().map(Segment::len).sum()
    }

    fn open_ms(&self, now: i64) -> i64 {
        self.run_start.map_or(0, |start| (now - start).max(0))
    }

    fn limit_ms(&self) -> i64 {
        match self.plan.as_ref() {
            Some(Plan { mode: Mode::Countdown | Mode::Mock, planned_ms: Some(ms), .. }) => *ms,
            _ => DAY_MS,
        }
    }
}
