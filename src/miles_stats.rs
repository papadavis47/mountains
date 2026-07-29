use crate::models::DailyLog;
use crate::period::Period;
use chrono::NaiveDate;

/// Rounds to one decimal place, normalizing negative zero to positive zero.
/// An empty `f32` sum yields `-0.0` (std's additive identity), which would
/// otherwise display as "-0.0" when no miles are logged for the period.
fn round_tenths(total: f32) -> f32 {
    let rounded = (total * 10.0).round() / 10.0;
    if rounded == 0.0 { 0.0 } else { rounded }
}

/// Totals `miles_covered` over the logs falling in `period` (relative to
/// `reference_date`). `filter_map` drops the `None` entries in one pass, so days
/// with no mileage don't need a separate guard.
fn sum_miles(logs: &[DailyLog], reference_date: NaiveDate, period: Period) -> f32 {
    let total: f32 = logs
        .iter()
        .filter(|log| period.contains(log.date, reference_date))
        .filter_map(|log| log.miles_covered)
        .sum();

    round_tenths(total)
}

pub fn calculate_weekly_miles(logs: &[DailyLog], reference_date: NaiveDate) -> f32 {
    sum_miles(logs, reference_date, Period::Week)
}

pub fn calculate_monthly_miles(logs: &[DailyLog], reference_date: NaiveDate) -> f32 {
    sum_miles(logs, reference_date, Period::Month)
}

pub fn calculate_yearly_miles(logs: &[DailyLog], reference_date: NaiveDate) -> f32 {
    sum_miles(logs, reference_date, Period::Year)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn log(date: NaiveDate, miles: Option<f32>) -> DailyLog {
        DailyLog {
            date,
            miles_covered: miles,
            ..DailyLog::new(date)
        }
    }

    #[test]
    fn calculate_weekly_miles_uses_iso_week_boundaries() {
        let reference = NaiveDate::from_ymd_opt(2026, 7, 22).unwrap();
        let logs = vec![
            log(NaiveDate::from_ymd_opt(2026, 7, 19).unwrap(), Some(20.0)),
            log(NaiveDate::from_ymd_opt(2026, 7, 20).unwrap(), Some(5.5)),
            log(NaiveDate::from_ymd_opt(2026, 7, 26).unwrap(), Some(3.2)),
            log(NaiveDate::from_ymd_opt(2026, 7, 27).unwrap(), Some(30.0)),
        ];

        assert_eq!(calculate_weekly_miles(&logs, reference), 8.7);
    }

    #[test]
    fn calculate_weekly_miles_handles_iso_week_across_calendar_years() {
        let reference = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let logs = vec![
            log(NaiveDate::from_ymd_opt(2025, 12, 28).unwrap(), Some(20.0)),
            log(NaiveDate::from_ymd_opt(2025, 12, 29).unwrap(), Some(5.0)),
            log(NaiveDate::from_ymd_opt(2026, 1, 4).unwrap(), Some(7.0)),
            log(NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(), Some(30.0)),
        ];

        assert_eq!(calculate_weekly_miles(&logs, reference), 12.0);
    }

    #[test]
    fn calculate_monthly_miles_matches_month_and_year() {
        let reference = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let logs = vec![
            log(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(), Some(20.0)),
            log(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), Some(5.5)),
            log(NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(), Some(3.2)),
            log(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(), Some(30.0)),
        ];

        assert_eq!(calculate_monthly_miles(&logs, reference), 8.7);
    }

    #[test]
    fn calculate_yearly_miles_matches_year_and_skips_none() {
        let reference = NaiveDate::from_ymd_opt(2026, 7, 22).unwrap();
        let logs = vec![
            log(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None),
            log(NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(), Some(5.5)),
            log(NaiveDate::from_ymd_opt(2026, 7, 22).unwrap(), Some(3.2)),
            log(NaiveDate::from_ymd_opt(2025, 7, 22).unwrap(), Some(30.0)),
        ];

        assert_eq!(calculate_yearly_miles(&logs, reference), 8.7);
    }

    #[test]
    fn mileage_totals_round_to_tenths() {
        let reference = NaiveDate::from_ymd_opt(2026, 7, 22).unwrap();
        let logs = vec![
            log(NaiveDate::from_ymd_opt(2026, 7, 21).unwrap(), Some(7.64)),
            log(NaiveDate::from_ymd_opt(2026, 7, 22).unwrap(), Some(30.476)),
        ];

        assert_eq!(calculate_weekly_miles(&logs, reference), 38.1);
        assert_eq!(calculate_monthly_miles(&logs, reference), 38.1);
        assert_eq!(calculate_yearly_miles(&logs, reference), 38.1);
    }

    #[test]
    fn empty_mileage_totals_are_positive_zero() {
        let reference = NaiveDate::from_ymd_opt(2026, 7, 22).unwrap();
        for result in [
            calculate_weekly_miles(&[], reference),
            calculate_monthly_miles(&[], reference),
            calculate_yearly_miles(&[], reference),
        ] {
            assert_eq!(result, 0.0);
            assert!(!result.is_sign_negative());
            assert_eq!(format!("{result:.1}"), "0.0");
        }
    }
}
