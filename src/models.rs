use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub mod field_accessor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyLog {
    pub date: NaiveDate,
    pub food_entries: Vec<FoodEntry>,
    pub weight: Option<f32>,
    pub waist: Option<f32>,
    pub miles_covered: Option<f32>,
    pub elevation_gain: Option<i32>,
    pub sokay_entries: Vec<String>,
    pub strength_mobility: Option<String>,
    pub notes: Option<String>,
}

impl DailyLog {
    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            food_entries: Vec::new(),
            weight: None,
            waist: None,
            miles_covered: None,
            elevation_gain: None,
            sokay_entries: Vec::new(),
            strength_mobility: None,
            notes: None,
        }
    }

    pub fn add_food_entry(&mut self, entry: FoodEntry) {
        self.food_entries.push(entry);
    }

    pub fn remove_food_entry(&mut self, index: usize) {
        if index < self.food_entries.len() {
            self.food_entries.remove(index);
        }
    }

    pub fn add_sokay_entry(&mut self, entry: String) {
        self.sokay_entries.push(entry);
    }

    pub fn remove_sokay_entry(&mut self, index: usize) {
        if index < self.sokay_entries.len() {
            self.sokay_entries.remove(index);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodEntry {
    pub name: String,
}

impl FoodEntry {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MeasurementField {
    Weight,
    Waist,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RunningField {
    Miles,
    Elevation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FocusedSection {
    Measurements { focused_field: MeasurementField },
    Running { focused_field: RunningField },
    FoodItems,
    Sokay,
    StrengthMobility,
    Notes,
}

/// How the editor currently on screen was opened. A direct shortcut (or a
/// click) targets one field, so saving leaves focus there; arriving by Enter
/// after Shift+J/K is a top-to-bottom entry pass, which advances to the next
/// field.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditOrigin {
    Shortcut,
    Navigation,
}

/// Target for delete confirmation dialogs
#[derive(Debug, Clone, Copy)]
pub enum DeleteTarget {
    Day,
    Food(usize),
    Sokay(usize),
    /// Everything in `AppState.marks`
    Marked,
}

/// A DailyView list whose rows can be marked
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowList {
    Food,
    Sokay,
}

/// Rows marked (Ctrl+click / `x`) for one-confirmation batch delete. Marks
/// belong to one list at a time; marking in another list starts over. Food
/// and Sokay marks are row indices tied to a day; log marks are dates, so a
/// background reload reordering `daily_logs` can't shift them.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Marks {
    #[default]
    None,
    Rows {
        list: RowList,
        date: NaiveDate,
        rows: BTreeSet<usize>,
    },
    Logs(BTreeSet<NaiveDate>),
}

impl Marks {
    pub fn clear(&mut self) {
        *self = Marks::None;
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Marks::None)
    }

    pub fn count(&self) -> usize {
        match self {
            Marks::None => 0,
            Marks::Rows { rows, .. } => rows.len(),
            Marks::Logs(dates) => dates.len(),
        }
    }

    /// Toggles a Food/Sokay row on `date`
    pub fn toggle_row(&mut self, list: RowList, date: NaiveDate, row: usize) {
        match self {
            Marks::Rows {
                list: l,
                date: d,
                rows,
            } if *l == list && *d == date => {
                if !rows.remove(&row) {
                    rows.insert(row);
                }
                if rows.is_empty() {
                    self.clear();
                }
            }
            _ => {
                *self = Marks::Rows {
                    list,
                    date,
                    rows: BTreeSet::from([row]),
                };
            }
        }
    }

    /// Toggles a whole day in the log list
    pub fn toggle_log(&mut self, date: NaiveDate) {
        match self {
            Marks::Logs(dates) => {
                if !dates.remove(&date) {
                    dates.insert(date);
                }
                if dates.is_empty() {
                    self.clear();
                }
            }
            _ => *self = Marks::Logs(BTreeSet::from([date])),
        }
    }

    /// Marked rows of `list` on `date`, if any
    pub fn rows(&self, list: RowList, date: NaiveDate) -> Option<&BTreeSet<usize>> {
        match self {
            Marks::Rows {
                list: l,
                date: d,
                rows,
            } if *l == list && *d == date => Some(rows),
            _ => None,
        }
    }

    pub fn is_row_marked(&self, list: RowList, date: NaiveDate, row: usize) -> bool {
        self.rows(list, date)
            .is_some_and(|rows| rows.contains(&row))
    }

    pub fn logs(&self) -> Option<&BTreeSet<NaiveDate>> {
        match self {
            Marks::Logs(dates) => Some(dates),
            _ => None,
        }
    }

    pub fn is_log_marked(&self, date: NaiveDate) -> bool {
        self.logs().is_some_and(|dates| dates.contains(&date))
    }
}

#[derive(Debug, Clone)]
pub enum AppScreen {
    Startup,
    Statistics,
    Home,
    DailyView,
    AddFood,
    EditFood(usize),
    AddSokay,
    EditSokay(usize),
    InputField(field_accessor::FieldType),
    ConfirmDelete(DeleteTarget),
    ShortcutsHelp,
    DateInput,
    Syncing,
    ConfigSync,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigSyncField {
    DbUrl,
    AuthToken,
    EnableToggle,
}

/// Which screen the app opens on, chosen by CLI flags before the TUI starts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LaunchMode {
    Startup,
    Today,
    Simple,
    Notes,
    Food,
    Exercise,
}

/// Which DailyView layout is active. `Full` is the normal full-screen view;
/// the others (`-s`, `-n`, `-f`, `-e` flags) show a subset of sections in a centered
/// window. Reset to `Full` when leaving DailyView for Home.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Full,
    /// `-s`: Running, Food, and Notes.
    Simple,
    /// `-n`: Notes only.
    Notes,
    /// `-f`: Food and Sokay.
    Food,
    /// `-e`: Strength & Mobility only.
    Exercise,
}

#[derive(Debug)]
pub struct AppState {
    pub current_screen: AppScreen,
    pub selected_date: NaiveDate,
    pub daily_logs: Vec<DailyLog>,
    pub focused_section: FocusedSection,
    pub edit_origin: EditOrigin,
    pub food_list_focused: bool,
    pub sokay_list_focused: bool,
    pub strength_mobility_scroll: u16,
    pub notes_scroll: u16,
    pub date_input_error: Option<String>,
    pub config_sync_focused_field: ConfigSyncField,
    pub config_sync_status: Option<String>,
    /// DailyView layout chosen by CLI flag (`-s`, `-n`, `-f`); `Full` otherwise.
    pub view_mode: ViewMode,
    /// Last rendered frame size, used to bound multi-line section scrolling.
    pub frame_width: u16,
    pub frame_height: u16,
    /// Rows marked for batch delete
    pub marks: Marks,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_screen: AppScreen::Startup,
            selected_date: chrono::Local::now().date_naive(),
            daily_logs: Vec::new(),
            focused_section: FocusedSection::Measurements {
                focused_field: MeasurementField::Weight,
            },
            edit_origin: EditOrigin::Navigation,
            food_list_focused: false,
            sokay_list_focused: false,
            strength_mobility_scroll: 0,
            notes_scroll: 0,
            date_input_error: None,
            config_sync_focused_field: ConfigSyncField::DbUrl,
            config_sync_status: None,
            view_mode: ViewMode::Full,
            frame_width: 0,
            frame_height: 0,
            marks: Marks::None,
        }
    }

    pub fn get_or_create_daily_log(&mut self, date: NaiveDate) -> &mut DailyLog {
        if let Some(pos) = self.daily_logs.iter().position(|log| log.date == date) {
            &mut self.daily_logs[pos]
        } else {
            self.daily_logs.push(DailyLog::new(date));
            self.daily_logs
                .sort_by_key(|log| std::cmp::Reverse(log.date));
            self.daily_logs
                .iter_mut()
                .find(|log| log.date == date)
                .unwrap()
        }
    }

    pub fn get_daily_log(&self, date: NaiveDate) -> Option<&DailyLog> {
        self.daily_logs.iter().find(|log| log.date == date)
    }

    /// Returns to the Home screen from DailyView. Leaving DailyView ends any
    /// compact view mode (`-s`, `-n`, `-f`, `-e`): the rest of the session behaves
    /// like the full app.
    pub fn go_home_from_daily_view(&mut self) {
        self.view_mode = ViewMode::Full;
        self.current_screen = AppScreen::Home;
    }

    /// Applies a CLI-selected launch mode: every mode but `Startup` jumps
    /// straight into today's DailyView (creating the log if needed), and the
    /// compact modes also set their view mode and starting focus.
    pub fn apply_launch_mode(&mut self, mode: LaunchMode) {
        if mode == LaunchMode::Startup {
            return;
        }
        self.selected_date = chrono::Local::now().date_naive();
        self.get_or_create_daily_log(self.selected_date);
        self.current_screen = AppScreen::DailyView;
        match mode {
            LaunchMode::Simple => {
                self.view_mode = ViewMode::Simple;
                self.focused_section = FocusedSection::Running {
                    focused_field: RunningField::Miles,
                };
            }
            LaunchMode::Notes => {
                self.view_mode = ViewMode::Notes;
                self.focused_section = FocusedSection::Notes;
            }
            LaunchMode::Food => {
                self.view_mode = ViewMode::Food;
                self.focused_section = FocusedSection::FoodItems;
            }
            LaunchMode::Exercise => {
                self.view_mode = ViewMode::Exercise;
                self.focused_section = FocusedSection::StrengthMobility;
            }
            LaunchMode::Startup | LaunchMode::Today => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_mode_startup_leaves_state_untouched() {
        let mut state = AppState::new();
        state.apply_launch_mode(LaunchMode::Startup);
        assert!(matches!(state.current_screen, AppScreen::Startup));
        assert!(state.daily_logs.is_empty());
        assert_eq!(state.view_mode, ViewMode::Full);
    }

    #[test]
    fn launch_mode_today_opens_daily_view_for_today() {
        let mut state = AppState::new();
        state.apply_launch_mode(LaunchMode::Today);
        let today = chrono::Local::now().date_naive();
        assert!(matches!(state.current_screen, AppScreen::DailyView));
        assert_eq!(state.selected_date, today);
        assert!(state.get_daily_log(today).is_some());
        assert_eq!(state.view_mode, ViewMode::Full);
    }

    #[test]
    fn launch_mode_today_reuses_existing_log() {
        let mut state = AppState::new();
        let today = chrono::Local::now().date_naive();
        state.get_or_create_daily_log(today).weight = Some(180.0);
        state.apply_launch_mode(LaunchMode::Today);
        assert_eq!(state.daily_logs.len(), 1);
        assert_eq!(state.get_daily_log(today).unwrap().weight, Some(180.0));
    }

    #[test]
    fn going_home_from_daily_view_clears_compact_view_modes() {
        for mode in [
            LaunchMode::Simple,
            LaunchMode::Notes,
            LaunchMode::Food,
            LaunchMode::Exercise,
        ] {
            let mut state = AppState::new();
            state.apply_launch_mode(mode);
            state.go_home_from_daily_view();
            assert!(matches!(state.current_screen, AppScreen::Home));
            assert_eq!(state.view_mode, ViewMode::Full);
        }
    }

    #[test]
    fn launch_mode_simple_opens_daily_view_in_simple_mode() {
        let mut state = AppState::new();
        state.apply_launch_mode(LaunchMode::Simple);
        let today = chrono::Local::now().date_naive();
        assert!(matches!(state.current_screen, AppScreen::DailyView));
        assert_eq!(state.selected_date, today);
        assert!(state.get_daily_log(today).is_some());
        assert_eq!(state.view_mode, ViewMode::Simple);
        assert_eq!(
            state.focused_section,
            FocusedSection::Running {
                focused_field: RunningField::Miles
            }
        );
    }

    #[test]
    fn launch_mode_notes_opens_daily_view_focused_on_notes() {
        let mut state = AppState::new();
        state.apply_launch_mode(LaunchMode::Notes);
        let today = chrono::Local::now().date_naive();
        assert!(matches!(state.current_screen, AppScreen::DailyView));
        assert!(state.get_daily_log(today).is_some());
        assert_eq!(state.view_mode, ViewMode::Notes);
        assert_eq!(state.focused_section, FocusedSection::Notes);
    }

    #[test]
    fn launch_mode_food_opens_daily_view_focused_on_food() {
        let mut state = AppState::new();
        state.apply_launch_mode(LaunchMode::Food);
        let today = chrono::Local::now().date_naive();
        assert!(matches!(state.current_screen, AppScreen::DailyView));
        assert!(state.get_daily_log(today).is_some());
        assert_eq!(state.view_mode, ViewMode::Food);
        assert_eq!(state.focused_section, FocusedSection::FoodItems);
    }

    #[test]
    fn launch_mode_exercise_opens_daily_view_focused_on_strength_mobility() {
        let mut state = AppState::new();
        state.apply_launch_mode(LaunchMode::Exercise);
        let today = chrono::Local::now().date_naive();
        assert!(matches!(state.current_screen, AppScreen::DailyView));
        assert!(state.get_daily_log(today).is_some());
        assert_eq!(state.view_mode, ViewMode::Exercise);
        assert_eq!(state.focused_section, FocusedSection::StrengthMobility);
    }

    // -- Marks ---------------------------------------------------------------

    fn day(d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, d).unwrap()
    }

    #[test]
    fn marks_toggle_row_on_and_off() {
        let mut m = Marks::None;
        m.toggle_row(RowList::Food, day(1), 2);
        m.toggle_row(RowList::Food, day(1), 0);
        assert_eq!(m.count(), 2);
        assert!(m.is_row_marked(RowList::Food, day(1), 2));
        m.toggle_row(RowList::Food, day(1), 2);
        assert!(!m.is_row_marked(RowList::Food, day(1), 2));
        m.toggle_row(RowList::Food, day(1), 0);
        assert!(m.is_empty());
    }

    #[test]
    fn marks_switching_list_or_day_starts_over() {
        let mut m = Marks::None;
        m.toggle_row(RowList::Food, day(1), 0);
        m.toggle_row(RowList::Sokay, day(1), 1);
        assert_eq!(m.count(), 1);
        assert!(m.rows(RowList::Food, day(1)).is_none());
        m.toggle_row(RowList::Sokay, day(2), 3);
        assert!(m.rows(RowList::Sokay, day(1)).is_none());
        m.toggle_log(day(5));
        assert!(m.rows(RowList::Sokay, day(2)).is_none());
        assert!(m.is_log_marked(day(5)));
    }

    #[test]
    fn marks_toggle_log_dates() {
        let mut m = Marks::None;
        m.toggle_log(day(1));
        m.toggle_log(day(3));
        assert_eq!(m.count(), 2);
        m.toggle_log(day(1));
        assert!(!m.is_log_marked(day(1)));
        m.toggle_log(day(3));
        assert!(m.is_empty());
    }
}
