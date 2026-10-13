pub mod app;
pub mod clock;
pub mod commands;
pub mod error;
pub mod events;
mod lecture;
mod media;
pub mod settings;
pub mod shell;
pub mod store;
mod system;
pub mod timer;
mod windows;

use std::sync::Arc;

use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};

use crate::app::Aside;
use crate::clock::SystemClock;
use crate::events::EventSink;
use crate::windows::TauriShell;

struct TauriEvents {
    app: AppHandle,
}

impl EventSink for TauriEvents {
    fn emit(&self, event: &str, payload: Value) {
        let _ = self.app.emit(event, payload);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let data_dir = app.path().app_data_dir()?;
            let aside = Aside::open(
                data_dir,
                Arc::new(SystemClock),
                Arc::new(TauriEvents { app: handle.clone() }),
                Arc::new(TauriShell { app: handle.clone() }),
            )?;
            Aside::spawn_ticker(aside.clone());
            app.manage(aside);
            if let Some(main) = app.get_webview_window("main") {
                main.show()?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![commands::call])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
