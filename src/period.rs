use chrono::{Datelike, NaiveDate};

/// A reporting period, evaluated relative to a reference date. This is the single
/// definition of what "this week / month / year" contains, so every statistic
/// agrees on period boundaries — change the rule here and all callers follow.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Period {
    Week,
    Month,
    Year,
}

impl Period {
    /// Whether `date` falls in this period relative to `reference`.
    ///
    /// Weeks use ISO-8601 week-years, which are Monday-based and whose week-year
    /// can differ from the calendar year around New Year (e.g. Dec 29 2025 and
    /// Jan 1 2026 share an ISO week). Comparing the whole `IsoWeek` — week number
    /// *and* week-year — handles that boundary correctly, where comparing month
    /// or calendar year would not.
    pub fn contains(self, date: NaiveDate, reference: NaiveDate) -> bool {
        match self {
            Period::Week => date.iso_week() == reference.iso_week(),
            Period::Month => date.year() == reference.year() && date.month() == reference.month(),
            Period::Year => date.year() == reference.year(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn week_groups_by_iso_week_across_calendar_years() {
        let reference = date(2026, 1, 1); // ISO week 1 of 2026
        assert!(Period::Week.contains(date(2025, 12, 29), reference)); // same ISO week
        assert!(Period::Week.contains(date(2026, 1, 4), reference)); // still that week
        assert!(!Period::Week.contains(date(2026, 1, 5), reference)); // next week
    }

    #[test]
    fn month_requires_matching_month_and_year() {
        let reference = date(2026, 1, 15);
        assert!(Period::Month.contains(date(2026, 1, 31), reference));
        assert!(!Period::Month.contains(date(2025, 1, 15), reference)); // same month, wrong year
        assert!(!Period::Month.contains(date(2026, 2, 1), reference)); // wrong month
    }

    #[test]
    fn year_matches_calendar_year_only() {
        let reference = date(2026, 7, 22);
        assert!(Period::Year.contains(date(2026, 1, 1), reference));
        assert!(!Period::Year.contains(date(2025, 12, 31), reference));
    }
}
