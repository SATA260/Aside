use std::collections::HashMap;
use std::sync::Mutex;

/// 业务逻辑需要的窗口和系统操作。正式运行时由 Tauri 实现，开发桥和测试各有自己的实现。
pub trait Shell: Send + Sync {
    /// 打开某个界面，比如 `main`、`overlay`、`mini`。
    fn open_view(&self, view: &str);
    /// 关掉某个界面。
    fn close_view(&self, view: &str);
    /// 专注期间不让电脑休眠、不让屏幕熄灭。
    fn keep_awake(&self, on: bool);
    /// 把全局快捷键 `action` 换成 `accel`。新的注册不上时返回原因，旧的保持不变。
    fn bind_shortcut(&self, action: &str, accel: &str) -> Result<(), String>;
    /// 录新快捷键时先让已有的全局快捷键歇一下，不然按下旧组合会直接触发。
    fn pause_shortcuts(&self, paused: bool);
    fn autostart(&self) -> bool;
    fn set_autostart(&self, on: bool) -> Result<(), String>;
    /// 运行用户在“快捷指令”里建好的专注模式指令。找不到时返回能给用户看的原因。
    fn focus_shortcut(&self, name: &str) -> Result<(), String>;
    /// 浮窗换成胶囊或形象时，窗口大小跟着变。
    fn resize_mini(&self, width: f64, height: f64);
}

/// 只把调用记下来，测试用。
#[derive(Default)]
pub struct RecordingShell {
    pub calls: Mutex<Vec<String>>,
    awake: Mutex<bool>,
    bound: Mutex<HashMap<String, String>>,
    taken: Mutex<Vec<String>>,
    shortcuts_paused: Mutex<bool>,
    autostart: Mutex<bool>,
    autostart_refused: Mutex<bool>,
    focus_fail: Mutex<bool>,
}

impl RecordingShell {
    pub fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }

    pub fn awake(&self) -> bool {
        *self.awake.lock().unwrap()
    }

    pub fn bound(&self, action: &str) -> Option<String> {
        self.bound.lock().unwrap().get(action).cloned()
    }

    pub fn shortcuts_paused(&self) -> bool {
        *self.shortcuts_paused.lock().unwrap()
    }

    pub fn autostart(&self) -> bool {
        *self.autostart.lock().unwrap()
    }

    /// 模拟系统不让改登录项。
    pub fn refuse_autostart(&self) {
        *self.autostart_refused.lock().unwrap() = true;
    }

    /// 模拟快捷指令不存在。
    pub fn fail_focus(&self) {
        *self.focus_fail.lock().unwrap() = true;
    }

    /// 模拟别的软件已经占用了这个快捷键。
    pub fn take(&self, accel: &str) {
        self.taken.lock().unwrap().push(accel.to_string());
    }
}

impl Shell for RecordingShell {
    fn open_view(&self, view: &str) {
        self.calls.lock().unwrap().push(format!("open:{view}"));
    }

    fn close_view(&self, view: &str) {
        self.calls.lock().unwrap().push(format!("close:{view}"));
    }

    fn keep_awake(&self, on: bool) {
        *self.awake.lock().unwrap() = on;
    }

    fn bind_shortcut(&self, action: &str, accel: &str) -> Result<(), String> {
        if self.taken.lock().unwrap().iter().any(|t| t == accel) {
            return Err("被别的软件占用了".into());
        }
        self.bound.lock().unwrap().insert(action.to_string(), accel.to_string());
        Ok(())
    }

    fn pause_shortcuts(&self, paused: bool) {
        *self.shortcuts_paused.lock().unwrap() = paused;
    }

    fn autostart(&self) -> bool {
        *self.autostart.lock().unwrap()
    }

    fn set_autostart(&self, on: bool) -> Result<(), String> {
        if *self.autostart_refused.lock().unwrap() {
            return Err("没有权限".into());
        }
        *self.autostart.lock().unwrap() = on;
        Ok(())
    }

    fn focus_shortcut(&self, name: &str) -> Result<(), String> {
        self.calls.lock().unwrap().push(format!("focus:{name}"));
        if *self.focus_fail.lock().unwrap() {
            Err(crate::system::dnd::missing_shortcut(name))
        } else {
            Ok(())
        }
    }

    fn resize_mini(&self, width: f64, height: f64) {
        self.calls.lock().unwrap().push(format!("resize:{width}x{height}"));
    }
}
