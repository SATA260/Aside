//! 托盘（Windows）和菜单栏（macOS）：常驻图标、三项菜单，专注时显示剩余时间。

use std::sync::{Arc, Mutex};

use serde_json::Value;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

use super::tray_look::{TrayLook, TrayTimer, look, ring};
use crate::app::Aside;
use crate::settings::TimeDisplay;
use crate::windows::show_main;

const ACCENT: [u8; 3] = [159, 183, 217];
#[cfg(not(target_os = "macos"))]
const MUTED: [u8; 3] = [125, 136, 155];
/// Windows 进度圈每 1/48 圈重画一次，不必每秒都换图标
const RING_STEPS: f64 = 48.0;

pub struct Tray {
    icon: TrayIcon<Wry>,
    toggle: MenuItem<Wry>,
    last: Mutex<Option<(TrayLook, i64)>>,
}

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let toggle = MenuItem::with_id(app, "toggle", "开始专注", true, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
    let exit = MenuItem::with_id(app, "quit", "退出 Aside", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&toggle, &show, &PredefinedMenuItem::separator(app)?, &exit])?;
    let icon = TrayIconBuilder::with_id("aside")
        .icon(idle_icon(app))
        .icon_as_template(cfg!(target_os = "macos"))
        .tooltip("Aside")
        .menu(&menu)
        // macOS 的菜单栏点一下就出菜单；Windows 左键打开主窗口，右键出菜单
        .show_menu_on_left_click(cfg!(target_os = "macos"))
        .on_menu_event(|app, event| match event.id().as_ref() {
            "toggle" => in_background(app, |a| a.timer_toggle().map(drop)),
            "show" => {
                let _ = show_main(app);
            }
            "quit" => quit(app),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if cfg!(target_os = "macos") {
                return;
            }
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                let _ = show_main(tray.app_handle());
            }
        })
        .build(app)?;
    app.manage(Tray { icon, toggle, last: Mutex::new(None) });
    Ok(())
}

/// 菜单和快捷键在主线程上触发，计时操作会开关窗口，放到别的线程做。
pub fn in_background(app: &AppHandle, f: impl FnOnce(&Aside) -> crate::error::AppResult<()> + Send + 'static) {
    let Some(aside) = app.try_state::<Arc<Aside>>().map(|s| s.inner().clone()) else {
        return;
    };
    std::thread::spawn(move || {
        if let Err(e) = f(&aside) {
            eprintln!("托盘或快捷键操作没有完成：{e}");
        }
    });
}

/// 退出前把进行中的这一段记下来。
pub fn quit(app: &AppHandle) {
    if let Some(aside) = app.try_state::<Arc<Aside>>() {
        aside.shutdown();
    }
    app.exit(0);
}

fn idle_icon(app: &AppHandle) -> Image<'static> {
    if cfg!(target_os = "macos") {
        return Image::new_owned(ring(36, None, [0, 0, 0]), 36, 36);
    }
    match app.default_window_icon() {
        Some(i) => Image::new_owned(i.rgba().to_vec(), i.width(), i.height()),
        None => Image::new_owned(ring(32, None, ACCENT), 32, 32),
    }
}

/// 每次 `timer:tick` 时调用。
pub fn update(app: &AppHandle, payload: &Value) {
    let Some(tray) = app.try_state::<Tray>() else { return };
    let Ok(timer) = serde_json::from_value::<TrayTimer>(payload.clone()) else { return };
    let display = app.try_state::<Arc<Aside>>().map_or(TimeDisplay::Remaining, |a| a.settings().time_display);
    let look = look(&timer, display);
    let step = look.progress.map_or(-1, |p| (p * RING_STEPS).floor() as i64);

    let mut last = tray.last.lock().unwrap_or_else(|e| e.into_inner());
    let before = last.as_ref();
    if before.is_some_and(|(l, s)| *l == look && *s == step) {
        return;
    }
    if before.is_none_or(|(l, _)| l.active != look.active) {
        let _ = tray.toggle.set_text(if look.active { "结束这一段" } else { "开始专注" });
    }
    if before.is_none_or(|(l, _)| l.tooltip != look.tooltip) {
        let _ = tray.icon.set_tooltip(Some(&look.tooltip));
    }
    #[cfg(target_os = "macos")]
    if before.is_none_or(|(l, _)| l.title != look.title) {
        let _ = tray.icon.set_title(look.title.as_deref());
    }
    #[cfg(not(target_os = "macos"))]
    if before.is_none_or(|(l, s)| *s != step || l.paused != look.paused) {
        let icon = match look.progress {
            Some(p) => Image::new_owned(ring(32, Some(p), if look.paused { MUTED } else { ACCENT }), 32, 32),
            None => idle_icon(app),
        };
        let _ = tray.icon.set_icon(Some(icon));
    }
    *last = Some((look, step));
}
