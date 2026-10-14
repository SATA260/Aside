//! 记录页的统计：从数据库取出运行段，交给 `stats` 里的纯函数汇总。

use chrono::{Days, NaiveDate};
use serde::Serialize;

use super::{
    DayTotal, SubjectMs, balance, daily, day_start_of, days_studied, due_week_prompt, heatmap_from, local_date, month_of,
    sum_subjects, week_of,
};
use crate::app::Aside;
use crate::error::{AppResult, invalid};
use crate::store::sessions;

#[derive(Debug, Clone, Serialize)]
pub struct Period {
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub total_ms: i64,
    pub days_studied: usize,
    pub subjects: Vec<SubjectMs>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WeekReport {
    pub week: Period,
    pub days: Vec<DayTotal>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WeekPrompt {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

#[derive(Debug, Clone, Serialize)]
pub struct Overview {
    pub today: NaiveDate,
    /// 热力图第一天到今天，每天一项
    pub days: Vec<DayTotal>,
    pub week: Period,
    pub month: Period,
    /// 这一周的科目平衡
    pub balance: String,
}

/// 日历一次最多查这么多天。
const MAX_RANGE_DAYS: i64 = 400;

impl Aside {
    pub fn stats_overview(&self) -> AppResult<Overview> {
        let today = local_date(self.clock.now_ms());
        let days = self.daily(heatmap_from(today), today)?;
        let (week, month) = (period(&days, week_of(today)), period(&days, month_of(today)));
        let subjects = self.subjects_list(true)?;
        let named: Vec<(&str, i64)> = subjects
            .iter()
            .map(|s| {
                let ms = week.subjects.iter().find(|w| w.subject_id == s.id).map_or(0, |w| w.ms);
                (s.name.as_str(), ms)
            })
            .collect();
        Ok(Overview { today, days, balance: balance(&named), week, month })
    }

    /// 日历用：`from` 到 `to`（都含）每天学了多久。
    pub fn stats_days(&self, from: NaiveDate, to: NaiveDate) -> AppResult<Vec<DayTotal>> {
        if to < from || (to - from).num_days() >= MAX_RANGE_DAYS {
            return invalid("日期范围不对");
        }
        self.daily(from, to)
    }

    /// 某一周的周报。传入这一周里的任意一天，都会归到周一。
    pub fn stats_week(&self, date: NaiveDate) -> AppResult<WeekReport> {
        let range = week_of(date);
        let days = self.daily(range.0, range.1)?;
        Ok(WeekReport { week: period(&days, range), days })
    }

    /// 周一、上周有记录、这周还没提示过，就返回上周的范围，并记下来这周已经提示过。
    pub fn week_prompt(&self) -> AppResult<Option<WeekPrompt>> {
        let today = local_date(self.clock.now_ms());
        let monday = week_of(today).0;
        let last_monday = monday - Days::new(7);
        let last = (last_monday, last_monday + Days::new(6));
        let days = self.daily(last.0, last.1)?;
        let ms = days.iter().map(|d| d.total_ms).sum();
        if !due_week_prompt(today, &self.settings().week_prompted, ms) {
            return Ok(None);
        }
        self.edit_settings(|s| s.week_prompted = monday.to_string())?;
        Ok(Some(WeekPrompt { from: last.0, to: last.1 }))
    }

    fn daily(&self, from: NaiveDate, to: NaiveDate) -> AppResult<Vec<DayTotal>> {
        let end = day_start_of(to.succ_opt().unwrap_or(to));
        let pieces = self.store.read(|c| sessions::pieces_between(c, day_start_of(from), end))?;
        Ok(daily(&pieces, from, to))
    }
}

fn period(days: &[DayTotal], (from, to): (NaiveDate, NaiveDate)) -> Period {
    let inside: Vec<DayTotal> = days.iter().filter(|d| d.date >= from && d.date <= to).cloned().collect();
    Period {
        from,
        to,
        total_ms: inside.iter().map(|d| d.total_ms).sum(),
        days_studied: days_studied(&inside),
        subjects: sum_subjects(&inside),
    }
}
