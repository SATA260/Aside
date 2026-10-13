//! 阻止休眠。Windows 的 SetThreadExecutionState 只对调用它的线程有效，线程结束就失效，
//! 所以由一个常驻线程持有这把锁，其他线程只发开关消息。

use std::sync::Mutex;
use std::sync::mpsc::{Sender, channel};

pub struct Power {
    tx: Mutex<Sender<bool>>,
}

impl Power {
    pub fn spawn() -> Self {
        let (tx, rx) = channel::<bool>();
        std::thread::spawn(move || {
            let mut guard = None;
            for on in rx {
                if !on {
                    guard = None;
                } else if guard.is_none() {
                    guard = keepawake::Builder::default()
                        .display(true)
                        .idle(true)
                        .reason("专注中")
                        .app_name("Aside")
                        .app_reverse_domain("com.aside.app")
                        .create()
                        .map_err(|e| eprintln!("阻止休眠失败：{e}"))
                        .ok();
                }
            }
        });
        Self { tx: Mutex::new(tx) }
    }

    pub fn set(&self, on: bool) {
        let _ = self.tx.lock().unwrap_or_else(|e| e.into_inner()).send(on);
    }
}
