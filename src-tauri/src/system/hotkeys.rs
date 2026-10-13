//! 全局快捷键在系统里的注册。哪个组合对应哪个动作记在这里，按下时由插件的回调查表。

use std::collections::HashMap;
use std::sync::Mutex;

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

#[derive(Default)]
pub struct Hotkeys {
    bound: Mutex<HashMap<String, Shortcut>>,
    paused: Mutex<bool>,
}

impl Hotkeys {
    /// 暂停时从系统里注销，恢复时按原样注册回去。
    pub fn pause(&self, app: &AppHandle, paused: bool) {
        let bound = self.bound.lock().unwrap_or_else(|e| e.into_inner());
        let mut now = self.paused.lock().unwrap_or_else(|e| e.into_inner());
        if *now == paused {
            return;
        }
        let keys = app.global_shortcut();
        for s in bound.values() {
            let _ = if paused { keys.unregister(*s) } else { keys.register(*s) };
        }
        *now = paused;
    }

    /// 先注册新的，成功了再注销旧的；注册不上时旧的保持不变。
    pub fn bind(&self, app: &AppHandle, action: &str, accel: &str) -> Result<(), String> {
        let new: Shortcut = accel.parse().map_err(|_| "认不出".to_string())?;
        let mut bound = self.bound.lock().unwrap_or_else(|e| e.into_inner());
        let old = bound.get(action).copied();
        if old == Some(new) {
            return Ok(());
        }
        let keys = app.global_shortcut();
        keys.register(new).map_err(|_| "被别的软件占用了".to_string())?;
        if let Some(old) = old {
            let _ = keys.unregister(old);
        }
        bound.insert(action.to_string(), new);
        Ok(())
    }

    pub fn action_of(&self, pressed: &Shortcut) -> Option<String> {
        let bound = self.bound.lock().unwrap_or_else(|e| e.into_inner());
        bound.iter().find(|(_, s)| *s == pressed).map(|(a, _)| a.clone())
    }
}
