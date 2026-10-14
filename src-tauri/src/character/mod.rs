//! 形象包：读 character.json，核对每个状态的序列图。

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use image::GenericImageView;
use serde::{Deserialize, Serialize};

use crate::error::{AppResult, invalid};

mod penguin;

pub(crate) const FRAME_W: u32 = 360;
pub(crate) const FRAME_H: u32 = 440;
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

#[derive(Debug, Clone, Serialize)]
pub struct Character {
    pub id: String,
    pub name: String,
    pub builtin: bool,
    /// 导入形象的文件夹。内置形象为空，界面用自己打包的图。
    pub dir: String,
}

const BUILTIN_ID: &str = "gaga";
const ALLOWED: &[&str] = &[
    "character.json",
    "focus.png",
    "pause.png",
    "last5.png",
    "done.png",
    "blink.png",
    "preview.png",
    "done.mp3",
];
const MAX_UNPACK: u64 = 30 * 1024 * 1024;

pub fn list(data_dir: &Path) -> Vec<Character> {
    let mut out = vec![Character { id: BUILTIN_ID.into(), name: "嘎嘎".into(), builtin: true, dir: String::new() }];
    let root = data_dir.join("characters");
    let Ok(entries) = std::fs::read_dir(&root) else {
        return out;
    };
    let mut found = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Ok(pack) = load_pack(&path) else {
            continue;
        };
        let Some(id) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if id.starts_with('.') {
            continue;
        }
        found.push(Character {
            id: id.to_string(),
            name: pack.name,
            builtin: false,
            dir: path.to_string_lossy().into_owned(),
        });
    }
    found.sort_by(|a, b| a.id.cmp(&b.id));
    out.extend(found);
    out
}

/// 把文件夹或 zip 复制进数据目录。原文件之后删掉也不影响。
pub fn install(data_dir: &Path, src: &Path) -> AppResult<Character> {
    let stage = data_dir.join("characters").join(".incoming");
    let _ = std::fs::remove_dir_all(&stage);
    std::fs::create_dir_all(&stage)?;
    if src.is_dir() {
        copy_pack(src, &stage)?;
    } else if src.extension().and_then(|e| e.to_str()).unwrap_or("").eq_ignore_ascii_case("zip") {
        unzip(src, &stage)?;
    } else {
        return invalid("请选择形象文件夹或 zip");
    }
    let root = pack_root(&stage)?;
    let pack = load_pack(&root)?;
    let id = folder_id(&pack.name, data_dir);
    let dest = data_dir.join("characters").join(&id);
    if dest.exists() {
        std::fs::remove_dir_all(&dest)?;
    }
    if root == stage {
        std::fs::rename(&stage, &dest)?;
    } else {
        std::fs::rename(&root, &dest)?;
        let _ = std::fs::remove_dir_all(&stage);
    }
    Ok(Character { id, name: pack.name, builtin: false, dir: dest.to_string_lossy().into_owned() })
}

fn folder_id(name: &str, data_dir: &Path) -> String {
    let slug: String = name.chars().filter(|c| !c.is_whitespace()).take(12).collect();
    let slug = if slug.is_empty() { "pack".into() } else { slug };
    let mut id = slug.clone();
    let mut n = 2;
    while data_dir.join("characters").join(&id).exists() || id == BUILTIN_ID {
        id = format!("{slug}-{n}");
        n += 1;
    }
    id
}

fn pack_root(stage: &Path) -> AppResult<PathBuf> {
    if stage.join("character.json").is_file() {
        return Ok(stage.to_path_buf());
    }
    let mut found = Vec::new();
    if let Ok(entries) = std::fs::read_dir(stage) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("character.json").is_file() {
                found.push(path);
            }
        }
    }
    if found.len() == 1 {
        return Ok(found.remove(0));
    }
    invalid("缺少 character.json")
}

fn copy_pack(src: &Path, dest: &Path) -> AppResult<()> {
    for name in ALLOWED {
        let from = src.join(name);
        if from.is_file() {
            std::fs::copy(from, dest.join(name))?;
        }
    }
    Ok(())
}

fn unzip(src: &Path, dest: &Path) -> AppResult<()> {
    let file = std::fs::File::open(src)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|_| crate::error::AppError::Invalid("这个 zip 读不出来".into()))?;
    let mut total = 0u64;
    for index in 0..archive.len() {
        let mut item = archive.by_index(index).map_err(|_| crate::error::AppError::Invalid("这个 zip 读不出来".into()))?;
        if item.is_dir() {
            continue;
        }
        let name = item.name().replace('\\', "/");
        if name.contains("..") {
            return invalid("形象包里的路径不对");
        }
        let file_name = Path::new(&name).file_name().and_then(|s| s.to_str()).unwrap_or("");
        if !ALLOWED.contains(&file_name) {
            continue;
        }
        total += item.size();
        if total > MAX_UNPACK {
            return invalid("形象包太大");
        }
        let parent = Path::new(&name).parent().unwrap_or(Path::new(""));
        let folder = dest.join(parent);
        std::fs::create_dir_all(&folder)?;
        let mut out = std::fs::File::create(folder.join(file_name))?;
        let mut buf = Vec::new();
        item.read_to_end(&mut buf).map_err(|_| crate::error::AppError::Invalid("这个 zip 读不出来".into()))?;
        out.write_all(&buf)?;
    }
    Ok(())
}

pub fn allowed_file(name: &str) -> bool {
    ALLOWED.contains(&name)
}

pub fn keep_character(id: &str) -> bool {
    matches!(id, "gaga" | "capsule") || (!id.is_empty() && !id.contains('/') && !id.contains('\\') && !id.contains(".."))
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

    #[test]
    fn the_placeholder_penguin_is_a_valid_pack() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/assets/characters/gaga");
        std::fs::create_dir_all(&root).unwrap();
        let states = [("focus", 4u32), ("pause", 4), ("last5", 4), ("done", 4), ("blink", 2)];
        std::fs::write(
            root.join("character.json"),
            r#"{"name":"嘎嘎","author":"Aside","version":1,"states":{
                "focus":{"frames":4,"ms":180},
                "pause":{"frames":4,"ms":180},
                "last5":{"frames":4,"ms":180},
                "done":{"frames":4,"ms":160},
                "blink":{"frames":2,"ms":160}
            }}"#,
        )
        .unwrap();
        for (state, frames) in states {
            super::penguin::sheet(state, frames).save(root.join(format!("{state}.png"))).unwrap();
        }
        super::penguin::sheet("focus", 1).save(root.join("preview.png")).unwrap();
        let pack = load_pack(&root).unwrap();
        assert_eq!(pack.name, "嘎嘎");
    }

    #[test]
    fn a_folder_and_a_zip_both_install_and_the_original_can_go() {
        let data = tempfile::tempdir().unwrap();
        let src = tempfile::tempdir().unwrap();
        write_pack(src.path(), 2);
        let installed = install(data.path(), src.path()).unwrap();
        assert_eq!(installed.name, "嘎嘎");
        assert!(!installed.builtin);
        std::fs::remove_dir_all(src.path()).unwrap();
        let listed = list(data.path());
        assert_eq!(listed[0].id, "gaga");
        assert!(listed.iter().any(|c| c.id == installed.id && c.name == "嘎嘎"));

        let zip_dir = tempfile::tempdir().unwrap();
        write_pack(&zip_dir.path().join("bird"), 2);
        std::fs::write(zip_dir.path().join("bird").join("character.json"), include_name("小鸟")).unwrap();
        let zip_path = data.path().join("bird.zip");
        zip_dir_to(&zip_dir.path().join("bird"), &zip_path);
        let bird = install(data.path(), &zip_path).unwrap();
        assert_eq!(bird.name, "小鸟");
        assert!(load_pack(&data.path().join("characters").join(&bird.id)).is_ok());
    }

    #[test]
    fn a_zip_missing_a_sheet_names_that_file() {
        let data = tempfile::tempdir().unwrap();
        let src = tempfile::tempdir().unwrap();
        write_pack(src.path(), 2);
        std::fs::remove_file(src.path().join("done.png")).unwrap();
        let zip_path = data.path().join("bad.zip");
        zip_dir_to(src.path(), &zip_path);
        let err = install(data.path(), &zip_path).unwrap_err().to_string();
        assert!(err.contains("缺少 done.png"), "{err}");
        assert!(list(data.path()).iter().all(|c| c.builtin));
    }

    fn include_name(name: &str) -> String {
        format!(
            r#"{{"name":"{name}","author":"Aside","version":1,"states":{{
                "focus":{{"frames":2,"ms":180}},
                "pause":{{"frames":2,"ms":180}},
                "last5":{{"frames":2,"ms":180}},
                "done":{{"frames":2,"ms":160}},
                "blink":{{"frames":2,"ms":160}}
            }}}}"#
        )
    }

    fn zip_dir_to(dir: &Path, zip_path: &Path) {
        let file = std::fs::File::create(zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let opts = zip::write::SimpleFileOptions::default();
        for name in ALLOWED {
            let path = dir.join(name);
            if !path.is_file() {
                continue;
            }
            zip.start_file(format!("bird/{name}"), opts).unwrap();
            zip.write_all(&std::fs::read(path).unwrap()).unwrap();
        }
        zip.finish().unwrap();
    }
}
