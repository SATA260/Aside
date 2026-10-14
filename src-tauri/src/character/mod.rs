//! 形象包：读 character.json，核对每个状态的序列图。

use std::path::Path;

use image::GenericImageView;
use serde::Deserialize;

use crate::error::{AppResult, invalid};

const FRAME_W: u32 = 360;
const FRAME_H: u32 = 440;
const MAX_FRAMES: u32 = 12;
const MAX_BYTES: u64 = 8 * 1024 * 1024;

const STATES: &[&str] = &["focus", "pause", "last5", "done", "blink"];

#[derive(Debug, Clone, Deserialize)]
pub struct StateClip {
    pub frames: u32,
    pub ms: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackFile {
    pub name: String,
    pub author: String,
    pub version: u32,
    pub states: std::collections::BTreeMap<String, StateClip>,
}

#[derive(Debug, Clone)]
pub struct Pack {
    pub name: String,
    pub author: String,
    pub dir: String,
}

pub fn load_pack(dir: &Path) -> AppResult<Pack> {
    let json_path = dir.join("character.json");
    if !json_path.is_file() {
        return invalid("缺少 character.json");
    }
    let raw = std::fs::read_to_string(&json_path)?;
    let file: PackFile = serde_json::from_str(&raw).map_err(|_| crate::error::AppError::Invalid("character.json 读不出来".into()))?;
    if file.name.trim().is_empty() || file.name.chars().count() > 10 {
        return invalid("形象名字要在 1 到 10 个字");
    }
    for state in STATES {
        let clip = file.states.get(*state).ok_or_else(|| crate::error::AppError::Invalid(format!("character.json 缺少 {state}")))?;
        if clip.frames == 0 || clip.frames > MAX_FRAMES {
            return invalid(format!("{state} 的帧数要在 1 到 {MAX_FRAMES} 之间"));
        }
        if clip.ms == 0 || clip.ms > 2000 {
            return invalid(format!("{state} 的每帧时长不对"));
        }
        check_sheet(&dir.join(format!("{state}.png")), clip.frames, state)?;
    }
    let preview = dir.join("preview.png");
    if !preview.is_file() {
        return invalid("缺少 preview.png");
    }
    let len = std::fs::metadata(&preview)?.len();
    if len == 0 || len > MAX_BYTES {
        return invalid("preview.png 太大");
    }
    Ok(Pack { name: file.name, author: file.author, dir: dir.to_string_lossy().into_owned() })
}

fn check_sheet(path: &Path, frames: u32, state: &str) -> AppResult<()> {
    let file_name = format!("{state}.png");
    if !path.is_file() {
        return invalid(format!("缺少 {file_name}"));
    }
    let len = std::fs::metadata(path)?.len();
    if len > MAX_BYTES {
        return invalid(format!("{file_name} 太大"));
    }
    let img = image::open(path).map_err(|_| crate::error::AppError::Invalid(format!("{file_name} 读不出来")))?;
    let (w, h) = img.dimensions();
    if h != FRAME_H || w != FRAME_W * frames {
        return invalid(format!("{file_name} 的宽度和 {frames} 帧对不上"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sheet(path: &Path, frames: u32) {
        let img = image::RgbaImage::new(FRAME_W * frames, FRAME_H);
        img.save(path).unwrap();
    }

    fn write_pack(dir: &Path, frames: u32) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(
            dir.join("character.json"),
            format!(
                r#"{{"name":"嘎嘎","author":"Aside","version":1,"states":{{
                "focus":{{"frames":{frames},"ms":180}},
                "pause":{{"frames":{frames},"ms":180}},
                "last5":{{"frames":{frames},"ms":180}},
                "done":{{"frames":{frames},"ms":160}},
                "blink":{{"frames":2,"ms":160}}
            }}}}"#
            ),
        )
        .unwrap();
        for state in ["focus", "pause", "last5", "done"] {
            sheet(&dir.join(format!("{state}.png")), frames);
        }
        sheet(&dir.join("blink.png"), 2);
        sheet(&dir.join("preview.png"), 1);
    }

    #[test]
    fn a_complete_pack_loads_its_name() {
        let dir = tempfile::tempdir().unwrap();
        write_pack(dir.path(), 4);
        let pack = load_pack(dir.path()).unwrap();
        assert_eq!(pack.name, "嘎嘎");
        assert_eq!(pack.author, "Aside");
    }

    #[test]
    fn a_missing_sheet_names_the_file() {
        let dir = tempfile::tempdir().unwrap();
        write_pack(dir.path(), 4);
        std::fs::remove_file(dir.path().join("pause.png")).unwrap();
        let err = load_pack(dir.path()).unwrap_err().to_string();
        assert!(err.contains("缺少 pause.png"), "{err}");
    }

    #[test]
    fn the_width_must_match_the_frame_count() {
        let dir = tempfile::tempdir().unwrap();
        write_pack(dir.path(), 4);
        sheet(&dir.path().join("focus.png"), 3);
        let err = load_pack(dir.path()).unwrap_err().to_string();
        assert!(err.contains("focus.png") && err.contains("4 帧"), "{err}");
    }
}
