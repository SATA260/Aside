//! 计时状态机：idle → running ⇄ paused → finished。
//! 只保存开始时刻和累计时长，所有窗口显示的时间都由这里推算。

use serde::Serialize;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Idle,
    Running,
    Paused,
    Finished,
}

#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    pub phase: Phase,
    pub focused_ms: u64,
    pub planned_ms: Option<u64>,
}

impl Snapshot {
    pub fn idle() -> Self {
        Self {
            phase: Phase::Idle,
            focused_ms: 0,
            planned_ms: None,
        }
    }
}
