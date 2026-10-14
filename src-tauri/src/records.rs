//! 记录页：某一天的记录，以及修改、删除一条记录。

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::app::Aside;
use crate::error::{AppResult, invalid};
use crate::stats::{clipped, day_start_of};
use crate::store::sessions::{self, Session};
use crate::store::subjects;

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

/// 只改给出的字段。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SessionPatch {
    pub subject_id: Option<i64>,
    pub task: Option<String>,
    pub note: Option<String>,
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
                        day_ms: session.segments.iter().map(|g| clipped(g, start, end)).sum(),
                        session,
                    })
                })
                .collect::<AppResult<Vec<_>>>()
        })?;
        let total_ms = entries.iter().map(|e| e.day_ms).sum();
        Ok(DayView { date, start, end, total_ms, entries })
    }

    pub fn session_update(&self, id: i64, patch: SessionPatch) -> AppResult<Session> {
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
            sessions::update_fields(c, &s)?;
            Ok(s)
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
