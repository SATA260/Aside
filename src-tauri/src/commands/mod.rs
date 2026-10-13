//! 暴露给前端调用的命令，只做参数转换，业务都交给其他模块。

use crate::timer::Snapshot;

#[tauri::command]
pub fn timer_snapshot() -> Snapshot {
    Snapshot::idle()
}
