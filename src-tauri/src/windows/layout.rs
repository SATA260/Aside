//! 窗口摆在哪里的纯计算，不碰 Tauri，方便单测。坐标都是逻辑像素。

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverlaySpot {
    pub label: String,
    pub screen: Rect,
    /// 不是主屏：只铺同样的底色，不放计时内容
    pub blank: bool,
}

/// 每块屏幕一个全屏窗口。主屏是 `overlay-0`，放计时内容；认不出主屏时用第一块。
pub fn overlay_spots(screens: &[Rect], primary: Option<Rect>) -> Vec<OverlaySpot> {
    let main = primary.and_then(|p| screens.iter().position(|s| *s == p)).unwrap_or(0);
    let others = screens.iter().enumerate().filter(|(i, _)| *i != main).map(|(_, s)| s);
    screens
        .get(main)
        .into_iter()
        .chain(others)
        .enumerate()
        .map(|(i, s)| OverlaySpot { label: format!("overlay-{i}"), screen: *s, blank: i > 0 })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEFT: Rect = Rect { x: 0.0, y: 0.0, w: 1470.0, h: 956.0 };
    const RIGHT: Rect = Rect { x: 1470.0, y: -200.0, w: 2560.0, h: 1440.0 };

    fn summary(spots: &[OverlaySpot]) -> Vec<(&str, f64, bool)> {
        spots.iter().map(|s| (s.label.as_str(), s.screen.x, s.blank)).collect()
    }

    #[test]
    fn single_screen_gets_the_timer() {
        assert_eq!(summary(&overlay_spots(&[LEFT], Some(LEFT))), [("overlay-0", 0.0, false)]);
    }

    #[test]
    fn timer_goes_on_the_primary_screen_and_the_others_stay_blank() {
        let spots = overlay_spots(&[LEFT, RIGHT], Some(RIGHT));
        assert_eq!(summary(&spots), [("overlay-0", 1470.0, false), ("overlay-1", 0.0, true)]);
    }

    #[test]
    fn first_screen_gets_the_timer_when_the_primary_is_unknown() {
        let spots = overlay_spots(&[RIGHT, LEFT], None);
        assert_eq!(summary(&spots), [("overlay-0", 1470.0, false), ("overlay-1", 0.0, true)]);
        let stale = Rect { x: 9.0, ..LEFT };
        assert_eq!(summary(&overlay_spots(&[RIGHT, LEFT], Some(stale)))[0], ("overlay-0", 1470.0, false));
    }
}
