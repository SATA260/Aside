//! 窗口管理：每个显示器一个全屏计时窗口、浮窗、网课窗口，以及它们之间的切换。

mod layout;

use std::sync::Arc;

use tauri::window::Color;
use tauri::{AppHandle, Manager, Monitor, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_autostart::ManagerExt;

use crate::app::Aside;
use crate::shell::Shell;
use crate::system::hotkeys::Hotkeys;
use crate::system::power::Power;
use layout::{MINI_H, MINI_W, OverlaySpot, Rect, mini_spot, overlay_spots};

const OVERLAY_PREFIX: &str = "overlay-";
/// 窗口刚建好、页面还没画出来时的底色，和夜读主题一致，避免闪一下白
const BACKDROP: Color = Color(16, 21, 31, 255);

pub struct TauriShell {
    pub app: AppHandle,
    pub power: Power,
    pub hotkeys: Arc<Hotkeys>,
}

impl Shell for TauriShell {
    fn open_view(&self, view: &str) {
        let result = match view {
            "main" => show_main(&self.app),
            "overlay" => open_overlay(&self.app),
            "mini" => open_mini(&self.app),
            "lecture" => open_lecture(&self.app),
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
            "mini" => {
                if let Some(w) = self.app.get_webview_window("mini") {
                    remember_position(&self.app, &w);
                    let _ = w.destroy();
                }
            }
            "lecture" => {
                if let Some(w) = self.app.get_window("lecture") {
                    let _ = w.destroy();
                }
            }
            _ => {}
        }
    }

    fn keep_awake(&self, on: bool) {
        self.power.set(on);
    }

    fn bind_shortcut(&self, action: &str, accel: &str) -> Result<(), String> {
        self.hotkeys.bind(&self.app, action, accel)
    }

    fn pause_shortcuts(&self, paused: bool) {
        self.hotkeys.pause(&self.app, paused);
    }

    fn autostart(&self) -> bool {
        self.app.autolaunch().is_enabled().unwrap_or(false)
    }

    fn set_autostart(&self, on: bool) -> Result<(), String> {
        let launcher = self.app.autolaunch();
        let done = if on { launcher.enable() } else { launcher.disable() };
        done.map_err(|e| e.to_string())
    }

    fn focus_shortcut(&self, name: &str) -> Result<(), String> {
        let output = std::process::Command::new("shortcuts")
            .args(["run", name])
            .output()
            .map_err(|_| crate::system::dnd::missing_shortcut(name))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(crate::system::dnd::missing_shortcut(name))
        }
    }

    fn open_course(&self, url: &str) {
        if let Some(window) = self.app.get_window("lecture") {
            let _ = crate::windows::show_course(&window, url);
        }
    }
}

pub fn show_main(app: &AppHandle) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Regular)?;
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
    move_to(w, r.x, r.y)?;
    #[cfg(target_os = "macos")]
    return w.set_size(tauri::LogicalSize::new(r.w, r.h));
    #[cfg(not(target_os = "macos"))]
    return w.set_size(tauri::PhysicalSize::new(r.w, r.h));
}

fn move_to(w: &WebviewWindow, x: f64, y: f64) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    return w.set_position(tauri::LogicalPosition::new(x, y));
    #[cfg(not(target_os = "macos"))]
    return w.set_position(tauri::PhysicalPosition::new(x, y));
}

/// 窗口左上角，单位和 `screen_rect` 一致。
fn position_of(w: &WebviewWindow) -> tauri::Result<(f64, f64)> {
    let p = w.outer_position()?;
    #[cfg(target_os = "macos")]
    let p = p.to_logical::<f64>(w.scale_factor()?);
    #[cfg(not(target_os = "macos"))]
    let p = p.cast::<f64>();
    Ok((p.x, p.y))
}

/// 逻辑像素换成屏幕坐标的单位：macOS 本来就是逻辑点，Windows 按主屏缩放换成物理像素。
fn native_scale(app: &AppHandle) -> f64 {
    if cfg!(target_os = "macos") {
        return 1.0;
    }
    app.primary_monitor().ok().flatten().map_or(1.0, |m| m.scale_factor())
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

/// 置顶的透明小窗，胶囊画在网页里。出现在上次拖到的位置。
fn open_mini(app: &AppHandle) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window("mini") {
        return w.show();
    }
    let (all, primary) = screens(app)?;
    let scale = native_scale(app);
    let saved = app.try_state::<Arc<Aside>>().and_then(|a| a.settings().mini_pos).map(|p| (p.x, p.y));
    let (x, y) = mini_spot(saved, (MINI_W * scale, MINI_H * scale), &all, primary);
    // macOS 只让没有程序坞图标的 app 把窗口放进别的 app 的全屏空间，而且窗口要在切换之后才创建。
    // 主窗口再出现时恢复程序坞图标，见 `show_main`。
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory)?;
    let w = WebviewWindowBuilder::new(app, "mini", WebviewUrl::App("index.html?view=mini".into()))
        .title("Aside")
        .inner_size(MINI_W, MINI_H)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .resizable(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .visible_on_all_workspaces(true)
        .accept_first_mouse(true)
        .focused(false)
        .visible(false)
        .build()?;
    move_to(&w, x, y)?;
    #[cfg(target_os = "macos")]
    float_over_fullscreen_apps(&w)?;
    w.show()
}

const COURSE_BAR: f64 = 52.0;
const COURSE_SIDE: f64 = 280.0;

/// 网课窗口：右边是侧栏网页，左边嵌课程网页。登录状态跟着系统网页视图一起留下。
fn open_lecture(app: &AppHandle) -> tauri::Result<()> {
    if let Some(w) = app.get_window("lecture") {
        w.show()?;
        return w.set_focus();
    }
    let window = tauri::window::WindowBuilder::new(app, "lecture")
        .title("网课")
        .inner_size(1180.0, 760.0)
        .min_inner_size(880.0, 600.0)
        .build()?;
    window.add_child(
        tauri::webview::WebviewBuilder::new("lecture-ui", WebviewUrl::App("index.html?view=lecture".into())).auto_resize(),
        tauri::LogicalPosition::new(0.0, 0.0),
        tauri::LogicalSize::new(1180.0, 760.0),
    )?;
    let saved = app.try_state::<Arc<Aside>>().map(|a| a.settings().lecture_url.clone()).unwrap_or_default();
    if !saved.is_empty() {
        show_course(&window, &saved)?;
    }
    window.show()?;
    window.set_focus()
}

pub(crate) fn show_course(window: &tauri::Window, raw: &str) -> tauri::Result<()> {
    let Ok(parsed) = url::Url::parse(raw) else {
        return Ok(());
    };
    if let Some(view) = window.app_handle().get_webview("course") {
        view.navigate(parsed)?;
    } else {
        window.add_child(
            tauri::webview::WebviewBuilder::new("course", WebviewUrl::External(parsed)),
            tauri::LogicalPosition::new(0.0, COURSE_BAR),
            tauri::LogicalSize::new(800.0, 600.0),
        )?;
    }
    place_course(window)
}

pub(crate) fn place_course(window: &tauri::Window) -> tauri::Result<()> {
    let Some(view) = window.app_handle().get_webview("course") else {
        return Ok(());
    };
    let size = window.inner_size()?;
    let scale = window.scale_factor()?;
    let width = size.width as f64 / scale;
    let height = size.height as f64 / scale;
    view.set_position(tauri::LogicalPosition::new(0.0, COURSE_BAR))?;
    view.set_size(tauri::LogicalSize::new((width - COURSE_SIDE).max(320.0), (height - COURSE_BAR).max(240.0)))?;
    Ok(())
}

/// 用户在全屏的网课或文档里时，浮窗也要浮在上面，而不是留在桌面那个空间里。
#[cfg(target_os = "macos")]
fn float_over_fullscreen_apps(w: &WebviewWindow) -> tauri::Result<()> {
    use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};
    let target = w.clone();
    w.run_on_main_thread(move || {
        let Ok(ptr) = target.ns_window() else { return };
        // SAFETY: 指针来自 Tauri，指向这个窗口的 NSWindow，并且这里在主线程上
        let ns = unsafe { &*(ptr as *const NSWindow) };
        ns.setCollectionBehavior(
            ns.collectionBehavior()
                | NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::FullScreenAuxiliary,
        );
    })
}

fn remember_position(app: &AppHandle, w: &WebviewWindow) {
    let (Ok((x, y)), Some(aside)) = (position_of(w), app.try_state::<Arc<Aside>>()) else {
        return;
    };
    if let Err(e) = aside.remember_mini_pos(x, y) {
        eprintln!("记住浮窗位置失败：{e}");
    }
}

/// macOS 用不新开桌面空间的简单全屏，盖住菜单栏和程序坞；Windows 用无边框全屏。
fn cover_screen(w: &WebviewWindow) -> tauri::Result<()> {
    w.set_simple_fullscreen(true)
}
