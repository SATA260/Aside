//! 导入的背景音和背景图片。文件本身在数据文件夹里，这里只记名字和位置。

use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;

use crate::error::{AppResult, invalid};
use crate::store::now_ms;

#[derive(Debug, Clone, Serialize)]
pub struct Sound {
    pub id: i64,
    pub name: String,
    /// 绝对路径，界面要用它显示或播放
    pub path: String,
    pub gain_db: f64,
    pub duration_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Picture {
    pub id: i64,
    pub name: String,
    pub path: String,
}

pub(crate) struct SoundRow {
    pub id: i64,
    pub name: String,
    pub file: String,
    pub gain_db: f64,
    pub duration_ms: i64,
}

pub(crate) struct PictureRow {
    pub file: String,
}

fn abs(data_dir: &Path, file: &str) -> String {
    data_dir.join(file).to_string_lossy().into_owned()
}

pub fn list_sounds(c: &Connection, data_dir: &Path) -> AppResult<Vec<Sound>> {
    let mut stmt = c.prepare("SELECT id, name, file, gain_db, duration_ms FROM sounds ORDER BY id")?;
    let rows = stmt.query_map([], |r| {
        Ok(SoundRow {
            id: r.get(0)?,
            name: r.get(1)?,
            file: r.get(2)?,
            gain_db: r.get(3)?,
            duration_ms: r.get(4)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        let row = row?;
        out.push(Sound {
            id: row.id,
            name: row.name,
            path: abs(data_dir, &row.file),
            gain_db: row.gain_db,
            duration_ms: row.duration_ms,
        });
    }
    Ok(out)
}

pub(crate) fn sound(c: &Connection, id: i64) -> AppResult<SoundRow> {
    c.query_row("SELECT id, name, file, gain_db, duration_ms FROM sounds WHERE id = ?1", [id], |r| {
        Ok(SoundRow {
            id: r.get(0)?,
            name: r.get(1)?,
            file: r.get(2)?,
            gain_db: r.get(3)?,
            duration_ms: r.get(4)?,
        })
    })
    .optional()?
    .ok_or_else(|| crate::error::AppError::Invalid("没有这段声音".into()))
}

pub fn insert_sound(c: &Connection, name: &str, file: &str, gain_db: f64, duration_ms: i64) -> AppResult<i64> {
    c.execute(
        "INSERT INTO sounds (name, file, gain_db, duration_ms, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![name, file, gain_db, duration_ms, now_ms()],
    )?;
    Ok(c.last_insert_rowid())
}

pub fn rename_sound(c: &Connection, id: i64, name: &str) -> AppResult<()> {
    let n = c.execute("UPDATE sounds SET name = ?1 WHERE id = ?2", params![name, id])?;
    if n == 0 {
        return invalid("没有这段声音");
    }
    Ok(())
}

pub fn delete_sound(c: &Connection, id: i64) -> AppResult<String> {
    let row = sound(c, id)?;
    c.execute("DELETE FROM sounds WHERE id = ?1", [id])?;
    Ok(row.file)
}

pub fn list_pictures(c: &Connection, data_dir: &Path) -> AppResult<Vec<Picture>> {
    let mut stmt = c.prepare("SELECT id, name, file FROM images ORDER BY id")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?)))?;
    let mut out = Vec::new();
    for row in rows {
        let (id, name, file) = row?;
        out.push(Picture { id, name, path: abs(data_dir, &file) });
    }
    Ok(out)
}

pub(crate) fn picture_file(c: &Connection, id: i64) -> AppResult<PictureRow> {
    c.query_row("SELECT file FROM images WHERE id = ?1", [id], |r| Ok(PictureRow { file: r.get(0)? }))
        .optional()?
        .ok_or_else(|| crate::error::AppError::Invalid("没有这张图片".into()))
}

pub fn insert_picture(c: &Connection, name: &str, file: &str) -> AppResult<i64> {
    c.execute(
        "INSERT INTO images (name, file, created_at) VALUES (?1, ?2, ?3)",
        params![name, file, now_ms()],
    )?;
    Ok(c.last_insert_rowid())
}

pub fn rename_picture(c: &Connection, id: i64, name: &str) -> AppResult<()> {
    let n = c.execute("UPDATE images SET name = ?1 WHERE id = ?2", params![name, id])?;
    if n == 0 {
        return invalid("没有这张图片");
    }
    Ok(())
}

pub fn delete_picture(c: &Connection, id: i64) -> AppResult<String> {
    let row = picture_file(c, id)?;
    c.execute("DELETE FROM images WHERE id = ?1", [id])?;
    Ok(row.file)
}

pub fn sound_path(data_dir: &Path, file: &str) -> PathBuf {
    data_dir.join(file)
}
