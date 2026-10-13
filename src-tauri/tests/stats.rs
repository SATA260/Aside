use aside_lib::stats::{self, Piece, SubjectMs};
use chrono::{Local, NaiveDate, TimeZone};

const MIN: i64 = 60_000;

fn at(d: u32, h: u32, m: u32) -> i64 {
    Local.with_ymd_and_hms(2026, 10, d, h, m, 0).unwrap().timestamp_millis()
}

fn date(m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, m, d).unwrap()
}

fn piece(subject_id: i64, start: i64, end: i64) -> Piece {
    Piece { subject_id, start, end }
}

#[test]
fn a_piece_across_midnight_is_split_between_the_two_days() {
    let days = stats::daily(&[piece(1, at(12, 23, 30), at(13, 0, 45))], date(10, 12), date(10, 13));
    assert_eq!(days.len(), 2);
    assert_eq!((days[0].date, days[0].total_ms), (date(10, 12), 30 * MIN));
    assert_eq!((days[1].date, days[1].total_ms), (date(10, 13), 45 * MIN));
    assert_eq!(days[1].start, at(13, 0, 0));
}

#[test]
fn every_day_in_the_range_is_listed_even_without_time() {
    let days = stats::daily(&[piece(1, at(14, 9, 0), at(14, 10, 0))], date(10, 12), date(10, 14));
    let totals: Vec<_> = days.iter().map(|d| (d.date, d.total_ms)).collect();
    assert_eq!(totals, vec![(date(10, 12), 0), (date(10, 13), 0), (date(10, 14), 60 * MIN)]);
    assert!(days[0].subjects.is_empty());
}

#[test]
fn pieces_outside_the_range_are_ignored() {
    let pieces = [piece(1, at(10, 9, 0), at(10, 10, 0)), piece(1, at(16, 9, 0), at(16, 10, 0))];
    let days = stats::daily(&pieces, date(10, 12), date(10, 14));
    assert!(days.iter().all(|d| d.total_ms == 0));
}

#[test]
fn a_day_sums_each_subject_with_the_largest_first() {
    let pieces = [
        piece(1, at(13, 8, 0), at(13, 8, 20)),
        piece(2, at(13, 9, 0), at(13, 10, 0)),
        piece(1, at(13, 14, 0), at(13, 14, 30)),
        piece(3, at(13, 15, 0), at(13, 15, 50)),
    ];
    let day = &stats::daily(&pieces, date(10, 13), date(10, 13))[0];
    assert_eq!(day.total_ms, 160 * MIN);
    assert_eq!(
        day.subjects,
        vec![
            SubjectMs { subject_id: 2, ms: 60 * MIN },
            SubjectMs { subject_id: 1, ms: 50 * MIN },
            SubjectMs { subject_id: 3, ms: 50 * MIN },
        ]
    );
}

#[test]
fn subject_sums_over_several_days_add_up() {
    let pieces = [
        piece(1, at(12, 8, 0), at(12, 9, 0)),
        piece(2, at(13, 8, 0), at(13, 8, 30)),
        piece(1, at(14, 8, 0), at(14, 8, 30)),
    ];
    let days = stats::daily(&pieces, date(10, 12), date(10, 14));
    assert_eq!(
        stats::sum_subjects(&days),
        vec![SubjectMs { subject_id: 1, ms: 90 * MIN }, SubjectMs { subject_id: 2, ms: 30 * MIN }]
    );
}

#[test]
fn days_studied_only_counts_days_with_some_time() {
    let pieces = [piece(1, at(12, 8, 0), at(12, 8, 1)), piece(1, at(14, 8, 0), at(14, 9, 0))];
    let days = stats::daily(&pieces, date(10, 12), date(10, 18));
    assert_eq!(stats::days_studied(&days), 2);
}

#[test]
fn a_week_runs_from_monday_to_sunday() {
    assert_eq!(stats::week_of(date(10, 14)), (date(10, 12), date(10, 18)));
    assert_eq!(stats::week_of(date(10, 12)), (date(10, 12), date(10, 18)));
    assert_eq!(stats::week_of(date(10, 18)), (date(10, 12), date(10, 18)));
}

#[test]
fn a_month_runs_from_the_first_to_the_last_day() {
    assert_eq!(stats::month_of(date(10, 14)), (date(10, 1), date(10, 31)));
    assert_eq!(stats::month_of(date(2, 3)), (date(2, 1), date(2, 28)));
    assert_eq!(stats::month_of(date(12, 31)), (date(12, 1), date(12, 31)));
}

#[test]
fn the_heatmap_covers_sixteen_whole_weeks_up_to_this_one() {
    let from = stats::heatmap_from(date(10, 14));
    assert_eq!(from, date(6, 29));
    assert_eq!((stats::week_of(date(10, 14)).1 - from).num_days() + 1, 16 * 7);
}

#[test]
fn balance_says_nothing_happened_when_the_week_is_empty() {
    assert_eq!(stats::balance(&[]), "这一周还没有记录。");
    assert_eq!(stats::balance(&[("数学", 0), ("英语", 0)]), "这一周还没有记录。");
}

#[test]
fn balance_with_one_subject_says_where_the_time_went() {
    assert_eq!(stats::balance(&[("政治", 0), ("数学", 90 * MIN)]), "这一周的时间都给了数学。");
}

#[test]
fn balance_names_the_most_and_the_least_among_subjects_with_time() {
    let week = [("政治", 0), ("英语", 40 * MIN), ("数学", 300 * MIN), ("专业课", 120 * MIN)];
    assert_eq!(stats::balance(&week), "数学花的时间最多，英语这周少一些。");
}

#[test]
fn balance_calls_close_totals_even() {
    let week = [("英语", 100 * MIN), ("数学", 120 * MIN), ("专业课", 90 * MIN)];
    assert_eq!(stats::balance(&week), "各科时间差不多，数学稍多一些。");
    assert_eq!(stats::balance(&[("英语", 60 * MIN), ("数学", 60 * MIN)]), "各科时间差不多。");
}

#[test]
fn balance_breaks_ties_in_the_given_order() {
    let week = [("英语", 30 * MIN), ("数学", 200 * MIN), ("政治", 30 * MIN), ("专业课", 200 * MIN)];
    assert_eq!(stats::balance(&week), "数学花的时间最多，英语这周少一些。");
}
