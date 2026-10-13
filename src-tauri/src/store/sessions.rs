//! 专注记录和其中的运行段，以及进行中专注的心跳。

use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;

use crate::error::{AppResult, invalid};
use crate::stats::Piece;
use crate::timer::{Finished, Segment, Timer};

pub const MAX_NOTE_CHARS: usize = 200;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MockScore {
    pub score: i64,
    pub full_score: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Session {
    pub id: i64,
    pub subject_id: i64,
    pub mode: String,
    pub planned_ms: Option<i64>,
    pub started_at: i64,
    pub ended_at: i64,
    pub focused_ms: i64,
    pub task: String,
    pub note: String,
    pub source: String,
    pub segments: Vec<Segment>,
    pub mock: Option<MockScore>,
}

pub fn count(c: &Connection) -> AppResult<i64> {
    Ok(c.query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))?)
}

pub fn segments_of(c: &Connection, id: i64) -> AppResult<Vec<Segment>> {
    let mut stmt = c.prepare("SELECT start_at, end_at FROM segments WHERE session_id = ?1 ORDER BY start_at")?;
    let rows = stmt
        .query_map([id], |r| Ok(Segment { start: r.get(0)?, end: r.get(1)? }))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get(c: &Connection, id: i64) -> AppResult<Session> {
    let found = c
        .query_row(
            "SELECT s.id, s.subject_id, s.mode, s.planned_ms, s.started_at, s.ended_at, s.focused_ms,
                    s.task, s.note, s.source, m.score, m.full_score
             FROM sessions s LEFT JOIN mock_scores m ON m.session_id = s.id
             WHERE s.id = ?1",
            [id],
            |r| {
                let score: Option<i64> = r.get(10)?;
                let full_score: Option<i64> = r.get(11)?;
                Ok(Session {
                    id: r.get(0)?,
                    subject_id: r.get(1)?,
                    mode: r.get(2)?,
                    planned_ms: r.get(3)?,
                    started_at: r.get(4)?,
                    ended_at: r.get(5)?,
                    focused_ms: r.get(6)?,
                    task: r.get(7)?,
                    note: r.get(8)?,
                    source: r.get(9)?,
                    segments: Vec::new(),
                    mock: score.zip(full_score).map(|(score, full_score)| MockScore { score, full_score }),
                })
            },
        )
        .optional()?;
    let Some(mut session) = found else {
        return invalid("找不到这条记录");
    };
    session.segments = segments_of(c, id)?;
    Ok(session)
}

pub fn insert_segments(c: &Connection, id: i64, segments: &[Segment]) -> AppResult<()> {
    let mut stmt = c.prepare("INSERT INTO segments (session_id, start_at, end_at) VALUES (?1, ?2, ?3)")?;
    for s in segments.iter().filter(|s| s.len() > 0) {
        stmt.execute(params![id, s.start, s.end])?;
    }
    Ok(())
}

/// 计时结束后写入一条记录，返回新记录的 id。
pub fn insert_from_timer(c: &Connection, done: &Finished) -> AppResult<i64> {
    c.execute(
        "INSERT INTO sessions (subject_id, mode, planned_ms, started_at, ended_at, focused_ms, task, source)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'timer')",
        params![
            done.plan.subject_id,
            done.plan.mode.as_str(),
            done.plan.planned_ms,
            done.started_at,
            done.ended_at,
            done.focused_ms,
            done.plan.task,
        ],
    )?;
    let id = c.last_insert_rowid();
    insert_segments(c, id, &done.segments)?;
    Ok(id)
}

/// 和 [from, to) 有交集的运行段，带上所属科目。边界之外的部分由调用方裁掉。
pub fn pieces_between(c: &Connection, from: i64, to: i64) -> AppResult<Vec<Piece>> {
    let mut stmt = c.prepare(
        "SELECT s.subject_id, g.start_at, g.end_at
         FROM segments g JOIN sessions s ON s.id = g.session_id
         WHERE g.end_at > ?1 AND g.start_at < ?2
         ORDER BY g.start_at",
    )?;
    let rows = stmt
        .query_map(params![from, to], |r| Ok(Piece { subject_id: r.get(0)?, start: r.get(1)?, end: r.get(2)? }))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// 所有记录落在 [from, to) 里的专注时长，跨边界的运行段只算里面的部分。
pub fn focused_between(c: &Connection, from: i64, to: i64) -> AppResult<i64> {
    Ok(c.query_row(
        "SELECT COALESCE(SUM(MIN(end_at, ?2) - MAX(start_at, ?1)), 0)
         FROM segments WHERE end_at > ?1 AND start_at < ?2",
        params![from, to],
        |r| r.get(0),
    )?)
}

pub fn clean_note(note: &str) -> AppResult<String> {
    let note = note.trim();
    if note.chars().count() > MAX_NOTE_CHARS {
        return invalid(format!("备注最多 {MAX_NOTE_CHARS} 个字"));
    }
    Ok(note.to_string())
}

pub fn set_note(c: &Connection, id: i64, note: &str) -> AppResult<()> {
    let note = clean_note(note)?;
    if c.execute("UPDATE sessions SET note = ?1 WHERE id = ?2", params![note, id])? != 1 {
        return invalid("找不到这条记录");
    }
    Ok(())
}

pub fn save_active(c: &Connection, timer: &Timer, heartbeat_at: i64) -> AppResult<()> {
    c.execute(
        "INSERT INTO active_timer (id, state, heartbeat_at) VALUES (1, ?1, ?2)
         ON CONFLICT (id) DO UPDATE SET state = excluded.state, heartbeat_at = excluded.heartbeat_at",
        params![serde_json::to_string(timer)?, heartbeat_at],
    )?;
    Ok(())
}

pub fn load_active(c: &Connection) -> AppResult<Option<(Timer, i64)>> {
    let row: Option<(String, i64)> = c
        .query_row("SELECT state, heartbeat_at FROM active_timer WHERE id = 1", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .optional()?;
    Ok(match row {
        Some((state, heartbeat)) => Some((serde_json::from_str(&state)?, heartbeat)),
        None => None,
    })
}

pub fn clear_active(c: &Connection) -> AppResult<()> {
    c.execute("DELETE FROM active_timer", [])?;
    Ok(())
}
