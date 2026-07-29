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

#[derive(Debug)]
pub struct AppState {
    pub current_screen: AppScreen,
    pub selected_date: NaiveDate,
    pub daily_logs: Vec<DailyLog>,
    pub focused_section: FocusedSection,
    pub food_list_focused: bool,
    pub sokay_list_focused: bool,
    pub strength_mobility_scroll: u16,
    pub notes_scroll: u16,
    pub date_input_error: Option<String>,
    pub config_sync_focused_field: ConfigSyncField,
    pub config_sync_status: Option<String>,
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
            food_list_focused: false,
            sokay_list_focused: false,
            strength_mobility_scroll: 0,
            notes_scroll: 0,
            date_input_error: None,
            config_sync_focused_field: ConfigSyncField::DbUrl,
            config_sync_status: None,
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
}
