//! 窗口管理：每个显示器一个全屏计时窗口、浮窗、网课窗口，以及它们之间的切换。

mod layout;

use tauri::window::Color;
use tauri::{AppHandle, Manager, Monitor, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::shell::Shell;
use crate::system::power::Power;
use layout::{OverlaySpot, Rect, overlay_spots};

const OVERLAY_PREFIX: &str = "overlay-";
/// 窗口刚建好、页面还没画出来时的底色，和夜读主题一致，避免闪一下白
const BACKDROP: Color = Color(16, 21, 31, 255);

pub struct TauriShell {
    pub app: AppHandle,
    pub power: Power,
}

impl Shell for TauriShell {
    fn open_view(&self, view: &str) {
        let result = match view {
            "main" => show_main(&self.app),
            "overlay" => open_overlay(&self.app),
            _ => Ok(()),
        };
        if let Err(e) = result {
            eprintln!("打开窗口 {view} 失败：{e}");
        }
    }

    fn close_view(&self, view: &str) {
        match view {
            "main" => {
                if let Some(main) = self.app.get_webview_window("main") {
                    let _ = main.hide();
                }
            }
            "overlay" => {
                for (label, w) in self.app.webview_windows() {
                    if label.starts_with(OVERLAY_PREFIX) {
                        let _ = w.destroy();
                    }
                }
            }
            _ => {}
        }
    }

    fn keep_awake(&self, on: bool) {
        self.power.set(on);
    }
}

pub fn show_main(app: &AppHandle) -> tauri::Result<()> {
    if let Some(main) = app.get_webview_window("main") {
        main.show()?;
        main.unminimize()?;
        main.set_focus()?;
    }
    Ok(())
}

/// 屏幕的位置和大小。macOS 用逻辑点（各屏按自己的缩放换算后正好是系统坐标），
/// Windows 用物理像素（全局坐标），这样混用不同缩放的显示器时窗口不会放错屏。
fn screen_rect(m: &Monitor) -> Rect {
    #[cfg(target_os = "macos")]
    let (p, z) = (m.position().to_logical::<f64>(m.scale_factor()), m.size().to_logical::<f64>(m.scale_factor()));
    #[cfg(not(target_os = "macos"))]
    let (p, z) = (m.position().cast::<f64>(), m.size().cast::<f64>());
    Rect { x: p.x, y: p.y, w: z.width, h: z.height }
}

fn place(w: &WebviewWindow, r: Rect) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    {
        w.set_position(tauri::LogicalPosition::new(r.x, r.y))?;
        w.set_size(tauri::LogicalSize::new(r.w, r.h))
    }
    #[cfg(not(target_os = "macos"))]
    {
        w.set_position(tauri::PhysicalPosition::new(r.x, r.y))?;
        w.set_size(tauri::PhysicalSize::new(r.w, r.h))
    }
}

fn screens(app: &AppHandle) -> tauri::Result<(Vec<Rect>, Option<Rect>)> {
    let all = app.available_monitors()?.iter().map(screen_rect).collect();
    let primary = app.primary_monitor()?.as_ref().map(screen_rect);
    Ok((all, primary))
}

/// 每块屏幕铺一个全屏窗口，计时内容只放在主屏上，其他屏是同样的底色。
fn open_overlay(app: &AppHandle) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window(&format!("{OVERLAY_PREFIX}0")) {
        w.show()?;
        return w.set_focus();
    }
    let (all, primary) = screens(app)?;
    let mut spots = overlay_spots(&all, primary);
    if spots.is_empty() {
        let fallback = Rect { x: 0.0, y: 0.0, w: 1280.0, h: 800.0 };
        spots.push(OverlaySpot { label: format!("{OVERLAY_PREFIX}0"), screen: fallback, blank: false });
    }
    let mut timer_window = None;
    for spot in spots {
        let url = if spot.blank { "index.html?view=overlay&blank=1" } else { "index.html?view=overlay" };
        let w = WebviewWindowBuilder::new(app, &spot.label, WebviewUrl::App(url.into()))
            .title("Aside")
            .decorations(false)
            .resizable(false)
            .skip_taskbar(true)
            .always_on_top(true)
            .background_color(BACKDROP)
            .visible(false)
            .build()?;
        place(&w, spot.screen)?;
        cover_screen(&w)?;
        w.show()?;
        if !spot.blank {
            timer_window = Some(w);
        }
    }
    match timer_window {
        Some(w) => w.set_focus(),
        None => Ok(()),
    }
}

/// macOS 用不新开桌面空间的简单全屏，盖住菜单栏和程序坞；Windows 用无边框全屏。
fn cover_screen(w: &WebviewWindow) -> tauri::Result<()> {
    w.set_simple_fullscreen(true)
}
