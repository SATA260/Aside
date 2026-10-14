//! 用户设置。整份设置作为一个 JSON 存在 settings 表里，缺的字段用默认值补上，
//! 所以以后加新设置不需要迁移数据库。

use serde::{Deserialize, Serialize};

use crate::timer::{DAY_MS, Display, MINUTE_MS, Mode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scene {
    Sky,
    Rain,
    Lamp,
    Stars,
    Fog,
}

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
    /// 全局快捷键：开始或结束。只能经由 `shortcut_set` 修改，因为要先注册成功
    pub shortcut_toggle: String,
    /// 全局快捷键：全屏和浮窗互相切换
    pub shortcut_display: String,
    /// 每日目标。0 表示不设，柱状图上也就没有参考线
    pub daily_goal_ms: i64,
    pub scene: Scene,
    /// 背景音，空字符串表示关掉
    pub ambient: String,
    pub ambient_volume: f64,
    /// 考研倒计时，默认关掉。只在开始页显示
    pub exam_on: bool,
    /// 考试日期，形如 2026-12-21。空着表示还没填
    pub exam_date: String,
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
            shortcut_toggle: "CommandOrControl+Alt+S".into(),
            shortcut_display: "CommandOrControl+Alt+M".into(),
            daily_goal_ms: 0,
            scene: Scene::Sky,
            ambient: String::new(),
            ambient_volume: 0.6,
            exam_on: false,
            exam_date: String::new(),
        }
    }
}

impl Settings {
    /// 把音量这类数值限制在合法范围内。
    pub fn normalized(mut self) -> Self {
        self.sfx_volume = self.sfx_volume.clamp(0.0, 1.0);
        self.daily_goal_ms = self.daily_goal_ms.clamp(0, DAY_MS);
        self.ambient_volume = self.ambient_volume.clamp(0.0, 1.0);
        if !self.ambient.is_empty() && crate::audio::builtin::get(&self.ambient).is_none() {
            self.ambient.clear();
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_daily_goal_stays_between_nothing_and_one_day() {
        let low = Settings {
            daily_goal_ms: -1,
            ..Settings::default()
        }
        .normalized();
        let high = Settings {
            daily_goal_ms: DAY_MS + 1,
            ..Settings::default()
        }
        .normalized();
        assert_eq!(low.daily_goal_ms, 0);
        assert_eq!(high.daily_goal_ms, DAY_MS);
        assert_eq!(Settings::default().normalized().daily_goal_ms, 0);
    }

    #[test]
    fn an_unknown_sound_turns_off_and_the_volume_stays_in_range() {
        let loud = Settings { ambient: "nope".into(), ambient_volume: 4.0, ..Settings::default() }.normalized();
        assert_eq!(loud.ambient, "");
        assert_eq!(loud.ambient_volume, 1.0);
        let quiet = Settings { ambient: "rain".into(), ambient_volume: -1.0, ..Settings::default() }.normalized();
        assert_eq!(quiet.ambient, "rain");
        assert_eq!(quiet.ambient_volume, 0.0);
    }
}
