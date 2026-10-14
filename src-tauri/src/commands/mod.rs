//! 前端调用后端的唯一入口：`call(cmd, args)`。这里只做参数转换，业务都在 `Aside` 上。
//! 正式运行时经由 Tauri 的 `call` 命令进来，开发时经由开发桥进来，走的是同一个 `dispatch`。

use std::path::Path;
use std::sync::Arc;

use chrono::NaiveDate;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::app::{Aside, StartArgs};
use crate::error::{AppError, AppResult};
use crate::records::{ManualEntry, SessionPatch};
use crate::timer::Display;

fn args<T: DeserializeOwned>(value: Value) -> AppResult<T> {
    serde_json::from_value(value).map_err(|e| AppError::Invalid(format!("参数不对：{e}")))
}

fn ok<T: Serialize>(value: T) -> AppResult<Value> {
    Ok(serde_json::to_value(value)?)
}

#[derive(Deserialize)]
struct Id {
    id: i64,
}

#[derive(Deserialize)]
struct IncludeArchived {
    #[serde(default)]
    include_archived: bool,
}

#[derive(Deserialize)]
struct SubjectInput {
    name: String,
    color: String,
}

#[derive(Deserialize)]
struct SubjectPatch {
    id: i64,
    name: Option<String>,
    color: Option<String>,
}

#[derive(Deserialize)]
struct Ids {
    ids: Vec<i64>,
}

#[derive(Deserialize)]
struct Again {
    #[serde(default)]
    again: bool,
}

#[derive(Deserialize)]
struct DisplayArg {
    display: Display,
}

#[derive(Deserialize)]
struct Paused {
    paused: bool,
}

#[derive(Deserialize)]
struct On {
    on: bool,
}

#[derive(Deserialize)]
struct ShortcutArg {
    action: String,
    accel: String,
}

#[derive(Deserialize)]
struct Note {
    id: i64,
    note: String,
}

#[derive(Deserialize)]
struct Score {
    id: i64,
    score: i64,
    full_score: i64,
}

#[derive(Deserialize)]
struct Patch {
    id: i64,
    #[serde(flatten)]
    patch: SessionPatch,
}

#[derive(Deserialize)]
struct DateArg {
    date: NaiveDate,
}

#[derive(Deserialize)]
struct Range {
    from: NaiveDate,
    to: NaiveDate,
}

#[derive(Deserialize)]
struct ImportPath {
    path: String,
    name: Option<String>,
}

#[derive(Deserialize)]
struct RenameMedia {
    id: i64,
    name: String,
}

/// `caller` 是发起调用的窗口标签，比如 `main`、`overlay-0`、`mini`。
pub fn dispatch(app: &Aside, cmd: &str, input: Value, _caller: &str) -> AppResult<Value> {
    let input = if input.is_null() {
        Value::Object(Default::default())
    } else {
        input
    };
    match cmd {
        "timer_view" => ok(app.timer_view()),
        "timer_start" => ok(app.timer_start(args::<StartArgs>(input)?)?),
        "timer_pause" => ok(app.timer_pause()?),
        "timer_resume" => ok(app.timer_resume()?),
        "timer_finish" => ok(app.timer_finish()?),
        "timer_close" => ok(app.timer_close(args::<Again>(input)?.again)?),
        "timer_dismiss_recovered" => ok(app.timer_dismiss_recovered()),
        "timer_switch_display" => ok(app.timer_switch_display(args::<DisplayArg>(input)?.display)?),
        "timer_toggle_display" => ok(app.timer_toggle_display()?),

        "settings_get" => ok(app.settings()),
        "settings_update" => ok(app.settings_update(input)?),
        "shortcut_set" => {
            let a: ShortcutArg = args(input)?;
            ok(app.shortcut_set(&a.action, &a.accel)?)
        }
        "shortcuts_pause" => ok(app.shortcuts_pause(args::<Paused>(input)?.paused)),
        "autostart_get" => ok(app.autostart_enabled()),
        "autostart_set" => ok(app.autostart_set(args::<On>(input)?.on)?),

        "subjects_list" => ok(app.subjects_list(args::<IncludeArchived>(input)?.include_archived)?),
        "subject_create" => {
            let a: SubjectInput = args(input)?;
            ok(app.subject_create(&a.name, &a.color)?)
        }
        "subject_update" => {
            let a: SubjectPatch = args(input)?;
            ok(app.subject_update(a.id, a.name.as_deref(), a.color.as_deref())?)
        }
        "subject_archive" => ok(app.subject_archive(args::<Id>(input)?.id)?),
        "subject_restore" => ok(app.subject_restore(args::<Id>(input)?.id)?),
        "subjects_reorder" => ok(app.subjects_reorder(&args::<Ids>(input)?.ids)?),

        "session_set_score" => {
            let a: Score = args(input)?;
            ok(app.session_set_score(a.id, a.score, a.full_score)?)
        }
        "session_set_note" => {
            let a: Note = args(input)?;
            ok(app.session_set_note(a.id, &a.note)?)
        }

        "session_add" => ok(app.session_add(args::<ManualEntry>(input)?)?),
        "session_update" => {
            let a: Patch = args(input)?;
            ok(app.session_update(a.id, a.patch)?)
        }
        "session_delete" => ok(app.session_delete(args::<Id>(input)?.id)?),
        "day_view" => ok(app.day_view(args::<DateArg>(input)?.date)?),

        "alarm_stop" => {
            app.silence_alarm();
            ok(())
        }
        "media_sounds" => ok(app.sounds()?),
        "media_images" => ok(app.pictures()?),
        "media_import_sound" => {
            let a: ImportPath = args(input)?;
            ok(app.import_sound(Path::new(&a.path), a.name.as_deref())?)
        }
        "media_import_image" => {
            let a: ImportPath = args(input)?;
            ok(app.import_picture(Path::new(&a.path), a.name.as_deref())?)
        }
        "media_rename_sound" => {
            let a: RenameMedia = args(input)?;
            ok(app.rename_sound(a.id, &a.name)?)
        }
        "media_delete_sound" => ok(app.delete_sound(args::<Id>(input)?.id)?),
        "media_rename_image" => {
            let a: RenameMedia = args(input)?;
            ok(app.rename_picture(a.id, &a.name)?)
        }
        "media_delete_image" => ok(app.delete_picture(args::<Id>(input)?.id)?),

        "stats_overview" => ok(app.stats_overview()?),
        "stats_days" => {
            let a: Range = args(input)?;
            ok(app.stats_days(a.from, a.to)?)
        }

        _ => Err(AppError::Invalid(format!("未知的命令：{cmd}"))),
    }
}

#[tauri::command]
pub async fn call(
    state: tauri::State<'_, Arc<Aside>>,
    window: tauri::Window,
    cmd: String,
    args: Option<Value>,
) -> Result<Value, String> {
    let app = state.inner().clone();
    let caller = window.label().to_string();
    tauri::async_runtime::spawn_blocking(move || {
        dispatch(&app, &cmd, args.unwrap_or(Value::Null), &caller)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}
