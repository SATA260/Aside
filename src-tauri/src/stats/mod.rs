//! 按天汇总：本地时间的一天从 0 点算起，跨零点的运行段在零点处拆开。

pub mod service;

use chrono::{Datelike, Days, Local, Months, NaiveDate, TimeZone, Weekday};
use serde::Serialize;

use crate::timer::Segment;

/// 一段真正在跑的时间，以及它属于哪个科目。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Piece {
    pub subject_id: i64,
    pub start: i64,
    pub end: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SubjectMs {
    pub subject_id: i64,
    pub ms: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DayTotal {
    pub date: NaiveDate,
    /// 这一天 0 点的时刻
    pub start: i64,
    pub total_ms: i64,
    pub subjects: Vec<SubjectMs>,
}

/// 热力图显示最近几周，每周从周一开始。
pub const HEATMAP_WEEKS: u64 = 16;

/// 科目时长相差不到这个比例时，算作差不多。
const EVEN_RATIO: f64 = 0.7;

/// 从 `from` 到 `to`（都含）每一天的专注时长，一天也不漏。跨零点的运行段拆到两天。
pub fn daily(pieces: &[Piece], from: NaiveDate, to: NaiveDate) -> Vec<DayTotal> {
    from.iter_days()
        .take_while(|d| *d <= to)
        .map(|date| {
            let start = day_start_of(date);
            let end = day_start_of(date.succ_opt().unwrap_or(date));
            let mut subjects: Vec<SubjectMs> = Vec::new();
            for p in pieces {
                let ms = clipped(&Segment { start: p.start, end: p.end }, start, end);
                if ms == 0 {
                    continue;
                }
                match subjects.iter_mut().find(|s| s.subject_id == p.subject_id) {
                    Some(s) => s.ms += ms,
                    None => subjects.push(SubjectMs { subject_id: p.subject_id, ms }),
                }
            }
            sort_subjects(&mut subjects);
            DayTotal { date, start, total_ms: subjects.iter().map(|s| s.ms).sum(), subjects }
        })
        .collect()
}

/// 几天里每个科目的合计，时间多的在前。
pub fn sum_subjects(days: &[DayTotal]) -> Vec<SubjectMs> {
    let mut out: Vec<SubjectMs> = Vec::new();
    for s in days.iter().flat_map(|d| &d.subjects) {
        match out.iter_mut().find(|o| o.subject_id == s.subject_id) {
            Some(o) => o.ms += s.ms,
            None => out.push(s.clone()),
        }
    }
    sort_subjects(&mut out);
    out
}

fn sort_subjects(list: &mut [SubjectMs]) {
    list.sort_by(|a, b| b.ms.cmp(&a.ms).then(a.subject_id.cmp(&b.subject_id)));
}

/// 学了的天数：有专注时长的日子，不要求连续。
pub fn days_studied(days: &[DayTotal]) -> usize {
    days.iter().filter(|d| d.total_ms > 0).count()
}

/// `date` 所在这一周的周一和周日。
pub fn week_of(date: NaiveDate) -> (NaiveDate, NaiveDate) {
    let monday = date.week(Weekday::Mon).first_day();
    (monday, monday + Days::new(6))
}

/// `date` 所在这个月的第一天和最后一天。
pub fn month_of(date: NaiveDate) -> (NaiveDate, NaiveDate) {
    let first = date.with_day(1).unwrap_or(date);
    let last = (first + Months::new(1)).pred_opt().unwrap_or(first);
    (first, last)
}

/// 热力图第一列的周一：这一周往前数，一共 `HEATMAP_WEEKS` 整周。
pub fn heatmap_from(today: NaiveDate) -> NaiveDate {
    week_of(today).0 - Days::new((HEATMAP_WEEKS - 1) * 7)
}

/// 一句只陈述、不催促的科目平衡。`week` 按科目顺序排列，并列时取靠前的那个。
pub fn balance(week: &[(&str, i64)]) -> String {
    let studied: Vec<(&str, i64)> = week.iter().copied().filter(|(_, ms)| *ms > 0).collect();
    let Some(top) = (0..studied.len()).reduce(|a, b| if studied[b].1 > studied[a].1 { b } else { a }) else {
        return "这一周还没有记录。".into();
    };
    let low = (0..studied.len()).filter(|&i| i != top).reduce(|a, b| if studied[b].1 < studied[a].1 { b } else { a });
    let (most, least) = match low {
        Some(low) => (studied[top], studied[low]),
        None => return format!("这一周的时间都给了{}。", studied[top].0),
    };
    if (least.1 as f64) < most.1 as f64 * EVEN_RATIO {
        format!("{}花的时间最多，{}这周少一些。", most.0, least.0)
    } else if studied.iter().enumerate().any(|(i, s)| i != top && s.1 == most.1) {
        "各科时间差不多。".into()
    } else {
        format!("各科时间差不多，{}稍多一些。", most.0)
    }
}

/// 周一第一次打开、并且上周有记录时，提示一次。`prompted` 是已经提示过的那个周一。
pub fn due_week_prompt(today: NaiveDate, prompted: &str, last_week_ms: i64) -> bool {
    if today.weekday() != Weekday::Mon || last_week_ms <= 0 {
        return false;
    }
    prompted != week_of(today).0.to_string()
}

/// 本地日期 `date` 的 0 点（Unix 毫秒）。
pub fn day_start_of(date: NaiveDate) -> i64 {
    let midnight = date.and_hms_opt(0, 0, 0).expect("0 点总是合法的");
    Local
        .from_local_datetime(&midnight)
        .earliest()
        .unwrap_or_else(|| Local.from_utc_datetime(&midnight))
        .timestamp_millis()
}

pub fn local_date(ts: i64) -> NaiveDate {
    Local.timestamp_millis_opt(ts).single().map_or_else(
        || chrono::DateTime::from_timestamp_millis(ts).unwrap_or_default().date_naive(),
        |d| d.date_naive(),
    )
}

/// `ts` 所在这一天的 [0 点, 次日 0 点)。
pub fn day_bounds(ts: i64) -> (i64, i64) {
    let date = local_date(ts);
    let next = date.succ_opt().unwrap_or(date);
    (day_start_of(date), day_start_of(next))
}

/// 运行段落在 [from, to) 里的长度。
pub fn clipped(seg: &Segment, from: i64, to: i64) -> i64 {
    (seg.end.min(to) - seg.start.max(from)).max(0)
}
