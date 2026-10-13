//! Aside 的业务核心：持有数据库、计时器和设置，所有命令最终都调用这里的方法。
//! 它不直接依赖 Tauri，窗口和系统操作通过 `Shell`，事件通过 `EventSink`，
//! 所以测试和开发桥都能直接用它。

use std::path::PathBuf;
use std::sync::Arc;

use crate::clock::Clock;
use crate::error::AppResult;
use crate::events::EventSink;
use crate::shell::Shell;
use crate::timer::{Snapshot, Timer};

pub struct Aside {
    pub data_dir: PathBuf,
    pub clock: Arc<dyn Clock>,
    pub events: Arc<dyn EventSink>,
    pub shell: Arc<dyn Shell>,
}

impl Aside {
    pub fn new(
        data_dir: PathBuf,
        clock: Arc<dyn Clock>,
        events: Arc<dyn EventSink>,
        shell: Arc<dyn Shell>,
    ) -> AppResult<Arc<Self>> {
        std::fs::create_dir_all(&data_dir)?;
        Ok(Arc::new(Self { data_dir, clock, events, shell }))
    }

    pub fn timer_snapshot(&self) -> Snapshot {
        Timer::default().snapshot(self.clock.now_ms())
    }
}
