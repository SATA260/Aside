//! 前端调用后端的唯一入口：`call(cmd, args)`。这里只做参数转换，业务都在 `Aside` 上。
//! 正式运行时经由 Tauri 的 `call` 命令进来，开发时经由开发桥进来，走的是同一个 `dispatch`。

use std::sync::Arc;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::app::Aside;
use crate::error::{AppError, AppResult};

fn args<T: DeserializeOwned>(value: Value) -> AppResult<T> {
    Ok(serde_json::from_value(value)?)
}

fn ok<T: Serialize>(value: T) -> AppResult<Value> {
    Ok(serde_json::to_value(value)?)
}

/// `caller` 是发起调用的窗口标签，比如 `main`、`overlay-0`、`mini`。
pub fn dispatch(app: &Aside, cmd: &str, input: Value, _caller: &str) -> AppResult<Value> {
    let input = if input.is_null() { Value::Object(Default::default()) } else { input };
    match cmd {
        "timer_snapshot" => {
            let _: Value = args(input)?;
            ok(app.timer_snapshot())
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
