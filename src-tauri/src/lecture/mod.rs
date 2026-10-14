//! 网课窗口里的课程地址和书签。网页本身由窗口层打开。

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Bookmark {
    pub name: &'static str,
    pub url: &'static str,
}

pub fn bookmarks() -> Vec<Bookmark> {
    vec![
        Bookmark { name: "哔哩哔哩", url: "https://www.bilibili.com" },
        Bookmark { name: "中国大学MOOC", url: "https://www.icourse163.org" },
        Bookmark { name: "学堂在线", url: "https://www.xuetangx.com" },
        Bookmark { name: "研招网", url: "https://yz.chsi.com.cn" },
    ]
}

/// 补上 https，并且只允许 http / https。
pub fn course_url(raw: &str) -> Result<String, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("地址是空的".into());
    }
    let lower = raw.to_ascii_lowercase();
    if lower.starts_with("javascript:") || lower.starts_with("file:") || lower.starts_with("data:") {
        return Err("只打开 http 和 https 的网页".into());
    }
    if raw.contains(' ') || raw.contains('\n') {
        return Err("这个地址打不开".into());
    }
    if lower.starts_with("http://") || lower.starts_with("https://") {
        Ok(raw.to_string())
    } else {
        Ok(format!("https://{raw}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bookmarks_cover_the_four_sites() {
        let names: Vec<_> = bookmarks().into_iter().map(|b| b.name).collect();
        assert_eq!(names, ["哔哩哔哩", "中国大学MOOC", "学堂在线", "研招网"]);
    }

    #[test]
    fn a_bare_host_gets_https_and_other_schemes_are_refused() {
        assert_eq!(course_url("bilibili.com/video/1").unwrap(), "https://bilibili.com/video/1");
        assert_eq!(course_url("https://www.bilibili.com").unwrap(), "https://www.bilibili.com");
        assert!(course_url("javascript:alert(1)").unwrap_err().contains("http"));
        assert!(course_url("  ").is_err());
        assert!(course_url("not a url").is_err());
    }
}
