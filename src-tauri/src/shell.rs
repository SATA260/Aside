use std::sync::Mutex;

/// 业务逻辑需要的窗口和系统操作。正式运行时由 Tauri 实现，开发桥和测试各有自己的实现。
pub trait Shell: Send + Sync {
    /// 打开某个界面，比如 `main`、`overlay`、`mini`。
    fn open_view(&self, view: &str);
    /// 关掉某个界面。
    fn close_view(&self, view: &str);
    /// 专注期间不让电脑休眠、不让屏幕熄灭。
    fn keep_awake(&self, on: bool);
}

/// 只把调用记下来，测试用。
#[derive(Default)]
pub struct RecordingShell {
    pub calls: Mutex<Vec<String>>,
    awake: Mutex<bool>,
}

impl RecordingShell {
    pub fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }

    pub fn awake(&self) -> bool {
        *self.awake.lock().unwrap()
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
}
