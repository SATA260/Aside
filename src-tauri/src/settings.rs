//! 用户设置。整份设置作为一个 JSON 存在 settings 表里，缺的字段用默认值补上，
//! 所以以后加新设置不需要迁移数据库。

use serde::{Deserialize, Serialize};

use crate::timer::{Display, MINUTE_MS, Mode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Night,
    Bamboo,
    Mist,
    Dusk,
    /// 跟随天色：按真实时刻自动切换
    Sky,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeDisplay {
    /// 倒计时显示剩余时间，正计时显示已学时长
    Remaining,
    /// 始终显示已学时长
    Elapsed,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub theme: Theme,
    pub sfx_enabled: bool,
    pub sfx_volume: f64,
    pub time_display: TimeDisplay,
    pub last_mode: Mode,
    pub last_countdown_ms: i64,
    pub last_mock_ms: i64,
    pub last_display: Display,
    /// 上一段的科目，快捷键和托盘开始时沿用
    pub last_subject_id: Option<i64>,
    /// 浮窗上次关掉时的位置，没拖过时为空
    pub mini_pos: Option<Point>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::Night,
            sfx_enabled: true,
            sfx_volume: 0.4,
            time_display: TimeDisplay::Remaining,
            last_mode: Mode::Countdown,
            last_countdown_ms: 45 * MINUTE_MS,
            last_mock_ms: 180 * MINUTE_MS,
            last_display: Display::Fullscreen,
            last_subject_id: None,
            mini_pos: None,
        }
    }
}

impl Settings {
    /// 把音量这类数值限制在合法范围内。
    pub fn normalized(mut self) -> Self {
        self.sfx_volume = self.sfx_volume.clamp(0.0, 1.0);
        self
    }
}
