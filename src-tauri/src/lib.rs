pub mod app;
pub mod audio;
pub mod autostart;
pub mod clock;
pub mod commands;
#[cfg(feature = "devserver")]
pub mod devbridge;
pub mod error;
pub mod events;
pub mod lecture;
pub mod media;
pub mod records;
pub mod settings;
pub mod shell;
pub mod shortcuts;
pub mod stats;
pub mod store;
mod system;
pub mod timer;
mod windows;

use std::sync::Arc;

use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, RunEvent, WindowEvent};
use tauri_plugin_global_shortcut::ShortcutState;

use crate::app::Aside;
use crate::clock::SystemClock;
use crate::events::{EventSink, TIMER_TICK};
use crate::system::hotkeys::Hotkeys;
use crate::system::power::Power;
use crate::system::tray;
use crate::windows::{TauriShell, show_main};

struct TauriEvents {
    app: AppHandle,
}

impl EventSink for TauriEvents {
    fn emit(&self, event: &str, payload: Value) {
        if event == TIMER_TICK {
            tray::update(&self.app, &payload);
        }
        let _ = self.app.emit(event, payload);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        // 第二次打开 Aside 时不再起一个计时器，只把已有的主窗口叫出来
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            let _ = show_main(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::Builder::new().app_name("Aside").args([autostart::ARG]).build())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }
                    if let Some(action) = app.state::<Arc<Hotkeys>>().action_of(shortcut) {
                        tray::in_background(app, move |a| a.shortcut_pressed(&action));
                    }
                })
                .build(),
        )
        .setup(|app| {
            let handle = app.handle().clone();
            let data_dir = app.path().app_data_dir()?;
            let hotkeys = Arc::new(Hotkeys::default());
            app.manage(hotkeys.clone());
            let aside = Aside::open(
                data_dir,
                Arc::new(SystemClock),
                Arc::new(TauriEvents { app: handle.clone() }),
                Arc::new(TauriShell { app: handle.clone(), power: Power::spawn(), hotkeys }),
            )?;
            app.manage(aside.clone());
            tray::setup(&handle)?;
            aside.shortcuts_bind_all();
            Aside::spawn_ticker(aside.clone());
            #[cfg(all(debug_assertions, feature = "devserver"))]
            if std::env::var_os("ASIDE_DEV_BRIDGE").is_some() {
                let bridge = aside.clone();
                std::thread::spawn(move || devbridge::serve(bridge, "127.0.0.1:1431", None, None));
            }
            if autostart::launched_at_login(std::env::args()) {
                // 开机自启时只待在托盘或菜单栏里，程序坞也不出现，打开主窗口时再出现
                #[cfg(target_os = "macos")]
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            } else {
                show_main(&handle)?;
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::Resized(_) = event
                && window.label() == "lecture"
                && let Some(lecture) = window.app_handle().get_window("lecture")
            {
                let _ = windows::place_course(&lecture);
            }
            // 关掉主窗口只是藏起来，Aside 留在托盘或菜单栏里
            if window.label() == "main"
                && let WindowEvent::CloseRequested { api, .. } = event
            {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![commands::call])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|handle, event| match event {
        // 点程序坞图标
        #[cfg(target_os = "macos")]
        RunEvent::Reopen { .. } => {
            let _ = show_main(handle);
        }
        // 窗口全关了也不退出，只有菜单里的“退出”或 ⌘Q 才退出
        RunEvent::ExitRequested { code: None, api, .. } => api.prevent_exit(),
        RunEvent::Exit => {
            if let Some(aside) = handle.try_state::<Arc<Aside>>() {
                aside.shutdown();
            }
        }
        _ => {}
    });
}
