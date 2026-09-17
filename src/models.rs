use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

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
    /// Simple entry mode (`-s` flag): DailyView shows only Running, Food, and
    /// Notes in a centered window. Cleared when leaving DailyView for Home.
    pub simple_mode: bool,
    /// Last rendered frame size, used to bound multi-line section scrolling.
    pub frame_width: u16,
    pub frame_height: u16,
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
            simple_mode: false,
            frame_width: 0,
            frame_height: 0,
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

    /// Returns to the Home screen from DailyView. Leaving DailyView ends
    /// simple mode (`-s`): the rest of the session behaves like the full app.
    pub fn go_home_from_daily_view(&mut self) {
        self.simple_mode = false;
        self.current_screen = AppScreen::Home;
    }

    /// Applies a CLI-selected launch mode: `Today`/`Simple` jump straight into
    /// today's DailyView (creating the log if needed), bypassing Startup.
    pub fn apply_launch_mode(&mut self, mode: LaunchMode) {
        if mode == LaunchMode::Startup {
            return;
        }
        self.selected_date = chrono::Local::now().date_naive();
        self.get_or_create_daily_log(self.selected_date);
        self.current_screen = AppScreen::DailyView;
        if mode == LaunchMode::Simple {
            self.simple_mode = true;
            self.focused_section = FocusedSection::Running {
                focused_field: RunningField::Miles,
            };
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
        assert!(!state.simple_mode);
    }

    #[test]
    fn launch_mode_today_opens_daily_view_for_today() {
        let mut state = AppState::new();
        state.apply_launch_mode(LaunchMode::Today);
        let today = chrono::Local::now().date_naive();
        assert!(matches!(state.current_screen, AppScreen::DailyView));
        assert_eq!(state.selected_date, today);
        assert!(state.get_daily_log(today).is_some());
        assert!(!state.simple_mode);
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
    fn going_home_from_daily_view_clears_simple_mode() {
        let mut state = AppState::new();
        state.apply_launch_mode(LaunchMode::Simple);
        state.go_home_from_daily_view();
        assert!(matches!(state.current_screen, AppScreen::Home));
        assert!(!state.simple_mode);
    }

    #[test]
    fn launch_mode_simple_opens_daily_view_in_simple_mode() {
        let mut state = AppState::new();
        state.apply_launch_mode(LaunchMode::Simple);
        let today = chrono::Local::now().date_naive();
        assert!(matches!(state.current_screen, AppScreen::DailyView));
        assert_eq!(state.selected_date, today);
        assert!(state.get_daily_log(today).is_some());
        assert!(state.simple_mode);
        assert_eq!(
            state.focused_section,
            FocusedSection::Running {
                focused_field: RunningField::Miles
            }
        );
    }
}
