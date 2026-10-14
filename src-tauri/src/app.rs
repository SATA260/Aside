//! Aside 的业务核心：持有数据库、计时器和设置，所有命令最终都调用这里的方法。
//! 它不直接依赖 Tauri，窗口和系统操作通过 `Shell`，事件通过 `EventSink`，
//! 所以测试和开发桥都能直接用它。计时相关的方法在 `timer::service`，
//! 记录页的在 `records` 和 `stats::service`。

use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex, MutexGuard, RwLock};

use serde::Deserialize;
use serde_json::{Value, json};

use crate::audio::{self, Player};
use crate::clock::Clock;
use crate::error::AppResult;
use crate::events::{DATA_CHANGED, EventSink, SETTINGS_CHANGED};
use crate::media::{self, LoopMode};
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
    /// 背景音在单独的线程里播，因为声卡对象不能跨线程。
    ambient_tx: Mutex<Option<Sender<AmbientCmd>>>,
    ambient_id: Mutex<String>,
}

enum AmbientCmd {
    Play { samples: Vec<f32>, volume: f32 },
    PlayFile { path: PathBuf, gain_db: f32, volume: f32 },
    Once(Vec<f32>),
    Volume(f32),
    Stop,
}

fn ambient_thread(rx: mpsc::Receiver<AmbientCmd>) {
    let mut player: Option<Player> = None;
    while let Ok(cmd) = rx.recv() {
        match cmd {
            AmbientCmd::Play { samples, volume } => {
                let player = player.get_or_insert_with(Player::open);
                player.set_volume(volume);
                player.switch_to(samples);
            }
            AmbientCmd::PlayFile { path, gain_db, volume } => {
                let player = player.get_or_insert_with(Player::open);
                player.set_volume(volume);
                player.switch_live(path, gain_db);
            }
            AmbientCmd::Once(samples) => {
                let player = player.get_or_insert_with(Player::open);
                player.play_once(samples);
            }
            AmbientCmd::Volume(volume) => {
                if let Some(player) = player.as_ref() {
                    player.set_volume(volume);
                }
            }
            AmbientCmd::Stop => {
                if let Some(player) = player.as_ref() {
                    player.stop();
                }
            }
        }
    }
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
            settings: RwLock::new(settings.clone()),
            ambient_tx: Mutex::new(None),
            ambient_id: Mutex::new(String::new()),
        };
        app.sync_ambient(&settings);
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
        self.sync_ambient(&next);
        Ok(next)
    }

    /// 背景音跟着设置走。换一首才重新加载，调音量不会把正在播的声音打断。
    fn sync_ambient(&self, settings: &Settings) {
        let mut id = self.ambient_id.lock().unwrap_or_else(|e| e.into_inner());
        let changed = settings.ambient != *id;
        if changed {
            *id = settings.ambient.clone();
        }
        drop(id);
        if settings.ambient.is_empty() {
            if changed {
                self.send_ambient(AmbientCmd::Stop);
            }
            return;
        }
        if changed {
            if let Some(sound) = audio::builtin::get(&settings.ambient) {
                self.send_ambient(AmbientCmd::Play {
                    samples: audio::builtin::samples(sound),
                    volume: settings.ambient_volume as f32,
                });
            } else if let Some(id) = media::user_sound_id(&settings.ambient) {
                self.play_imported(id, settings.ambient_volume as f32);
            }
        } else {
            self.send_ambient(AmbientCmd::Volume(settings.ambient_volume as f32));
        }
    }

    pub fn ring(&self) {
        self.send_ambient(AmbientCmd::Once(audio::alarm()));
    }

    pub fn silence_alarm(&self) {
        self.send_ambient(AmbientCmd::Stop);
    }

    fn play_imported(&self, id: i64, volume: f32) {
        let Ok(row) = self.store.read(|c| store::media::sound(c, id)) else {
            return;
        };
        let path = self.data_dir.join(&row.file);
        if media::loop_mode(row.duration_ms) == LoopMode::Seamless {
            if let Ok(samples) = media::decode_loop(&path, row.gain_db as f32) {
                self.send_ambient(AmbientCmd::Play { samples, volume });
            }
        } else {
            self.send_ambient(AmbientCmd::PlayFile { path, gain_db: row.gain_db as f32, volume });
        }
    }

    pub fn sounds(&self) -> AppResult<Vec<store::media::Sound>> {
        self.store.read(|c| store::media::list_sounds(c, &self.data_dir))
    }

    pub fn pictures(&self) -> AppResult<Vec<store::media::Picture>> {
        self.store.read(|c| store::media::list_pictures(c, &self.data_dir))
    }

    pub fn import_sound(&self, path: &std::path::Path, name: Option<&str>) -> AppResult<store::media::Sound> {
        let sound = media::import_sound(&self.store, &self.data_dir, path, name, self.settings().loudness_match)?;
        self.edit_settings(|s| s.ambient = format!("s:{}", sound.id))?;
        self.data_changed();
        Ok(sound)
    }

    pub fn import_picture(&self, path: &std::path::Path, name: Option<&str>) -> AppResult<store::media::Picture> {
        let picture = media::import_picture(&self.store, &self.data_dir, path, name)?;
        self.edit_settings(|s| s.scene_image = Some(picture.id))?;
        self.data_changed();
        Ok(picture)
    }

    pub fn rename_sound(&self, id: i64, name: &str) -> AppResult<()> {
        let name = media::clean_name(name)?;
        self.store.write(|c| store::media::rename_sound(c, id, &name))?;
        self.data_changed();
        Ok(())
    }

    pub fn delete_sound(&self, id: i64) -> AppResult<()> {
        let rel = self.store.write(|c| store::media::delete_sound(c, id))?;
        media::remove_stored_file(&self.data_dir, &rel);
        if self.settings().ambient == format!("s:{id}") {
            self.edit_settings(|s| s.ambient.clear())?;
        }
        self.data_changed();
        Ok(())
    }

    pub fn rename_picture(&self, id: i64, name: &str) -> AppResult<()> {
        let name = media::clean_name(name)?;
        self.store.write(|c| store::media::rename_picture(c, id, &name))?;
        self.data_changed();
        Ok(())
    }

    pub fn delete_picture(&self, id: i64) -> AppResult<()> {
        let rel = self.store.write(|c| store::media::delete_picture(c, id))?;
        media::remove_stored_file(&self.data_dir, &rel);
        if self.settings().scene_image == Some(id) {
            self.edit_settings(|s| s.scene_image = None)?;
        }
        self.data_changed();
        Ok(())
    }

    pub fn picture_file(&self, id: i64) -> AppResult<PathBuf> {
        let row = self.store.read(|c| store::media::picture_file(c, id))?;
        Ok(self.data_dir.join(row.file))
    }

    fn send_ambient(&self, cmd: AmbientCmd) {
        let mut slot = self.ambient_tx.lock().unwrap_or_else(|e| e.into_inner());
        if slot.is_none() {
            if matches!(cmd, AmbientCmd::Stop | AmbientCmd::Volume(_)) {
                return;
            }
            let (tx, rx) = mpsc::channel();
            std::thread::spawn(move || ambient_thread(rx));
            *slot = Some(tx);
        }
        if let Some(tx) = slot.as_ref() {
            let _ = tx.send(cmd);
        }
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
