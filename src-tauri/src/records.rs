//! 记录页：某一天的记录，以及修改、删除一条记录。

use chrono::{Local, NaiveDate, TimeZone};
use serde::{Deserialize, Serialize};

use crate::app::Aside;
use crate::error::{AppResult, invalid};
use crate::stats::{clipped, day_start_of};
use crate::store::sessions::{self, Session};
use crate::store::subjects;
use crate::timer::{DAY_MS, MINUTE_MS, Phase};

/// 两段时间是否重叠。首尾刚好相接不算重叠。
pub fn overlaps(a0: i64, a1: i64, b0: i64, b1: i64) -> bool {
    a0 < b1 && b0 < a1
}

/// 起止时刻要落在 1 分钟到 24 小时之间，并且不能晚于现在。
pub fn check_span(started: i64, ended: i64, now: i64) -> AppResult<()> {
    let len = ended.saturating_sub(started);
    if started >= ended || len < MINUTE_MS {
        return invalid("时长至少 1 分钟");
    }
    if len > DAY_MS {
        return invalid("时长最长 24 小时");
    }
    if ended > now {
        return invalid("不能晚于当前时刻");
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct DayEntry {
    #[serde(flatten)]
    pub session: Session,
    pub subject_name: String,
    pub subject_color: String,
    /// 落在这一天里的专注时长，跨零点的记录只算这一天的部分
    pub day_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DayView {
    pub date: NaiveDate,
    /// 这一天的 0 点和次日 0 点
    pub start: i64,
    pub end: i64,
    pub total_ms: i64,
    pub entries: Vec<DayEntry>,
}

/// 补上一段已经发生的专注。
#[derive(Debug, Clone, Deserialize)]
pub struct ManualEntry {
    pub subject_id: i64,
    pub started_at: i64,
    pub ended_at: i64,
    #[serde(default)]
    pub task: String,
    #[serde(default)]
    pub note: String,
}

/// 只改给出的字段。改了起止时间的记录按连续学习重算时长。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SessionPatch {
    pub subject_id: Option<i64>,
    pub task: Option<String>,
    pub note: Option<String>,
    pub started_at: Option<i64>,
    pub ended_at: Option<i64>,
}

impl Aside {
    pub fn day_view(&self, date: NaiveDate) -> AppResult<DayView> {
        let start = day_start_of(date);
        let end = day_start_of(date.succ_opt().unwrap_or(date));
        let entries = self.store.read(|c| {
            let all = subjects::list(c, true)?;
            sessions::ids_between(c, start, end)?
                .into_iter()
                .map(|id| {
                    let session = sessions::get(c, id)?;
                    let subject = all.iter().find(|s| s.id == session.subject_id);
                    Ok(DayEntry {
                        subject_name: subject.map(|s| s.name.clone()).unwrap_or_default(),
                        subject_color: subject.map(|s| s.color.clone()).unwrap_or_default(),
                        day_ms: session
                            .segments
                            .iter()
                            .map(|g| clipped(g, start, end))
                            .sum(),
                        session,
                    })
                })
                .collect::<AppResult<Vec<_>>>()
        })?;
        let total_ms = entries.iter().map(|e| e.day_ms).sum();
        Ok(DayView {
            date,
            start,
            end,
            total_ms,
            entries,
        })
    }

    pub fn session_update(&self, id: i64, patch: SessionPatch) -> AppResult<Session> {
        let now = self.clock.now_ms();
        let active = self.active_wall(now);
        let session = self.store.write(|c| {
            let mut s = sessions::get(c, id)?;
            if let Some(subject_id) = patch.subject_id.filter(|&sid| sid != s.subject_id) {
                if subjects::get(c, subject_id)?.archived {
                    return invalid("这个科目已经归档了");
                }
                s.subject_id = subject_id;
            }
            if let Some(task) = &patch.task {
                s.task = sessions::clean_task(task)?;
            }
            if let Some(note) = &patch.note {
                s.note = sessions::clean_note(note)?;
            }
            let started = patch.started_at.unwrap_or(s.started_at);
            let ended = patch.ended_at.unwrap_or(s.ended_at);
            let moving = started != s.started_at || ended != s.ended_at;
            if moving {
                ensure_free(c, started, ended, id, now, active)?;
            }
            sessions::update_fields(c, &s)?;
            if moving {
                sessions::reschedule(c, id, started, ended)?;
            }
            sessions::get(c, id)
        })?;
        self.data_changed();
        Ok(session)
    }

    /// 正在进行的专注所占的墙钟区间。暂停的空隙也算在里面，因为结束时记录会带上这段。
    fn active_wall(&self, now: i64) -> Option<(i64, i64)> {
        let view = self.timer_view();
        if !matches!(view.snapshot.phase, Phase::Running | Phase::Paused) {
            return None;
        }
        view.snapshot.started_at.map(|start| (start, now))
    }

    pub fn session_add(&self, entry: ManualEntry) -> AppResult<Session> {
        let now = self.clock.now_ms();
        let active = self.active_wall(now);
        let session = self.store.write(|c| {
            if subjects::get(c, entry.subject_id)?.archived {
                return invalid("这个科目已经归档了");
            }
            ensure_free(c, entry.started_at, entry.ended_at, 0, now, active)?;
            let task = sessions::clean_task(&entry.task)?;
            let note = sessions::clean_note(&entry.note)?;
            let id = sessions::insert_manual(
                c,
                entry.subject_id,
                entry.started_at,
                entry.ended_at,
                &task,
                &note,
            )?;
            sessions::get(c, id)
        })?;
        self.data_changed();
        Ok(session)
    }

    pub fn session_set_note(&self, id: i64, note: &str) -> AppResult<()> {
        self.store.write(|c| sessions::set_note(c, id, note))?;
        self.data_changed();
        Ok(())
    }

    pub fn session_delete(&self, id: i64) -> AppResult<()> {
        self.store.write(|c| sessions::delete(c, id))?;
        self.data_changed();
        Ok(())
    }
}

fn ensure_free(
    c: &rusqlite::Connection,
    started: i64,
    ended: i64,
    except: i64,
    now: i64,
    active: Option<(i64, i64)>,
) -> AppResult<()> {
    check_span(started, ended, now)?;
    if let Some((a, b)) = active {
        if overlaps(started, ended, a, b) {
            return invalid("和正在进行的专注重叠了");
        }
    }
    if let Some(other) = sessions::first_overlap(c, started, ended, except)? {
        return invalid(conflict_text(c, &other)?);
    }
    Ok(())
}

fn conflict_text(c: &rusqlite::Connection, other: &Session) -> AppResult<String> {
    let name = subjects::get(c, other.subject_id)
        .map(|s| s.name)
        .unwrap_or_default();
    Ok(format!(
        "和 {}–{} 的{}重叠了",
        clock_hm(other.started_at),
        clock_hm(other.ended_at),
        name
    ))
}

fn clock_hm(ts: i64) -> String {
    Local
        .timestamp_millis_opt(ts)
        .single()
        .map(|d| d.format("%H:%M").to_string())
        .unwrap_or_else(|| "--:--".into())
}
