//! Aside 的业务核心：持有数据库、计时器和设置，所有命令最终都调用这里的方法。
//! 它不直接依赖 Tauri，窗口和系统操作通过 `Shell`，事件通过 `EventSink`，
//! 所以测试和开发桥都能直接用它。计时相关的方法在 `timer::service`，
//! 记录页的在 `records` 和 `stats::service`。

use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, RwLock};

use serde::Deserialize;
use serde_json::{Value, json};

use crate::clock::Clock;
use crate::error::AppResult;
use crate::events::{DATA_CHANGED, EventSink, SETTINGS_CHANGED};
use crate::settings::{Point, Settings};
use crate::shell::Shell;
use crate::store::{self, Store, subjects};
use crate::timer::service::TimerState;
use crate::timer::{Display, Mode};

pub struct Aside {
    pub data_dir: PathBuf,
    pub clock: Arc<dyn Clock>,
    pub events: Arc<dyn EventSink>,
    pub shell: Arc<dyn Shell>,
    pub store: Store,
    pub(crate) timer: Mutex<TimerState>,
    settings: RwLock<Settings>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StartArgs {
    pub subject_id: i64,
    pub mode: Mode,
    pub planned_ms: Option<i64>,
    #[serde(default)]
    pub task: String,
    pub display: Display,
}

const SETTINGS_KEY: &str = "app";

impl Aside {
    pub fn open(
        data_dir: PathBuf,
        clock: Arc<dyn Clock>,
        events: Arc<dyn EventSink>,
        shell: Arc<dyn Shell>,
    ) -> AppResult<Arc<Self>> {
        std::fs::create_dir_all(&data_dir)?;
        let store = Store::open(&data_dir.join("data.db"))?;
        let settings = store
            .read(|c| store::settings::get::<Settings>(c, SETTINGS_KEY))?
            .unwrap_or_default()
            .normalized();
        let app = Self {
            data_dir,
            clock,
            events,
            shell,
            store,
            timer: Mutex::new(TimerState::default()),
            settings: RwLock::new(settings),
        };
        app.recover_timer()?;
        Ok(Arc::new(app))
    }

    pub(crate) fn timer_state(&self) -> MutexGuard<'_, TimerState> {
        self.timer.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub(crate) fn data_changed(&self) {
        self.events.emit(DATA_CHANGED, json!({}));
    }

    // ---------- 设置 ----------

    pub fn settings(&self) -> Settings {
        self.settings.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// 用 `patch` 里的字段覆盖当前设置，类型不对的字段整体拒绝。快捷键要先注册成功，不走这里。
    pub fn settings_update(&self, patch: Value) -> AppResult<Settings> {
        let mut merged = serde_json::to_value(self.settings())?;
        if let (Some(target), Some(patch)) = (merged.as_object_mut(), patch.as_object()) {
            for (k, v) in patch.iter().filter(|(k, _)| !k.starts_with("shortcut_")) {
                target.insert(k.clone(), v.clone());
            }
        }
        let next: Settings = serde_json::from_value::<Settings>(merged)?.normalized();
        self.save_settings(next)
    }

    pub(crate) fn edit_settings(&self, f: impl FnOnce(&mut Settings)) -> AppResult<Settings> {
        let mut next = self.settings();
        f(&mut next);
        self.save_settings(next.normalized())
    }

    /// 浮窗关掉时记下它的位置，下次在同一个地方出现。
    pub fn remember_mini_pos(&self, x: f64, y: f64) -> AppResult<Settings> {
        self.edit_settings(|s| s.mini_pos = Some(Point { x, y }))
    }

    fn save_settings(&self, next: Settings) -> AppResult<Settings> {
        self.store.write(|c| store::settings::put(c, SETTINGS_KEY, &next))?;
        *self.settings.write().unwrap_or_else(|e| e.into_inner()) = next.clone();
        self.events.emit(SETTINGS_CHANGED, serde_json::to_value(&next)?);
        Ok(next)
    }

    // ---------- 科目 ----------

    pub fn subjects_list(&self, include_archived: bool) -> AppResult<Vec<subjects::Subject>> {
        self.store.read(|c| subjects::list(c, include_archived))
    }

    pub fn subject_create(&self, name: &str, color: &str) -> AppResult<subjects::Subject> {
        let s = self.store.write(|c| subjects::create(c, name, color))?;
        self.data_changed();
        Ok(s)
    }

    pub fn subject_update(&self, id: i64, name: Option<&str>, color: Option<&str>) -> AppResult<subjects::Subject> {
        let s = self.store.write(|c| subjects::update(c, id, name, color))?;
        self.data_changed();
        Ok(s)
    }

    pub fn subject_archive(&self, id: i64) -> AppResult<()> {
        self.store.write(|c| subjects::archive(c, id))?;
        self.data_changed();
        Ok(())
    }

    pub fn subject_restore(&self, id: i64) -> AppResult<()> {
        self.store.write(|c| subjects::restore(c, id))?;
        self.data_changed();
        Ok(())
    }

    pub fn subjects_reorder(&self, ids: &[i64]) -> AppResult<()> {
        self.store.write(|c| subjects::reorder(c, ids))?;
        self.data_changed();
        Ok(())
    }
}
