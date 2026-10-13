//! 开发桥：把后端命令通过 HTTP 暴露出来，只在开发时编译（`--features devserver`）。
//!
//! - 浏览器开发：`pnpm dev:bridge` 起一个不开窗口的后端，配合 `pnpm dev` 在浏览器里点界面。
//! - 真机检查：`ASIDE_DEV_BRIDGE=1 pnpm tauri dev --features devserver`，可以用 curl 驱动真实窗口。
//!
//! - `POST /call`    `{cmd, args, window}` → `{ok, value}` 或 `{ok: false, error}`
//! - `GET  /events?since=N` 长轮询，返回序号大于 N 的事件（只有浏览器开发时有）
//! - `POST /dev/advance` `{ms}` 让时钟快进（只有浏览器开发时有）

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use serde_json::{Value, json};
use tiny_http::{Header, Method, Response, Server};

use crate::app::Aside;
use crate::clock::OffsetClock;
use crate::commands::dispatch;
use crate::events::EventSink;
use crate::shell::Shell;

const KEEP: usize = 500;

pub struct Hub {
    state: Mutex<(i64, VecDeque<(i64, Value)>)>,
    changed: Condvar,
}

impl Default for Hub {
    fn default() -> Self {
        Hub { state: Mutex::new((1, VecDeque::new())), changed: Condvar::new() }
    }
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
pub struct DevShell {
    pub hub: Arc<Hub>,
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

/// 阻塞地处理请求，每个请求一个线程。
pub fn serve(app: Arc<Aside>, addr: &str, hub: Option<Arc<Hub>>, clock: Option<Arc<OffsetClock>>) {
    let server = match Server::http(addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("开发桥启动不了（{addr}）：{e}");
            return;
        }
    };
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
                (Method::Get, "/events") if hub.is_some() => {
                    let since = url
                        .split("since=")
                        .nth(1)
                        .and_then(|s| s.parse::<i64>().ok())
                        .unwrap_or(-1);
                    hub.as_ref().unwrap().poll(since)
                }
                (Method::Post, "/dev/advance") if clock.is_some() => {
                    let input: Value = serde_json::from_str(&body).unwrap_or(Value::Null);
                    clock.as_ref().unwrap().advance(input["ms"].as_i64().unwrap_or(0));
                    json!({ "ok": true })
                }
                _ => json!({ "ok": false, "error": "not found" }),
            };
            let _ = request.respond(json_response(response));
        });
    }
}
