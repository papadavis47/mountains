use crate::models::DailyLog;
use crate::period::Period;
use chrono::NaiveDate;

const ELEVATION_THRESHOLD: i32 = 1000;

pub fn count_monthly_1000_days(logs: &[DailyLog], reference_date: NaiveDate) -> usize {
    logs.iter()
        .filter(|log| {
            Period::Month.contains(log.date, reference_date)
                && log.elevation_gain.unwrap_or(0) >= ELEVATION_THRESHOLD
        })
        .count()
}

/// Sums `elevation_gain` over the logs falling in `period` (relative to
/// `reference_date`). Unlike miles this stays an integer with no rounding.
fn sum_elevation(logs: &[DailyLog], reference_date: NaiveDate, period: Period) -> i32 {
    logs.iter()
        .filter(|log| period.contains(log.date, reference_date))
        .filter_map(|log| log.elevation_gain)
        .sum()
}

pub fn calculate_weekly_elevation(logs: &[DailyLog], reference_date: NaiveDate) -> i32 {
    sum_elevation(logs, reference_date, Period::Week)
}

pub fn calculate_monthly_elevation(logs: &[DailyLog], reference_date: NaiveDate) -> i32 {
    sum_elevation(logs, reference_date, Period::Month)
}

pub fn calculate_yearly_elevation(logs: &[DailyLog], reference_date: NaiveDate) -> i32 {
    sum_elevation(logs, reference_date, Period::Year)
}

/// Returns streak count only if active (extends to most recent logged day)
pub fn calculate_current_streak(logs: &[DailyLog]) -> Option<usize> {
    if logs.is_empty() {
        return None;
    }

    let mut sorted_logs = logs.to_vec();
    // `Reverse` flips the ordering key, giving newest-first without a custom
    // closure — the idiomatic way to sort descending.
    sorted_logs.sort_by_key(|log| std::cmp::Reverse(log.date));

    let most_recent_date = sorted_logs.first()?.date;

    let most_recent_has_threshold =
        sorted_logs.first()?.elevation_gain.unwrap_or(0) >= ELEVATION_THRESHOLD;

    if !most_recent_has_threshold {
        return None;
    }

    let mut streak_count = 0;
    let mut current_date = most_recent_date;

    while let Some(log) = sorted_logs.iter().find(|log| log.date == current_date) {
        if log.elevation_gain.unwrap_or(0) >= ELEVATION_THRESHOLD {
            streak_count += 1;
            current_date = match current_date.pred_opt() {
                Some(date) => date,
                None => break,
            };
        } else {
            break;
        }
    }

    if streak_count >= 2 {
        Some(streak_count)
    } else {
        None
    }
}

pub fn get_streak_message(logs: &[DailyLog]) -> String {
    if let Some(streak_count) = calculate_current_streak(logs) {
        format!(
            "You currently have {} consecutive days of 1000+ feet of vert!",
            streak_count
        )
    } else {
        "Consider starting a streak - 1000+ daily feet of gain".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn log(date: NaiveDate, elevation: Option<i32>) -> DailyLog {
        DailyLog {
            date,
            elevation_gain: elevation,
            ..DailyLog::new(date)
        }
    }

    #[test]
    fn count_monthly_1000_days_matches_month_year_and_threshold() {
        let reference = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let logs = vec![
            log(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), Some(1200)),
            log(NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(), Some(800)),
            log(NaiveDate::from_ymd_opt(2026, 1, 3).unwrap(), Some(1500)),
            log(NaiveDate::from_ymd_opt(2025, 1, 3).unwrap(), Some(2000)),
            log(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(), Some(2000)),
        ];

        assert_eq!(count_monthly_1000_days(&logs, reference), 2);
    }

    #[test]
    fn elevation_totals_match_week_month_and_year() {
        let reference = NaiveDate::from_ymd_opt(2026, 7, 22).unwrap();
        let logs = vec![
            log(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), Some(400)),
            log(NaiveDate::from_ymd_opt(2026, 7, 19).unwrap(), Some(800)),
            log(NaiveDate::from_ymd_opt(2026, 7, 20).unwrap(), Some(1200)),
            log(NaiveDate::from_ymd_opt(2026, 7, 26).unwrap(), Some(1500)),
            log(NaiveDate::from_ymd_opt(2026, 7, 27).unwrap(), None),
            log(NaiveDate::from_ymd_opt(2025, 7, 22).unwrap(), Some(5000)),
        ];

        assert_eq!(calculate_weekly_elevation(&logs, reference), 2700);
        assert_eq!(calculate_monthly_elevation(&logs, reference), 3500);
        assert_eq!(calculate_yearly_elevation(&logs, reference), 3900);
    }

    #[test]
    fn calculate_weekly_elevation_handles_iso_week_across_calendar_years() {
        let reference = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let logs = vec![
            log(NaiveDate::from_ymd_opt(2025, 12, 28).unwrap(), Some(5000)),
            log(NaiveDate::from_ymd_opt(2025, 12, 29).unwrap(), Some(1200)),
            log(NaiveDate::from_ymd_opt(2026, 1, 4).unwrap(), Some(1500)),
            log(NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(), Some(6000)),
        ];

        assert_eq!(calculate_weekly_elevation(&logs, reference), 2700);
    }

    #[test]
    fn test_calculate_current_streak() {
        let today = NaiveDate::from_ymd_opt(2026, 7, 22).unwrap();
        let yesterday = today.pred_opt().unwrap();
        let two_days_ago = yesterday.pred_opt().unwrap();

        let logs = vec![
            DailyLog {
                date: today,
                elevation_gain: Some(1200),
                ..DailyLog::new(today)
            },
            DailyLog {
                date: yesterday,
                elevation_gain: Some(1500),
                ..DailyLog::new(yesterday)
            },
            DailyLog {
                date: two_days_ago,
                elevation_gain: Some(1100),
                ..DailyLog::new(two_days_ago)
            },
        ];

        assert_eq!(calculate_current_streak(&logs), Some(3));
    }
}
