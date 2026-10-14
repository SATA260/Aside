//! 勿扰没有公开接口。macOS 运行用户自己建的快捷指令，Windows 只提示按 Win+N。

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FocusStep {
    /// 运行这个名字的快捷指令
    Run(String),
    /// 提示一次可以按 Win+N
    Hint,
}

/// `hint_open` 为真时，Windows 还没点过“不再提示”。
pub fn focus_step(os: &str, starting: bool, on_name: &str, off_name: &str, hint_open: bool) -> Option<FocusStep> {
    match os {
        "macos" => {
            let name = if starting { on_name } else { off_name };
            let name = name.trim();
            if name.is_empty() { None } else { Some(FocusStep::Run(name.to_string())) }
        }
        "windows" if starting && hint_open => Some(FocusStep::Hint),
        _ => None,
    }
}

pub fn missing_shortcut(name: &str) -> String {
    format!("找不到名为‘{name}’的快捷指令")
}

pub const WIN_HINT: &str = "可以按 Win+N 打开勿扰";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mac_runs_the_open_name_at_the_start_and_the_close_name_at_the_end() {
        assert_eq!(
            focus_step("macos", true, "打开专注模式", "关闭专注模式", true),
            Some(FocusStep::Run("打开专注模式".into()))
        );
        assert_eq!(
            focus_step("macos", false, "打开专注模式", "关闭专注模式", true),
            Some(FocusStep::Run("关闭专注模式".into()))
        );
        assert_eq!(focus_step("macos", true, "  ", "", true), None);
    }

    #[test]
    fn windows_hints_once_until_it_is_dismissed() {
        assert_eq!(focus_step("windows", true, "", "", true), Some(FocusStep::Hint));
        assert_eq!(focus_step("windows", true, "", "", false), None);
        assert_eq!(focus_step("windows", false, "", "", true), None);
        assert_eq!(focus_step("linux", true, "打开专注模式", "关闭专注模式", true), None);
    }
}
