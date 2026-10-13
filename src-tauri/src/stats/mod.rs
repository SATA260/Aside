//! 按天汇总：本地时间的一天从 0 点算起，跨零点的运行段在零点处拆开。

use chrono::{Local, NaiveDate, TimeZone};

use crate::timer::Segment;

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
