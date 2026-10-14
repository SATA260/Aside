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

/// 课程网页里只允许调用这一个命令：视频播放状态，或者退出只看视频。
pub const PAUSE_SCRIPT: &str = r#"( () => {
  if (window.__asidePause) return;
  window.__asidePause = true;
  const send = (playing) => {
    const invoke = window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke;
    if (invoke) invoke("call", { cmd: "video_playing", args: { playing } });
  };
  document.addEventListener("play", (event) => { if (event.target && event.target.tagName === "VIDEO") send(true); }, true);
  document.addEventListener("pause", (event) => { if (event.target && event.target.tagName === "VIDEO") send(false); }, true);
} )();"#;
pub fn video_script(time: &str, on: bool) -> String {
    let time = time.replace('\\', "").replace('\'', "");
    if !on {
        return "(() => { document.documentElement.classList.remove('aside-video'); document.getElementById('aside-style')?.remove(); document.getElementById('aside-bar')?.remove(); window.__asideOff?.(); })();".into();
    }
    format!(
        r#"( () => {{
  const clear = () => {{
    document.documentElement.classList.remove("aside-video");
    document.getElementById("aside-style")?.remove();
    document.getElementById("aside-bar")?.remove();
    window.__asideOff?.();
  }};
  const videos = Array.from(document.querySelectorAll("video"));
  const playing = videos.filter((video) => !video.paused && !video.ended && video.readyState >= 2);
  let best = null;
  let area = 0;
  for (const video of playing) {{
    const box = video.getBoundingClientRect();
    const size = Math.max(box.width * box.height, video.videoWidth * video.videoHeight);
    if (size >= area) {{ area = size; best = video; }}
  }}
  clear();
  if (!best) {{
    const note = document.createElement("div");
    note.id = "aside-bar";
    note.textContent = "这个页面上没有找到正在播放的视频";
    note.style.cssText = "position:fixed;right:16px;top:16px;z-index:2147483647;color:#fff;font:14px sans-serif;";
    document.body.appendChild(note);
    return;
  }}
  best.classList.add("aside-focus");
  const style = document.createElement("style");
  style.id = "aside-style";
  style.textContent = "html.aside-video body{{visibility:hidden}} .aside-focus{{visibility:visible!important;position:fixed!important;inset:0!important;width:100vw!important;height:100vh!important;z-index:2147483646;object-fit:contain;background:#000}} #aside-bar{{position:fixed;top:0;right:0;bottom:0;width:72px;z-index:2147483647;display:flex;align-items:center;justify-content:center;background:rgba(0,0,0,.35);color:#fff;font:20px sans-serif;opacity:1;transition:opacity .4s}} #aside-bar.idle{{opacity:0}}";
  document.documentElement.classList.add("aside-video");
  document.documentElement.appendChild(best);
  document.documentElement.appendChild(style);
  const bar = document.createElement("div");
  bar.id = "aside-bar";
  bar.textContent = "{time}";
  document.documentElement.appendChild(bar);
  let idle;
  const wake = () => {{ bar.classList.remove("idle"); clearTimeout(idle); idle = setTimeout(() => bar.classList.add("idle"), 3000); }};
  window.addEventListener("mousemove", wake);
  const onKey = (event) => {{
    if (event.key === "Escape") {{
      clear();
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("mousemove", wake);
      const invoke = window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke;
      if (invoke) invoke("call", {{ cmd: "video_playing", args: {{ exit: true }} }});
    }}
  }};
  window.addEventListener("keydown", onKey);
  window.__asideOff = () => {{ window.removeEventListener("keydown", onKey); window.removeEventListener("mousemove", wake); }};
  wake();
}} )();"#
    )
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

    #[test]
    fn video_only_keeps_the_largest_playing_video_and_escape_leaves_it() {
        let on = video_script("32:48", true);
        assert!(on.contains("32:48"));
        assert!(on.contains("video"));
        assert!(on.contains("Escape"));
        assert!(on.contains("这个页面上没有找到正在播放的视频"));
        let off = video_script("", false);
        assert!(off.contains("aside-video"));
        assert!(!off.contains("32:48"));
    }
}
