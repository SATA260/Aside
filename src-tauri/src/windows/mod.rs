//! 窗口管理：每个显示器一个全屏计时窗口、浮窗、网课窗口，以及它们之间的切换。

use tauri::{AppHandle, Manager};

use crate::shell::Shell;

pub struct TauriShell {
    pub app: AppHandle,
}

impl Shell for TauriShell {
    fn open_view(&self, view: &str) {
        if view == "main" {
            if let Some(main) = self.app.get_webview_window("main") {
                let _ = main.show();
                let _ = main.set_focus();
            }
        }
    }

    fn close_view(&self, view: &str) {
        if view == "main" {
            if let Some(main) = self.app.get_webview_window("main") {
                let _ = main.hide();
            }
        }
    }
}
