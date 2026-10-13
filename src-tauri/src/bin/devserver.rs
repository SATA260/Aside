//! 浏览器开发用的后端：不开 Tauri 窗口，`cargo run --features devserver --bin devserver`，
//! 配合 `pnpm dev`，浏览器打开 http://localhost:1420 就是连着真实后端逻辑的界面。

use std::path::PathBuf;
use std::sync::Arc;

use aside_lib::app::Aside;
use aside_lib::clock::OffsetClock;
use aside_lib::devbridge::{DevShell, Hub, serve};

fn main() {
    let data_dir = std::env::var("ASIDE_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.devdata"));
    let hub = Arc::new(Hub::default());
    let clock = Arc::new(OffsetClock::default());
    let app = Aside::open(data_dir.clone(), clock.clone(), hub.clone(), Arc::new(DevShell { hub: hub.clone() }))
        .expect("无法初始化 Aside");
    Aside::spawn_ticker(app.clone());

    let addr = "127.0.0.1:1430";
    println!("开发桥已启动：http://{addr}，数据目录：{}", data_dir.display());
    serve(app, addr, Some(hub), Some(clock));
}
