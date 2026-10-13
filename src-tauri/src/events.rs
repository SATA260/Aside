use std::sync::Mutex;

use serde_json::Value;

/// 后端推给所有窗口的事件。
pub trait EventSink: Send + Sync {
    fn emit(&self, event: &str, payload: Value);
}

pub const TIMER_TICK: &str = "timer:tick";
pub const TIMER_FINISHED: &str = "timer:finished";
pub const SETTINGS_CHANGED: &str = "settings:changed";
pub const DATA_CHANGED: &str = "data:changed";

/// 只把事件记下来，测试用。
#[derive(Default)]
pub struct RecordingSink {
    pub events: Mutex<Vec<(String, Value)>>,
}

impl RecordingSink {
    pub fn names(&self) -> Vec<String> {
        self.events.lock().unwrap().iter().map(|(n, _)| n.clone()).collect()
    }

    pub fn last(&self, name: &str) -> Option<Value> {
        self.events
            .lock()
            .unwrap()
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.clone())
    }
}

impl EventSink for RecordingSink {
    fn emit(&self, event: &str, payload: Value) {
        self.events.lock().unwrap().push((event.to_string(), payload));
    }
}
