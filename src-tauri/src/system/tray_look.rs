//! 托盘和菜单栏上显示什么：菜单栏文字、悬停提示、进度圈。纯计算，方便单测。

use serde::Deserialize;

use crate::settings::TimeDisplay;
use crate::timer::{Mode, Phase, Plan};

const HOUR_MS: i64 = 3_600_000;

/// 从每秒广播的 `timer:tick` 里取出托盘需要的字段。
#[derive(Debug, Clone, Deserialize)]
pub struct TrayTimer {
    pub phase: Phase,
    pub focused_ms: i64,
    pub planned_ms: Option<i64>,
    pub remaining_ms: Option<i64>,
    pub plan: Option<Plan>,
    pub subject: Option<SubjectName>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubjectName {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrayLook {
    /// macOS 菜单栏图标旁的文字，没在专注时为空
    pub title: Option<String>,
    pub tooltip: String,
    /// Windows 托盘进度圈，没在专注时为空
    pub progress: Option<f64>,
    pub active: bool,
    pub paused: bool,
}

pub fn look(t: &TrayTimer, display: TimeDisplay) -> TrayLook {
    let paused = t.phase == Phase::Paused;
    if !paused && t.phase != Phase::Running {
        return TrayLook { title: None, tooltip: "Aside".into(), progress: None, active: false, paused: false };
    }
    let (verb, time) = match t.remaining_ms {
        Some(left) if display == TimeDisplay::Remaining => ("还剩", clock(left, true)),
        _ => ("已学", clock(t.focused_ms, false)),
    };
    let mock = t.plan.as_ref().is_some_and(|p| p.mode == Mode::Mock);
    let name = if mock { "模考" } else { t.subject.as_ref().map_or("专注", |s| s.name.as_str()) };
    let progress = match t.planned_ms {
        Some(p) if p > 0 => (t.focused_ms as f64 / p as f64).clamp(0.0, 1.0),
        _ => (t.focused_ms.rem_euclid(HOUR_MS)) as f64 / HOUR_MS as f64,
    };
    TrayLook {
        title: Some(if paused { format!("暂停 {time}") } else { time.clone() }),
        tooltip: format!("{name} · {verb} {time}{}", if paused { " · 暂停中" } else { "" }),
        progress: Some(progress),
        active: true,
        paused,
    }
}

/// 计时数字，和界面上的一致：不到 1 小时“分:秒”，否则“时:分:秒”。
pub fn clock(ms: i64, ceil: bool) -> String {
    let ms = ms.max(0);
    let total = if ceil { (ms + 999) / 1000 } else { ms / 1000 };
    let (h, m, s) = (total / 3600, total % 3600 / 60, total % 60);
    if h > 0 { format!("{h}:{m:02}:{s:02}") } else { format!("{m:02}:{s:02}") }
}

/// 一圈 RGBA 像素：从 12 点方向顺时针亮到 `progress`，其余是淡的底圈；`progress` 为空时整圈都亮，中间加一个点。
pub fn ring(size: u32, progress: Option<f64>, rgb: [u8; 3]) -> Vec<u8> {
    let mut px = vec![0u8; (size * size * 4) as usize];
    let c = size as f64 / 2.0;
    let outer = c - 1.0;
    let inner = outer - (size as f64 / 7.0).max(2.0);
    let dot = size as f64 / 8.0;
    for y in 0..size {
        for x in 0..size {
            let (dx, dy) = (x as f64 + 0.5 - c, y as f64 + 0.5 - c);
            let r = (dx * dx + dy * dy).sqrt();
            let on_ring = (outer - r + 0.5).clamp(0.0, 1.0) * (r - inner + 0.5).clamp(0.0, 1.0);
            let on_dot = if progress.is_none() { (dot - r + 0.5).clamp(0.0, 1.0) } else { 0.0 };
            let turn = (dx.atan2(-dy) / std::f64::consts::TAU).rem_euclid(1.0);
            let strength = match progress {
                Some(p) if turn >= p => 0.28,
                _ => 1.0,
            };
            let a = (on_ring * strength).max(on_dot);
            if a > 0.0 {
                let i = ((y * size + x) * 4) as usize;
                px[i..i + 3].copy_from_slice(&rgb);
                px[i + 3] = (a * 255.0).round() as u8;
            }
        }
    }
    px
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timer::Display;

    const MIN: i64 = 60_000;

    fn timer(phase: Phase, mode: Mode, planned: Option<i64>, focused: i64) -> TrayTimer {
        TrayTimer {
            phase,
            focused_ms: focused,
            planned_ms: planned,
            remaining_ms: planned.map(|p| p - focused),
            plan: Some(Plan { subject_id: 3, mode, planned_ms: planned, task: String::new(), display: Display::Mini }),
            subject: Some(SubjectName { name: "数学".into() }),
        }
    }

    #[test]
    fn countdown_shows_the_remaining_time() {
        let l = look(&timer(Phase::Running, Mode::Countdown, Some(45 * MIN), 12 * MIN + 12_000), TimeDisplay::Remaining);
        assert_eq!(l.title.as_deref(), Some("32:48"));
        assert_eq!(l.tooltip, "数学 · 还剩 32:48");
        assert!((l.progress.unwrap() - 732.0 / 2700.0).abs() < 1e-9);
        assert!(l.active && !l.paused);
    }

    #[test]
    fn remaining_time_rounds_up_like_the_big_clock() {
        let l = look(&timer(Phase::Running, Mode::Countdown, Some(45 * MIN), 45 * MIN - 1_500), TimeDisplay::Remaining);
        assert_eq!(l.title.as_deref(), Some("00:02"));
    }

    #[test]
    fn paused_session_says_so() {
        let l = look(&timer(Phase::Paused, Mode::Countdown, Some(45 * MIN), 12 * MIN + 12_000), TimeDisplay::Remaining);
        assert_eq!(l.title.as_deref(), Some("暂停 32:48"));
        assert_eq!(l.tooltip, "数学 · 还剩 32:48 · 暂停中");
        assert!(l.paused);
    }

    #[test]
    fn elapsed_setting_and_count_up_show_the_time_studied() {
        let l = look(&timer(Phase::Running, Mode::Countdown, Some(45 * MIN), 12 * MIN + 12_000), TimeDisplay::Elapsed);
        assert_eq!(l.title.as_deref(), Some("12:12"));
        assert_eq!(l.tooltip, "数学 · 已学 12:12");

        let up = look(&timer(Phase::Running, Mode::Countup, None, 65 * MIN), TimeDisplay::Remaining);
        assert_eq!(up.title.as_deref(), Some("1:05:00"));
        assert!((up.progress.unwrap() - 5.0 / 60.0).abs() < 1e-9);
    }

    #[test]
    fn mock_exam_is_named_as_such() {
        let l = look(&timer(Phase::Running, Mode::Mock, Some(150 * MIN), 30 * MIN), TimeDisplay::Remaining);
        assert_eq!(l.tooltip, "模考 · 还剩 2:00:00");
    }

    #[test]
    fn nothing_extra_when_not_focusing() {
        for phase in [Phase::Idle, Phase::Finished] {
            let l = look(&timer(phase, Mode::Countdown, Some(45 * MIN), 5 * MIN), TimeDisplay::Remaining);
            assert_eq!(l, TrayLook { title: None, tooltip: "Aside".into(), progress: None, active: false, paused: false });
        }
    }

    #[test]
    fn reads_the_tick_payload() {
        let mut t = crate::timer::Timer::default();
        let plan = Plan { subject_id: 3, mode: Mode::Countdown, planned_ms: Some(45 * MIN), task: "x".into(), display: Display::Fullscreen };
        t.start(plan, 0).unwrap();
        let payload = serde_json::json!({
            "snapshot": t.snapshot(5 * MIN),
            "subject": { "id": 3, "name": "数学", "color": "#8db59a", "sort": 2, "archived": false, "is_fallback": false },
        });
        // TimerView 把 snapshot 摊平在顶层
        let mut flat = payload["snapshot"].as_object().unwrap().clone();
        flat.insert("subject".into(), payload["subject"].clone());
        let parsed: TrayTimer = serde_json::from_value(flat.into()).unwrap();
        assert_eq!(look(&parsed, TimeDisplay::Remaining).title.as_deref(), Some("40:00"));
    }

    fn alpha(px: &[u8], size: u32, x: u32, y: u32) -> u8 {
        px[((y * size + x) * 4 + 3) as usize]
    }

    #[test]
    fn ring_lights_up_clockwise_from_twelve() {
        let s = 32;
        let half = ring(s, Some(0.5), [159, 183, 217]);
        assert_eq!(half.len(), (s * s * 4) as usize);
        let (top, right, bottom_left, left) = ((16, 2), (29, 16), (6, 26), (2, 16));
        assert_eq!(alpha(&half, s, top.0, top.1), 255);
        assert_eq!(alpha(&half, s, right.0, right.1), 255);
        assert!(alpha(&half, s, left.0, left.1) < 100);
        assert!(alpha(&half, s, bottom_left.0, bottom_left.1) < 100);
        assert_eq!(alpha(&half, s, 16, 16), 0, "圈中间是空的");

        let none = ring(s, Some(0.0), [159, 183, 217]);
        assert!(alpha(&none, s, top.0, top.1) < 100 && alpha(&none, s, top.0, top.1) > 0);
        let full = ring(s, Some(1.0), [159, 183, 217]);
        assert_eq!(alpha(&full, s, left.0, left.1), 255);
        assert_eq!(&full[((16 * s + 29) * 4) as usize..][..3], &[159, 183, 217]);
    }

    #[test]
    fn idle_glyph_is_a_full_ring_with_a_dot() {
        let s = 36;
        let g = ring(s, None, [0, 0, 0]);
        assert_eq!(alpha(&g, s, 18, 2), 255);
        assert_eq!(alpha(&g, s, 2, 18), 255);
        assert_eq!(alpha(&g, s, 18, 18), 255, "中间的点");
        assert_eq!(alpha(&g, s, 18, 11), 0, "点和圈之间是空的");
    }
}
