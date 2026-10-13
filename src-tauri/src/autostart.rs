//! 开机自启。开没开以系统里的登录项为准，不另外存一份，免得两边对不上。

use crate::app::Aside;
use crate::error::{AppError, AppResult};

/// 开机自启时带上的参数；带着它启动就只待在托盘或菜单栏里。
pub const ARG: &str = "--autostart";

pub fn launched_at_login<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter().skip(1).any(|a| a.as_ref() == ARG)
}

impl Aside {
    pub fn autostart_enabled(&self) -> bool {
        self.shell.autostart()
    }

    pub fn autostart_set(&self, on: bool) -> AppResult<bool> {
        self.shell.set_autostart(on).map_err(|e| {
            let what = if on { "打开" } else { "关掉" };
            AppError::Invalid(format!("系统没让 Aside {what}开机自启：{e}"))
        })?;
        Ok(self.shell.autostart())
    }
}
