//! 窗口管理：每个显示器一个全屏计时窗口、浮窗、网课窗口，以及它们之间的切换。

use tauri::window::Color;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::shell::Shell;

const OVERLAY_PREFIX: &str = "overlay-";
/// 窗口刚建好、页面还没画出来时的底色，和夜读主题一致，避免闪一下白
const BACKDROP: Color = Color(16, 21, 31, 255);

pub struct TauriShell {
    pub app: AppHandle,
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
}

fn show_main(app: &AppHandle) -> tauri::Result<()> {
    if let Some(main) = app.get_webview_window("main") {
        main.show()?;
        main.unminimize()?;
        main.set_focus()?;
    }
    Ok(())
}

fn open_overlay(app: &AppHandle) -> tauri::Result<()> {
    let label = format!("{OVERLAY_PREFIX}0");
    if let Some(w) = app.get_webview_window(&label) {
        w.show()?;
        return w.set_focus();
    }
    let monitor = app.primary_monitor()?.or(app.available_monitors()?.into_iter().next());
    let mut builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html?view=overlay".into()))
        .title("Aside")
        .decorations(false)
        .resizable(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .background_color(BACKDROP)
        .visible(false);
    if let Some(m) = &monitor {
        let scale = m.scale_factor();
        let pos = m.position().to_logical::<f64>(scale);
        let size = m.size().to_logical::<f64>(scale);
        builder = builder.position(pos.x, pos.y).inner_size(size.width, size.height);
    }
    let w = builder.build()?;
    cover_screen(&w)?;
    w.show()?;
    w.set_focus()
}

/// macOS 用不新开桌面空间的简单全屏，盖住菜单栏和程序坞；Windows 用无边框全屏。
fn cover_screen(w: &WebviewWindow) -> tauri::Result<()> {
    w.set_simple_fullscreen(true)
}
