use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::Serialize;

use crate::error::{AppResult, invalid};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Subject {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub sort: i64,
    pub archived: bool,
    pub is_fallback: bool,
}

const COLUMNS: &str = "id, name, color, sort, archived_at IS NOT NULL, is_fallback";
const MAX_NAME_CHARS: usize = 10;

fn from_row(r: &Row) -> rusqlite::Result<Subject> {
    Ok(Subject {
        id: r.get(0)?,
        name: r.get(1)?,
        color: r.get(2)?,
        sort: r.get(3)?,
        archived: r.get(4)?,
        is_fallback: r.get(5)?,
    })
}

pub fn list(c: &Connection, include_archived: bool) -> AppResult<Vec<Subject>> {
    let filter = if include_archived { "" } else { "WHERE archived_at IS NULL" };
    let mut stmt = c.prepare(&format!("SELECT {COLUMNS} FROM subjects {filter} ORDER BY sort, id"))?;
    let rows = stmt.query_map([], from_row)?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get(c: &Connection, id: i64) -> AppResult<Subject> {
    let found = c
        .query_row(&format!("SELECT {COLUMNS} FROM subjects WHERE id = ?1"), [id], from_row)
        .optional()?;
    match found {
        Some(s) => Ok(s),
        None => invalid("找不到这个科目"),
    }
}

/// 兜底科目“其他”。
pub fn fallback(c: &Connection) -> AppResult<Subject> {
    Ok(c.query_row(&format!("SELECT {COLUMNS} FROM subjects WHERE is_fallback = 1"), [], from_row)?)
}

fn clean_name(name: &str) -> AppResult<String> {
    let name = name.trim();
    if name.is_empty() {
        return invalid("科目名称不能为空");
    }
    if name.chars().count() > MAX_NAME_CHARS {
        return invalid(format!("科目名称最多 {MAX_NAME_CHARS} 个字"));
    }
    Ok(name.to_string())
}

fn check_color(color: &str) -> AppResult<()> {
    let ok = color.len() == 7
        && color.starts_with('#')
        && color[1..].chars().all(|ch| ch.is_ascii_hexdigit());
    if ok { Ok(()) } else { invalid("颜色格式不对，应该像 #8DB59A 这样") }
}

fn ensure_unique(c: &Connection, name: &str, except: Option<i64>) -> AppResult<()> {
    let taken: Option<i64> = c
        .query_row("SELECT id FROM subjects WHERE name = ?1", [name], |r| r.get(0))
        .optional()?;
    match taken {
        Some(id) if Some(id) != except => invalid(format!("已经有一个叫“{name}”的科目了")),
        _ => Ok(()),
    }
}

pub fn create(c: &Connection, name: &str, color: &str) -> AppResult<Subject> {
    let name = clean_name(name)?;
    check_color(color)?;
    ensure_unique(c, &name, None)?;
    let sort: i64 = c.query_row("SELECT COALESCE(MAX(sort), -1) + 1 FROM subjects", [], |r| r.get(0))?;
    c.execute(
        "INSERT INTO subjects (name, color, sort) VALUES (?1, ?2, ?3)",
        params![name, color, sort],
    )?;
    get(c, c.last_insert_rowid())
}

pub fn update(c: &Connection, id: i64, name: Option<&str>, color: Option<&str>) -> AppResult<Subject> {
    let current = get(c, id)?;
    if let Some(name) = name {
        let name = clean_name(name)?;
        if name != current.name {
            if current.is_fallback {
                return invalid("“其他”只能改颜色");
            }
            ensure_unique(c, &name, Some(id))?;
            c.execute("UPDATE subjects SET name = ?1 WHERE id = ?2", params![name, id])?;
        }
    }
    if let Some(color) = color {
        check_color(color)?;
        c.execute("UPDATE subjects SET color = ?1 WHERE id = ?2", params![color, id])?;
    }
    get(c, id)
}

pub fn archive(c: &Connection, id: i64) -> AppResult<()> {
    if get(c, id)?.is_fallback {
        return invalid("“其他”用来兜底，不能归档");
    }
    c.execute(
        "UPDATE subjects SET archived_at = ?1 WHERE id = ?2 AND archived_at IS NULL",
        params![super::now_ms(), id],
    )?;
    Ok(())
}

pub fn restore(c: &Connection, id: i64) -> AppResult<()> {
    get(c, id)?;
    c.execute("UPDATE subjects SET archived_at = NULL WHERE id = ?1", [id])?;
    Ok(())
}

pub fn reorder(c: &Connection, ids: &[i64]) -> AppResult<()> {
    for (index, id) in ids.iter().enumerate() {
        if c.execute("UPDATE subjects SET sort = ?1 WHERE id = ?2", params![index as i64, id])? != 1 {
            return invalid("找不到这个科目");
        }
    }
    Ok(())
}
