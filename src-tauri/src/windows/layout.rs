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

/// 浮窗窗口的逻辑大小：胶囊 248×56，四周留出阴影的位置。
pub const MINI_W: f64 = 280.0;
pub const MINI_H: f64 = 88.0;
const MINI_RIGHT: f64 = 24.0;
const MINI_TOP: f64 = 56.0;

/// 浮窗放在哪：上次的位置还在某块屏幕上就用它，否则放在主屏右上角。
/// `size` 和屏幕坐标用同样的单位。
pub fn mini_spot(saved: Option<(f64, f64)>, size: (f64, f64), screens: &[Rect], primary: Option<Rect>) -> (f64, f64) {
    let (w, h) = size;
    if let Some((x, y)) = saved {
        let (cx, cy) = (x + w / 2.0, y + h / 2.0);
        if screens.iter().any(|s| cx >= s.x && cx < s.x + s.w && cy >= s.y && cy < s.y + s.h) {
            return (x, y);
        }
    }
    let home = primary.filter(|p| screens.contains(p)).or(screens.first().copied());
    match home {
        Some(s) => (s.x + s.w - w - MINI_RIGHT, s.y + MINI_TOP),
        None => (MINI_RIGHT, MINI_TOP),
    }
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

    const SIZE: (f64, f64) = (MINI_W, MINI_H);

    #[test]
    fn floating_window_starts_at_the_top_right_of_the_primary_screen() {
        assert_eq!(mini_spot(None, SIZE, &[LEFT, RIGHT], Some(RIGHT)), (1470.0 + 2560.0 - 280.0 - 24.0, -200.0 + 56.0));
        assert_eq!(mini_spot(None, SIZE, &[LEFT], None), (1470.0 - 280.0 - 24.0, 56.0));
    }

    #[test]
    fn floating_window_returns_where_it_was_left() {
        assert_eq!(mini_spot(Some((300.0, 700.0)), SIZE, &[LEFT, RIGHT], Some(LEFT)), (300.0, 700.0));
        // 一半拖出了屏幕边缘，中心还在屏幕上
        assert_eq!(mini_spot(Some((1300.0, 10.0)), SIZE, &[LEFT], Some(LEFT)), (1300.0, 10.0));
    }

    #[test]
    fn floating_window_comes_back_when_its_screen_is_gone() {
        assert_eq!(mini_spot(Some((3000.0, 100.0)), SIZE, &[LEFT], Some(LEFT)), (1470.0 - 280.0 - 24.0, 56.0));
    }

    #[test]
    fn floating_window_has_a_spot_even_without_screen_info() {
        assert_eq!(mini_spot(None, SIZE, &[], None), (24.0, 56.0));
    }
}
