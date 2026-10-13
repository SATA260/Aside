//! 开发桥：不开 Tauri 窗口，把后端命令通过 HTTP 暴露给浏览器里的界面。
//! 只在开发时用：`cargo run --features devserver --bin devserver`，配合 `pnpm dev`，
//! 浏览器打开 http://localhost:1420 就是连着真实后端逻辑的界面。
//!
//! - `POST /call`    `{cmd, args, window}` → `{ok, value}` 或 `{ok: false, error}`
//! - `GET  /events?since=N` 长轮询，返回序号大于 N 的事件
//! - `POST /dev/advance` `{ms}` 让时钟快进，方便测试倒计时到点

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use aside_lib::app::Aside;
use aside_lib::clock::OffsetClock;
use aside_lib::commands::dispatch;
use aside_lib::events::EventSink;
use aside_lib::shell::Shell;
use serde_json::{Value, json};
use tiny_http::{Header, Method, Response, Server};

const KEEP: usize = 500;

#[derive(Default)]
struct Hub {
    state: Mutex<(i64, VecDeque<(i64, Value)>)>,
    changed: Condvar,
}

impl Hub {
    fn poll(&self, since: i64) -> Value {
        let mut guard = self.state.lock().unwrap();
        if since >= 0 && guard.0 <= since + 1 {
            guard = self.changed.wait_timeout(guard, Duration::from_secs(20)).unwrap().0;
        }
        let (next, queue) = &*guard;
        let events: Vec<&Value> = if since < 0 {
            Vec::new()
        } else {
            queue.iter().filter(|(seq, _)| *seq > since).map(|(_, e)| e).collect()
        };
        json!({ "next": next - 1, "events": events })
    }
}

impl EventSink for Hub {
    fn emit(&self, event: &str, payload: Value) {
        let mut guard = self.state.lock().unwrap();
        let seq = guard.0;
        guard.0 += 1;
        guard.1.push_back((seq, json!({ "event": event, "payload": payload })));
        while guard.1.len() > KEEP {
            guard.1.pop_front();
        }
        self.changed.notify_all();
    }
}

/// 浏览器里只有一个标签页，打开某个界面就让页面跳到对应的 `?view=`。
struct DevShell {
    hub: Arc<Hub>,
}

impl Shell for DevShell {
    fn open_view(&self, view: &str) {
        self.hub.emit("dev:open", json!({ "view": view }));
    }

    fn close_view(&self, view: &str) {
        self.hub.emit("dev:close", json!({ "view": view }));
    }
}

fn json_response(value: Value) -> Response<std::io::Cursor<Vec<u8>>> {
    let header = Header::from_bytes("Content-Type", "application/json").unwrap();
    Response::from_string(value.to_string()).with_header(header)
}

fn main() {
    let data_dir = std::env::var("ASIDE_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.devdata"));
    let hub = Arc::new(Hub { state: Mutex::new((1, VecDeque::new())), changed: Condvar::new() });
    let clock = Arc::new(OffsetClock::default());
    let app = Aside::open(data_dir.clone(), clock.clone(), hub.clone(), Arc::new(DevShell { hub: hub.clone() }))
        .expect("无法初始化 Aside");
    Aside::spawn_ticker(app.clone());

    let addr = "127.0.0.1:1430";
    let server = Server::http(addr).expect("端口 1430 被占用");
    println!("开发桥已启动：http://{addr}，数据目录：{}", data_dir.display());

    for mut request in server.incoming_requests() {
        let app = app.clone();
        let hub = hub.clone();
        let clock = clock.clone();
        std::thread::spawn(move || {
            let url = request.url().to_string();
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            let response = match (request.method(), url.split('?').next().unwrap_or("")) {
                (Method::Post, "/call") => {
                    let input: Value = serde_json::from_str(&body).unwrap_or(Value::Null);
                    let cmd = input["cmd"].as_str().unwrap_or_default();
                    let caller = input["window"].as_str().unwrap_or("main");
                    match dispatch(&app, cmd, input["args"].clone(), caller) {
                        Ok(value) => json!({ "ok": true, "value": value }),
                        Err(err) => json!({ "ok": false, "error": err.to_string() }),
                    }
                }
                (Method::Get, "/events") => {
                    let since = url
                        .split("since=")
                        .nth(1)
                        .and_then(|s| s.parse::<i64>().ok())
                        .unwrap_or(-1);
                    hub.poll(since)
                }
                (Method::Post, "/dev/advance") => {
                    let input: Value = serde_json::from_str(&body).unwrap_or(Value::Null);
                    clock.advance(input["ms"].as_i64().unwrap_or(0));
                    json!({ "ok": true })
                }
                _ => json!({ "ok": false, "error": "not found" }),
            };
            let _ = request.respond(json_response(response));
        });
    }
}
