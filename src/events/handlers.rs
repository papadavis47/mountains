use crate::db_manager::DbManager;
use crate::file_manager::FileManager;
use crate::models::{
    AppScreen, AppState, DailyLog, FocusedSection, FoodEntry, MeasurementField, RunningField,
    field_accessor::FieldType,
};
use crossterm::event::{KeyCode, KeyModifiers};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct InputHandler {
    pub input_buffer: String,
    pub cursor_position: usize,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            input_buffer: String::new(),
            cursor_position: 0,
        }
    }

    pub fn clear(&mut self) {
        self.input_buffer.clear();
        self.cursor_position = 0;
    }

    pub fn set_input(&mut self, text: String) {
        self.cursor_position = text.len();
        self.input_buffer = text;
    }

    pub fn insert_char(&mut self, c: char) {
        // `cursor_position` is a *byte* offset (a `String` is indexed by bytes),
        // so it must advance by the char's encoded width, not by 1. Advancing by
        // 1 desyncs the cursor after any multi-byte char (e.g. 'é' is 2 bytes),
        // and a later slice on a non-boundary byte index would panic.
        self.input_buffer.insert(self.cursor_position, c);
        self.cursor_position += c.len_utf8();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position == 0 {
            return;
        }
        // Step back to the start of the preceding char, then remove the whole
        // char there. `String::remove` requires a char-boundary byte index.
        let prev = self.prev_char_boundary(self.cursor_position);
        self.input_buffer.remove(prev);
        self.cursor_position = prev;
    }

    pub fn delete_char_forward(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            // cursor sits on a char boundary by invariant, so this removes the
            // char under the cursor and leaves the cursor where it is.
            self.input_buffer.remove(self.cursor_position);
        }
    }

    pub fn move_cursor_left(&mut self) {
        self.cursor_position = self.prev_char_boundary(self.cursor_position);
    }

    pub fn move_cursor_right(&mut self) {
        self.cursor_position = self.next_char_boundary(self.cursor_position);
    }

    /// Byte index of the char boundary immediately before `byte_idx`
    /// (returns 0 when already at the start).
    fn prev_char_boundary(&self, byte_idx: usize) -> usize {
        if byte_idx == 0 {
            return 0;
        }
        let mut i = byte_idx - 1;
        while i > 0 && !self.input_buffer.is_char_boundary(i) {
            i -= 1;
        }
        i
    }

    /// Byte index of the char boundary immediately after `byte_idx`
    /// (returns the buffer length when already at the end).
    fn next_char_boundary(&self, byte_idx: usize) -> usize {
        let len = self.input_buffer.len();
        if byte_idx >= len {
            return len;
        }
        let mut i = byte_idx + 1;
        while i < len && !self.input_buffer.is_char_boundary(i) {
            i += 1;
        }
        i
    }

    /// Largest char boundary `<= byte_idx`. Used to snap a byte-column computed
    /// during vertical movement back onto a valid boundary so it never lands
    /// mid-char.
    fn floor_char_boundary(&self, byte_idx: usize) -> usize {
        let mut i = byte_idx.min(self.input_buffer.len());
        while i > 0 && !self.input_buffer.is_char_boundary(i) {
            i -= 1;
        }
        i
    }

    pub fn move_cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor_position = self.input_buffer.len();
    }

    pub fn insert_newline(&mut self) -> bool {
        let current_line_count = self.input_buffer.chars().filter(|&c| c == '\n').count() + 1;
        if current_line_count >= 200 {
            return false;
        }
        self.insert_char('\n');
        true
    }

    /// Handles the editing keys common to every input mode (deletion, horizontal
    /// movement, Home/End). Returns `true` if `key` was one of them, so each
    /// public handler can try its mode-specific keys first and delegate the rest
    /// here — the single place this shared behavior lives.
    fn handle_edit_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Backspace => self.delete_char(),
            KeyCode::Delete => self.delete_char_forward(),
            KeyCode::Left => self.move_cursor_left(),
            KeyCode::Right => self.move_cursor_right(),
            KeyCode::Home => self.move_cursor_home(),
            KeyCode::End => self.move_cursor_end(),
            _ => return false,
        }
        true
    }

    pub fn handle_text_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char(c) => {
                self.insert_char(c);
                true
            }
            _ => self.handle_edit_key(key),
        }
    }

    pub fn handle_numeric_input(&mut self, key: KeyCode) -> bool {
        match key {
            // A rejected char still "consumes" the key (returns true) so callers
            // don't fall through to other handling; only the insert is skipped.
            KeyCode::Char(c) => {
                if c.is_ascii_digit() || c == '.' {
                    self.insert_char(c);
                }
                true
            }
            _ => self.handle_edit_key(key),
        }
    }

    pub fn handle_integer_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char(c) => {
                if c.is_ascii_digit() {
                    self.insert_char(c);
                }
                true
            }
            _ => self.handle_edit_key(key),
        }
    }

    pub fn handle_multiline_text_input(&mut self, key: KeyCode, _modifiers: KeyModifiers) -> bool {
        match key {
            KeyCode::Char(c) => {
                self.insert_char(c);
                true
            }
            KeyCode::Up => {
                self.move_cursor_up();
                true
            }
            KeyCode::Down => {
                self.move_cursor_down();
                true
            }
            _ => self.handle_edit_key(key),
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_position == 0 {
            return;
        }

        let text_up_to_cursor = &self.input_buffer[..self.cursor_position];
        let mut current_line_start = 0;
        let mut prev_line_start = 0;

        for (i, ch) in text_up_to_cursor.char_indices() {
            if ch == '\n' {
                prev_line_start = current_line_start;
                current_line_start = i + 1;
            }
        }

        let current_column = self.cursor_position - current_line_start;

        if current_line_start > 0 {
            let prev_line_end = current_line_start - 1;
            let prev_line_length = prev_line_end - prev_line_start;

            let new_column = std::cmp::min(current_column, prev_line_length);
            self.cursor_position = self.floor_char_boundary(prev_line_start + new_column);
        } else {
            self.cursor_position = 0;
        }
    }

    pub fn move_cursor_down(&mut self) {
        let total_length = self.input_buffer.len();
        if self.cursor_position >= total_length {
            return;
        }

        let text_up_to_cursor = &self.input_buffer[..self.cursor_position];
        let mut current_line_start = 0;

        for (i, ch) in text_up_to_cursor.char_indices() {
            if ch == '\n' {
                current_line_start = i + 1;
            }
        }

        let current_column = self.cursor_position - current_line_start;

        let remaining_text = &self.input_buffer[self.cursor_position..];
        if let Some(newline_pos) = remaining_text.find('\n') {
            let next_line_start = self.cursor_position + newline_pos + 1;

            let text_from_next_line = &self.input_buffer[next_line_start..];
            let next_line_end = if let Some(next_newline) = text_from_next_line.find('\n') {
                next_line_start + next_newline
            } else {
                total_length
            };

            let next_line_length = next_line_end - next_line_start;
            let new_column = std::cmp::min(current_column, next_line_length);
            self.cursor_position = self.floor_char_boundary(next_line_start + new_column);
        } else {
            self.cursor_position = total_length;
        }
    }
}

pub struct SectionNavigator;

impl SectionNavigator {
    pub fn move_focus_down(current: &FocusedSection) -> FocusedSection {
        match current {
            FocusedSection::Measurements { .. } => FocusedSection::Running {
                focused_field: RunningField::Miles,
            },
            FocusedSection::Running { .. } => FocusedSection::FoodItems,
            FocusedSection::FoodItems => FocusedSection::Sokay,
            FocusedSection::Sokay => FocusedSection::StrengthMobility,
            FocusedSection::StrengthMobility => FocusedSection::Notes,
            FocusedSection::Notes => FocusedSection::Measurements {
                focused_field: MeasurementField::Weight,
            },
        }
    }

    pub fn move_focus_up(current: &FocusedSection) -> FocusedSection {
        match current {
            FocusedSection::Measurements { .. } => FocusedSection::Notes,
            FocusedSection::Running { .. } => FocusedSection::Measurements {
                focused_field: MeasurementField::Weight,
            },
            FocusedSection::FoodItems => FocusedSection::Running {
                focused_field: RunningField::Miles,
            },
            FocusedSection::Sokay => FocusedSection::FoodItems,
            FocusedSection::StrengthMobility => FocusedSection::Sokay,
            FocusedSection::Notes => FocusedSection::StrengthMobility,
        }
    }

    /// Section/field to focus after a single-value field is saved with data,
    /// stepping one field forward in entry order and wrapping Notes → Weight.
    /// Food/Sokay are focus-only landing spots (their add-dialogs handle repeat
    /// entry), so nothing auto-opens here.
    pub fn advance_field(field: FieldType) -> FocusedSection {
        match field {
            FieldType::Weight => FocusedSection::Measurements {
                focused_field: MeasurementField::Waist,
            },
            FieldType::Waist => FocusedSection::Running {
                focused_field: RunningField::Miles,
            },
            FieldType::Miles => FocusedSection::Running {
                focused_field: RunningField::Elevation,
            },
            FieldType::Elevation => FocusedSection::FoodItems,
            FieldType::StrengthMobility => FocusedSection::Notes,
            FieldType::Notes => FocusedSection::Measurements {
                focused_field: MeasurementField::Weight,
            },
        }
    }

    /// The field's own section focus, used to keep focus put when a save leaves
    /// the value empty (no advance).
    pub fn field_section(field: FieldType) -> FocusedSection {
        match field {
            FieldType::Weight => FocusedSection::Measurements {
                focused_field: MeasurementField::Weight,
            },
            FieldType::Waist => FocusedSection::Measurements {
                focused_field: MeasurementField::Waist,
            },
            FieldType::Miles => FocusedSection::Running {
                focused_field: RunningField::Miles,
            },
            FieldType::Elevation => FocusedSection::Running {
                focused_field: RunningField::Elevation,
            },
            FieldType::StrengthMobility => FocusedSection::StrengthMobility,
            FieldType::Notes => FocusedSection::Notes,
        }
    }

    pub fn toggle_internal_focus(current: &FocusedSection) -> FocusedSection {
        match current {
            FocusedSection::Measurements { focused_field } => {
                let new_field = match focused_field {
                    MeasurementField::Weight => MeasurementField::Waist,
                    MeasurementField::Waist => MeasurementField::Weight,
                };
                FocusedSection::Measurements {
                    focused_field: new_field,
                }
            }
            FocusedSection::Running { focused_field } => {
                let new_field = match focused_field {
                    RunningField::Miles => RunningField::Elevation,
                    RunningField::Elevation => RunningField::Miles,
                };
                FocusedSection::Running {
                    focused_field: new_field,
                }
            }
            _ => current.clone(),
        }
    }
}

pub struct NavigationHandler;

impl NavigationHandler {
    pub fn move_selection_down(current_index: Option<usize>, list_len: usize) -> Option<usize> {
        if list_len == 0 {
            return None;
        }

        match current_index {
            Some(i) => {
                if i >= list_len - 1 {
                    Some(0)
                } else {
                    Some(i + 1)
                }
            }
            None => Some(0),
        }
    }

    pub fn move_selection_up(current_index: Option<usize>, list_len: usize) -> Option<usize> {
        if list_len == 0 {
            return None;
        }

        match current_index {
            Some(i) => {
                if i == 0 {
                    Some(list_len - 1)
                } else {
                    Some(i - 1)
                }
            }
            None => Some(0),
        }
    }
}

pub struct ActionHandler;

impl ActionHandler {
    pub fn save_food_entry(state: &mut AppState, food_name: String) -> Option<DailyLog> {
        if !food_name.is_empty() {
            let food_entry = FoodEntry::new(food_name);
            let log = state.get_or_create_daily_log(state.selected_date);
            log.add_food_entry(food_entry);
            return Some(log.clone());
        }
        None
    }

    /// Background persistence to avoid blocking UI
    pub async fn persist_daily_log(
        db_manager: Arc<RwLock<DbManager>>,
        file_manager: &FileManager,
        log: DailyLog,
    ) {
        let mut db = db_manager.write().await;
        let _ = db.save_daily_log(&log).await;
        let _ = file_manager.save_daily_log(&log);
    }

    pub fn update_food_entry(
        state: &mut AppState,
        food_index: usize,
        new_name: String,
    ) -> Option<DailyLog> {
        if !new_name.is_empty()
            && let Some(log) = state
                .daily_logs
                .iter_mut()
                .find(|log| log.date == state.selected_date)
            && food_index < log.food_entries.len()
        {
            log.food_entries[food_index].name = new_name;
            return Some(log.clone());
        }
        None
    }

    pub fn delete_food_entry(state: &mut AppState, food_index: usize) -> Option<DailyLog> {
        if let Some(log) = state
            .daily_logs
            .iter_mut()
            .find(|log| log.date == state.selected_date)
            && food_index < log.food_entries.len()
        {
            log.remove_food_entry(food_index);
            return Some(log.clone());
        }
        None
    }

    pub fn handle_home_enter(state: &mut AppState, selected_index: Option<usize>) {
        if let Some(index) = selected_index {
            if index < state.daily_logs.len() {
                state.selected_date = state.daily_logs[index].date;
            }
        } else {
            state.selected_date = chrono::Local::now().date_naive();
        }
        state.current_screen = AppScreen::DailyView;
    }

    /// Generic field accessor - gets current value for editing
    pub fn start_edit_field(state: &AppState, field_type: FieldType) -> String {
        field_type.get_value(state)
    }

    /// Generic field updater - updates field with new value
    pub fn update_field(state: &mut AppState, field_type: FieldType, input: String) -> DailyLog {
        field_type.update_value(state, input)
    }

    pub fn start_edit_food(state: &AppState, food_index: usize) -> Option<String> {
        if let Some(log) = state.get_daily_log(state.selected_date)
            && food_index < log.food_entries.len()
        {
            return Some(log.food_entries[food_index].name.clone());
        }
        None
    }

    pub fn save_sokay_entry(state: &mut AppState, sokay_text: String) -> Option<DailyLog> {
        if !sokay_text.is_empty() {
            let log = state.get_or_create_daily_log(state.selected_date);
            log.add_sokay_entry(sokay_text);
            return Some(log.clone());
        }
        None
    }

    pub fn update_sokay_entry(
        state: &mut AppState,
        sokay_index: usize,
        new_text: String,
    ) -> Option<DailyLog> {
        if !new_text.is_empty()
            && let Some(log) = state
                .daily_logs
                .iter_mut()
                .find(|log| log.date == state.selected_date)
            && sokay_index < log.sokay_entries.len()
        {
            log.sokay_entries[sokay_index] = new_text;
            return Some(log.clone());
        }
        None
    }

    pub fn delete_sokay_entry(state: &mut AppState, sokay_index: usize) -> Option<DailyLog> {
        if let Some(log) = state
            .daily_logs
            .iter_mut()
            .find(|log| log.date == state.selected_date)
            && sokay_index < log.sokay_entries.len()
        {
            log.remove_sokay_entry(sokay_index);
            return Some(log.clone());
        }
        None
    }

    pub fn start_edit_sokay(state: &AppState, sokay_index: usize) -> Option<String> {
        if let Some(log) = state.get_daily_log(state.selected_date)
            && sokay_index < log.sokay_entries.len()
        {
            return Some(log.sokay_entries[sokay_index].clone());
        }
        None
    }

    pub fn calculate_cumulative_sokay(state: &AppState, up_to_date: chrono::NaiveDate) -> usize {
        state
            .daily_logs
            .iter()
            .filter(|log| log.date <= up_to_date)
            .map(|log| log.sokay_entries.len())
            .sum()
    }

    pub async fn delete_daily_log(
        state: &mut AppState,
        db_manager: &mut DbManager,
        file_manager: &FileManager,
        date: chrono::NaiveDate,
    ) -> anyhow::Result<()> {
        db_manager.delete_daily_log(date).await?;
        state.daily_logs.retain(|log| log.date != date);
        let _ = file_manager.delete_daily_log(date);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod input_handler {
        use super::*;

        #[test]
        fn inserts_ascii_and_tracks_cursor() {
            let mut h = InputHandler::new();
            for c in "abc".chars() {
                h.insert_char(c);
            }
            assert_eq!(h.input_buffer, "abc");
            assert_eq!(h.cursor_position, 3);
        }

        #[test]
        fn multibyte_insert_advances_cursor_by_utf8_width() {
            let mut h = InputHandler::new();
            h.insert_char('é'); // 2 bytes in UTF-8
            assert_eq!(h.cursor_position, 2);
            h.insert_char('x');
            assert_eq!(h.input_buffer, "éx");
            assert_eq!(h.cursor_position, 3);
        }

        #[test]
        fn backspace_removes_whole_multibyte_char() {
            let mut h = InputHandler::new();
            h.set_input("aé".to_string()); // cursor at end (byte 3)
            h.delete_char();
            assert_eq!(h.input_buffer, "a");
            assert_eq!(h.cursor_position, 1);
            h.delete_char();
            assert_eq!(h.input_buffer, "");
            assert_eq!(h.cursor_position, 0);
            h.delete_char(); // no-op at start
            assert_eq!(h.cursor_position, 0);
        }

        #[test]
        fn horizontal_movement_lands_on_char_boundaries() {
            let mut h = InputHandler::new();
            h.set_input("aé".to_string());
            h.move_cursor_home();
            h.move_cursor_right(); // past 'a'
            assert_eq!(h.cursor_position, 1);
            h.move_cursor_right(); // past 'é' (2 bytes)
            assert_eq!(h.cursor_position, 3);
            h.move_cursor_right(); // clamped at end
            assert_eq!(h.cursor_position, 3);
            h.move_cursor_left();
            assert_eq!(h.cursor_position, 1);
        }

        #[test]
        fn delete_forward_removes_char_under_cursor() {
            let mut h = InputHandler::new();
            h.set_input("aé".to_string());
            h.move_cursor_home();
            h.delete_char_forward();
            assert_eq!(h.input_buffer, "é");
            assert_eq!(h.cursor_position, 0);
        }

        #[test]
        fn numeric_input_rejects_letters_but_still_consumes_key() {
            let mut h = InputHandler::new();
            assert!(h.handle_numeric_input(KeyCode::Char('1')));
            assert!(h.handle_numeric_input(KeyCode::Char('.')));
            assert!(h.handle_numeric_input(KeyCode::Char('a'))); // consumed, not inserted
            assert_eq!(h.input_buffer, "1.");
        }

        #[test]
        fn integer_input_rejects_decimal_point() {
            let mut h = InputHandler::new();
            h.handle_integer_input(KeyCode::Char('1'));
            h.handle_integer_input(KeyCode::Char('.'));
            h.handle_integer_input(KeyCode::Char('2'));
            assert_eq!(h.input_buffer, "12");
        }

        #[test]
        fn shared_edit_keys_work_across_modes() {
            let mut h = InputHandler::new();
            h.set_input("12".to_string());
            assert!(h.handle_numeric_input(KeyCode::Backspace));
            assert_eq!(h.input_buffer, "1");
            // Up is not an edit key in single-line text mode, so it is not consumed.
            assert!(!h.handle_text_input(KeyCode::Up));
        }

        #[test]
        fn vertical_movement_lands_on_char_boundary() {
            let mut h = InputHandler::new();
            h.set_input("abc\ndé".to_string());
            h.move_cursor_up();
            assert!(h.input_buffer.is_char_boundary(h.cursor_position));
            h.move_cursor_down();
            assert!(h.input_buffer.is_char_boundary(h.cursor_position));
        }
    }

    mod navigation_handler {
        use super::*;

        #[test]
        fn test_move_selection_down_empty_list() {
            let result = NavigationHandler::move_selection_down(None, 0);
            assert_eq!(result, None);
        }

        #[test]
        fn test_move_selection_down_single_item() {
            // Starting at None should select the first item
            let result = NavigationHandler::move_selection_down(None, 1);
            assert_eq!(result, Some(0));

            // Moving down from the only item should wrap to top
            let result = NavigationHandler::move_selection_down(Some(0), 1);
            assert_eq!(result, Some(0));
        }

        #[test]
        fn test_move_selection_down_multiple_items() {
            let list_len = 5;

            // Starting at None should select first item
            let result = NavigationHandler::move_selection_down(None, list_len);
            assert_eq!(result, Some(0));

            // Normal navigation down
            let result = NavigationHandler::move_selection_down(Some(0), list_len);
            assert_eq!(result, Some(1));

            let result = NavigationHandler::move_selection_down(Some(1), list_len);
            assert_eq!(result, Some(2));

            let result = NavigationHandler::move_selection_down(Some(3), list_len);
            assert_eq!(result, Some(4));
        }

        #[test]
        fn test_move_selection_down_wraparound() {
            let list_len = 3;

            // At the bottom (index 2), should wrap to top (index 0)
            let result = NavigationHandler::move_selection_down(Some(2), list_len);
            assert_eq!(result, Some(0));
        }
    }

    mod section_navigator {
        use super::*;

        // Auto-advance after saving a single-value field: focus should step one
        // field forward in entry order and wrap Notes -> Weight.
        #[test]
        fn test_advance_field_full_chain() {
            assert_eq!(
                SectionNavigator::advance_field(FieldType::Weight),
                FocusedSection::Measurements {
                    focused_field: MeasurementField::Waist
                }
            );
            assert_eq!(
                SectionNavigator::advance_field(FieldType::Waist),
                FocusedSection::Running {
                    focused_field: RunningField::Miles
                }
            );
            assert_eq!(
                SectionNavigator::advance_field(FieldType::Miles),
                FocusedSection::Running {
                    focused_field: RunningField::Elevation
                }
            );
            // Elevation advances into the Food list (focus only, no dialog).
            assert_eq!(
                SectionNavigator::advance_field(FieldType::Elevation),
                FocusedSection::FoodItems
            );
            assert_eq!(
                SectionNavigator::advance_field(FieldType::StrengthMobility),
                FocusedSection::Notes
            );
            // Notes wraps back to the top of the chain.
            assert_eq!(
                SectionNavigator::advance_field(FieldType::Notes),
                FocusedSection::Measurements {
                    focused_field: MeasurementField::Weight
                }
            );
        }

        // Empty save stays put: field_section maps each field to its own focus.
        #[test]
        fn test_field_section_stays_on_field() {
            assert_eq!(
                SectionNavigator::field_section(FieldType::Weight),
                FocusedSection::Measurements {
                    focused_field: MeasurementField::Weight
                }
            );
            assert_eq!(
                SectionNavigator::field_section(FieldType::Waist),
                FocusedSection::Measurements {
                    focused_field: MeasurementField::Waist
                }
            );
            assert_eq!(
                SectionNavigator::field_section(FieldType::Miles),
                FocusedSection::Running {
                    focused_field: RunningField::Miles
                }
            );
            assert_eq!(
                SectionNavigator::field_section(FieldType::Elevation),
                FocusedSection::Running {
                    focused_field: RunningField::Elevation
                }
            );
            assert_eq!(
                SectionNavigator::field_section(FieldType::StrengthMobility),
                FocusedSection::StrengthMobility
            );
            assert_eq!(
                SectionNavigator::field_section(FieldType::Notes),
                FocusedSection::Notes
            );
        }
    }

    mod action_handler {
        use super::*;

        fn date(y: i32, m: u32, d: u32) -> chrono::NaiveDate {
            chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap()
        }

        #[test]
        fn save_food_entry_adds_and_rejects_empty() {
            let mut state = AppState::new();
            // Empty name is a no-op returning None (nothing to persist).
            assert!(ActionHandler::save_food_entry(&mut state, String::new()).is_none());

            let log = ActionHandler::save_food_entry(&mut state, "Eggs".to_string()).unwrap();
            assert_eq!(log.food_entries.len(), 1);
            assert_eq!(log.food_entries[0].name, "Eggs");
        }

        #[test]
        fn update_food_entry_respects_index_and_bounds() {
            let mut state = AppState::new();
            ActionHandler::save_food_entry(&mut state, "Eggs".to_string());

            let log = ActionHandler::update_food_entry(&mut state, 0, "Bacon".to_string()).unwrap();
            assert_eq!(log.food_entries[0].name, "Bacon");
            // Empty replacement and out-of-range index both reject.
            assert!(ActionHandler::update_food_entry(&mut state, 0, String::new()).is_none());
            assert!(ActionHandler::update_food_entry(&mut state, 9, "X".to_string()).is_none());
        }

        #[test]
        fn delete_food_entry_removes_and_bounds() {
            let mut state = AppState::new();
            ActionHandler::save_food_entry(&mut state, "Eggs".to_string());
            assert!(ActionHandler::delete_food_entry(&mut state, 9).is_none());

            let log = ActionHandler::delete_food_entry(&mut state, 0).unwrap();
            assert!(log.food_entries.is_empty());
        }

        #[test]
        fn sokay_add_update_delete_cycle() {
            let mut state = AppState::new();
            assert!(ActionHandler::save_sokay_entry(&mut state, String::new()).is_none());
            ActionHandler::save_sokay_entry(&mut state, "Soda".to_string()).unwrap();

            let log =
                ActionHandler::update_sokay_entry(&mut state, 0, "Candy".to_string()).unwrap();
            assert_eq!(log.sokay_entries[0], "Candy");
            assert!(ActionHandler::update_sokay_entry(&mut state, 5, "X".to_string()).is_none());

            let log = ActionHandler::delete_sokay_entry(&mut state, 0).unwrap();
            assert!(log.sokay_entries.is_empty());
        }

        #[test]
        fn cumulative_sokay_counts_up_to_and_including_date() {
            let mut state = AppState::new();
            state
                .get_or_create_daily_log(date(2026, 7, 1))
                .add_sokay_entry("a".to_string());
            state
                .get_or_create_daily_log(date(2026, 7, 2))
                .add_sokay_entry("b".to_string());
            state
                .get_or_create_daily_log(date(2026, 7, 2))
                .add_sokay_entry("c".to_string());
            state
                .get_or_create_daily_log(date(2026, 7, 3))
                .add_sokay_entry("d".to_string());

            // Boundary is inclusive: through 07-02 counts the 07-01 and both 07-02 entries.
            assert_eq!(
                ActionHandler::calculate_cumulative_sokay(&state, date(2026, 7, 2)),
                3
            );
            assert_eq!(
                ActionHandler::calculate_cumulative_sokay(&state, date(2026, 7, 3)),
                4
            );
        }

        #[test]
        fn home_enter_with_index_selects_that_day() {
            let mut state = AppState::new();
            let d = date(2026, 6, 1);
            state.get_or_create_daily_log(d);

            ActionHandler::handle_home_enter(&mut state, Some(0));
            assert_eq!(state.selected_date, d);
            assert!(matches!(state.current_screen, AppScreen::DailyView));
        }
    }
}
