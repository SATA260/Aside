//! 两个全局快捷键：开始或结束、全屏和浮窗互相切换。可以在设置里改，
//! 新的组合注册不上（被别的软件占用）时提示原因并保留旧的。

use tauri_plugin_global_shortcut::{Modifiers, Shortcut};

use crate::app::Aside;
use crate::error::{AppError, AppResult};
use crate::settings::Settings;

pub const ACTIONS: [&str; 2] = ["toggle", "display"];

fn label(action: &str) -> &'static str {
    if action == "toggle" { "开始或结束" } else { "全屏和浮窗切换" }
}

/// 认得出、按住了 ⌘ / Ctrl / Alt 之一、只有一个普通键的组合才能用，否则打字时会被抢走。
pub fn parse(accel: &str) -> AppResult<Shortcut> {
    let shortcut: Shortcut = accel.parse().map_err(|_| AppError::Invalid("认不出这个组合键".into()))?;
    if !shortcut.mods.intersects(Modifiers::CONTROL | Modifiers::ALT | Modifiers::SUPER) {
        return Err(AppError::Invalid("至少要按住一个修饰键，比如 ⌘、Ctrl 或 Alt".into()));
    }
    Ok(shortcut)
}

impl Aside {
    fn shortcut_of(s: &Settings, action: &str) -> String {
        if action == "toggle" { s.shortcut_toggle.clone() } else { s.shortcut_display.clone() }
    }

    /// 启动时注册设置里的快捷键；注册不上的只记日志，用户在设置里换一个即可。
    pub fn shortcuts_bind_all(&self) {
        let s = self.settings();
        for action in ACTIONS {
            if let Err(e) = self.shell.bind_shortcut(action, &Self::shortcut_of(&s, action)) {
                eprintln!("快捷键“{}”没有注册上：{e}", label(action));
            }
        }
    }

    pub fn shortcuts_pause(&self, paused: bool) {
        self.shell.pause_shortcuts(paused);
    }

    /// 换一个快捷键。不管换没换成，录制时暂停的快捷键都会恢复。
    pub fn shortcut_set(&self, action: &str, accel: &str) -> AppResult<Settings> {
        self.shell.pause_shortcuts(false);
        if !ACTIONS.contains(&action) {
            return Err(AppError::Invalid(format!("没有这个快捷键：{action}")));
        }
        let accel = accel.trim();
        let wanted = parse(accel)?;
        let s = self.settings();
        let other = ACTIONS.into_iter().find(|a| *a != action).unwrap_or("display");
        if parse(&Self::shortcut_of(&s, other)).is_ok_and(|o| o == wanted) {
            return Err(AppError::Invalid(format!("和“{}”的快捷键重复了", label(other))));
        }
        self.shell
            .bind_shortcut(action, accel)
            .map_err(|e| AppError::Invalid(format!("这个组合{e}，还在用原来的")))?;
        self.edit_settings(|s| {
            if action == "toggle" {
                s.shortcut_toggle = accel.to_string();
            } else {
                s.shortcut_display = accel.to_string();
            }
        })
    }

    /// 按下快捷键。切换显示只在专注中有效，其他时候什么也不做。
    pub fn shortcut_pressed(&self, action: &str) -> AppResult<()> {
        match action {
            "toggle" => self.timer_toggle().map(drop),
            "display" if self.timer_is_active() => self.timer_toggle_display().map(drop),
            _ => Ok(()),
        }
    }
}
