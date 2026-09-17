use super::*;
use crate::models::field_accessor::FieldType;

impl App {
    pub(super) async fn handle_navigation_input(
        &mut self,
        key: KeyCode,
        modifiers: crossterm::event::KeyModifiers,
    ) -> Result<()> {
        // Shift+J/K switches section focus in DailyView
        if modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
            match key {
                KeyCode::Char('J') => {
                    if matches!(self.state.current_screen, AppScreen::DailyView) {
                        self.focus_section(SectionNavigator::move_focus_down(
                            &self.state.focused_section,
                            self.state.simple_mode,
                        ));
                    }
                    return Ok(());
                }
                KeyCode::Char('K') => {
                    if matches!(self.state.current_screen, AppScreen::DailyView) {
                        self.focus_section(SectionNavigator::move_focus_up(
                            &self.state.focused_section,
                            self.state.simple_mode,
                        ));
                    }
                    return Ok(());
                }
                _ => {}
            }
        }

        match key {
            KeyCode::Char('q') => {
                self.state.current_screen = AppScreen::Syncing;
            }
            KeyCode::Tab => {
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.state.focused_section =
                        SectionNavigator::toggle_internal_focus(&self.state.focused_section);
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    match self.state.focused_section {
                        FocusedSection::FoodItems => self.move_food_selection_down(),
                        FocusedSection::Sokay => self.move_sokay_selection_down(),
                        FocusedSection::StrengthMobility => {
                            let max = self.strength_mobility_max_scroll();
                            self.state.strength_mobility_scroll = self
                                .state
                                .strength_mobility_scroll
                                .saturating_add(1)
                                .min(max);
                        }
                        FocusedSection::Notes => {
                            let max = self.notes_max_scroll();
                            self.state.notes_scroll =
                                self.state.notes_scroll.saturating_add(1).min(max);
                        }
                        _ => {}
                    }
                } else if matches!(self.state.current_screen, AppScreen::Home) {
                    self.move_selection_down();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    match self.state.focused_section {
                        FocusedSection::FoodItems => self.move_food_selection_up(),
                        FocusedSection::Sokay => self.move_sokay_selection_up(),
                        FocusedSection::StrengthMobility => {
                            self.state.strength_mobility_scroll =
                                self.state.strength_mobility_scroll.saturating_sub(1);
                        }
                        FocusedSection::Notes => {
                            self.state.notes_scroll = self.state.notes_scroll.saturating_sub(1);
                        }
                        _ => {}
                    }
                } else if matches!(self.state.current_screen, AppScreen::Home) {
                    self.move_selection_up();
                }
            }
            KeyCode::Enter => {
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_section_enter().await?;
                } else {
                    self.handle_enter();
                }
            }
            KeyCode::Esc => {
                self.handle_escape();
            }
            KeyCode::Char('d') => {
                if matches!(self.state.current_screen, AppScreen::Home) {
                    self.handle_delete_day_confirmation();
                } else if matches!(self.state.current_screen, AppScreen::DailyView) {
                    use crate::models::DeleteTarget;
                    match self.state.focused_section {
                        FocusedSection::FoodItems => {
                            if self.state.food_list_focused
                                && let Some(selected_index) = self.food_list_state.selected()
                            {
                                self.state.current_screen =
                                    AppScreen::ConfirmDelete(DeleteTarget::Food(selected_index));
                            }
                        }
                        FocusedSection::Sokay => {
                            if self.state.sokay_list_focused
                                && let Some(selected_index) = self.sokay_list_state.selected()
                            {
                                self.state.current_screen =
                                    AppScreen::ConfirmDelete(DeleteTarget::Sokay(selected_index));
                            }
                        }
                        _ => {}
                    }
                }
            }
            KeyCode::Char('f') => {
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.focus_section(FocusedSection::FoodItems);
                    self.state.current_screen = AppScreen::AddFood;
                }
            }
            KeyCode::Char('e') => {
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    match self.state.focused_section {
                        FocusedSection::FoodItems => self.handle_edit_food(),
                        FocusedSection::Sokay => self.handle_edit_sokay(),
                        _ => {}
                    }
                }
            }
            KeyCode::Char('w') => {
                if matches!(self.state.current_screen, AppScreen::DailyView)
                    && self.daily_view_key_enabled('w')
                {
                    self.handle_edit_field(FieldType::Weight, EditOrigin::Shortcut);
                }
            }
            KeyCode::Char('s') => {
                if matches!(self.state.current_screen, AppScreen::Startup) {
                    self.state.current_screen = AppScreen::Statistics;
                } else if matches!(self.state.current_screen, AppScreen::DailyView)
                    && self.daily_view_key_enabled('s')
                {
                    self.handle_edit_field(FieldType::Waist, EditOrigin::Shortcut);
                }
            }
            KeyCode::Char('t') => {
                if matches!(self.state.current_screen, AppScreen::DailyView)
                    && self.daily_view_key_enabled('t')
                {
                    self.handle_edit_field(FieldType::StrengthMobility, EditOrigin::Shortcut);
                }
            }
            KeyCode::Char('n') => {
                if matches!(self.state.current_screen, AppScreen::Startup) {
                    self.state.selected_date = chrono::Local::now().date_naive();
                    self.state.get_or_create_daily_log(self.state.selected_date);
                    self.state.current_screen = AppScreen::DailyView;
                } else if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_edit_field(FieldType::Notes, EditOrigin::Shortcut);
                }
            }
            KeyCode::Char('m') => {
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_edit_field(FieldType::Miles, EditOrigin::Shortcut);
                }
            }
            KeyCode::Char('l') => {
                if matches!(self.state.current_screen, AppScreen::Startup) {
                    self.state.current_screen = AppScreen::Home;
                } else if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_edit_field(FieldType::Elevation, EditOrigin::Shortcut);
                }
            }
            KeyCode::Char('c') => {
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    if self.daily_view_key_enabled('c') {
                        self.focus_section(FocusedSection::Sokay);
                        self.state.current_screen = AppScreen::AddSokay;
                    }
                } else if matches!(self.state.current_screen, AppScreen::Startup) {
                    self.open_config_sync();
                }
            }
            KeyCode::Char('S') => {
                if matches!(self.state.current_screen, AppScreen::Home)
                    || (matches!(self.state.current_screen, AppScreen::DailyView)
                        && self.daily_view_key_enabled('S'))
                {
                    self.state.current_screen = AppScreen::Startup;
                }
            }
            KeyCode::Char('a') => {
                if matches!(
                    self.state.current_screen,
                    AppScreen::Home | AppScreen::Startup
                ) {
                    self.input_handler.clear();
                    self.state.date_input_error = None;
                    self.state.current_screen = AppScreen::DateInput;
                }
            }
            KeyCode::Char(' ') => {
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.state.current_screen = AppScreen::ShortcutsHelp;
                } else if matches!(self.state.current_screen, AppScreen::ShortcutsHelp) {
                    self.state.current_screen = AppScreen::DailyView;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) fn open_config_sync(&mut self) {
        self.config_url_buffer = self.config.sync.db_url.clone();
        self.config_token_buffer = String::new();
        self.config_sync_enabled = self.config.sync.enabled;
        self.state.config_sync_focused_field = ConfigSyncField::DbUrl;
        self.state.config_sync_status = None;
        self.input_handler
            .set_input(self.config.sync.db_url.clone());
        self.state.current_screen = AppScreen::ConfigSync;
    }

    pub(super) fn focus_config_sync_field(&mut self, field: ConfigSyncField) {
        match self.state.config_sync_focused_field {
            ConfigSyncField::DbUrl => {
                self.config_url_buffer = self.input_handler.input_buffer.clone();
            }
            ConfigSyncField::AuthToken => {
                self.config_token_buffer = self.input_handler.input_buffer.clone();
            }
            ConfigSyncField::EnableToggle => {}
        }

        self.state.config_sync_focused_field = field.clone();
        match field {
            ConfigSyncField::DbUrl => {
                self.input_handler.set_input(self.config_url_buffer.clone());
            }
            ConfigSyncField::AuthToken => {
                self.input_handler
                    .set_input(self.config_token_buffer.clone());
            }
            ConfigSyncField::EnableToggle => self.input_handler.clear(),
        }
    }

    pub(super) async fn handle_section_enter(&mut self) -> Result<()> {
        match &self.state.focused_section {
            FocusedSection::Measurements { focused_field } => match focused_field {
                MeasurementField::Weight => {
                    self.handle_edit_field(FieldType::Weight, EditOrigin::Navigation)
                }
                MeasurementField::Waist => {
                    self.handle_edit_field(FieldType::Waist, EditOrigin::Navigation)
                }
            },
            FocusedSection::Running { focused_field } => match focused_field {
                RunningField::Miles => {
                    self.handle_edit_field(FieldType::Miles, EditOrigin::Navigation)
                }
                RunningField::Elevation => {
                    self.handle_edit_field(FieldType::Elevation, EditOrigin::Navigation)
                }
            },
            FocusedSection::FoodItems => {
                self.state.current_screen = AppScreen::AddFood;
            }
            FocusedSection::Sokay => {
                self.state.current_screen = AppScreen::AddSokay;
            }
            FocusedSection::StrengthMobility => {
                self.handle_edit_field(FieldType::StrengthMobility, EditOrigin::Navigation);
            }
            FocusedSection::Notes => {
                self.handle_edit_field(FieldType::Notes, EditOrigin::Navigation);
            }
        }
        Ok(())
    }

    pub(super) fn move_selection_down(&mut self) {
        if self.list_state.selected().is_none() && !self.state.daily_logs.is_empty() {
            self.list_state.select(Some(0));
        } else {
            let new_selection = NavigationHandler::move_selection_down(
                self.list_state.selected(),
                self.state.daily_logs.len(),
            );
            self.list_state.select(new_selection);
        }
    }

    pub(super) fn move_selection_up(&mut self) {
        if self.list_state.selected().is_none() && !self.state.daily_logs.is_empty() {
            self.list_state
                .select(Some(self.state.daily_logs.len() - 1));
        } else {
            let new_selection = NavigationHandler::move_selection_up(
                self.list_state.selected(),
                self.state.daily_logs.len(),
            );
            self.list_state.select(new_selection);
        }
    }

    pub(super) fn move_food_selection_down(&mut self) {
        if let Some(log) = self.state.get_daily_log(self.state.selected_date) {
            if !self.state.food_list_focused && !log.food_entries.is_empty() {
                self.state.food_list_focused = true;
                self.food_list_state.select(Some(0));
            } else {
                let new_selection = NavigationHandler::move_selection_down(
                    self.food_list_state.selected(),
                    log.food_entries.len(),
                );
                self.food_list_state.select(new_selection);
            }
        }
    }

    pub(super) fn move_food_selection_up(&mut self) {
        if let Some(log) = self.state.get_daily_log(self.state.selected_date) {
            let list_len = log.food_entries.len();
            let is_focused = self.state.food_list_focused;

            if !is_focused && list_len > 0 {
                self.state.food_list_focused = true;
                self.food_list_state.select(Some(list_len - 1));
            } else {
                let new_selection =
                    NavigationHandler::move_selection_up(self.food_list_state.selected(), list_len);
                self.food_list_state.select(new_selection);
            }
        }
    }

    pub(super) fn move_sokay_selection_down(&mut self) {
        if let Some(log) = self.state.get_daily_log(self.state.selected_date) {
            if !self.state.sokay_list_focused && !log.sokay_entries.is_empty() {
                self.state.sokay_list_focused = true;
                self.sokay_list_state.select(Some(0));
            } else {
                let new_selection = NavigationHandler::move_selection_down(
                    self.sokay_list_state.selected(),
                    log.sokay_entries.len(),
                );
                self.sokay_list_state.select(new_selection);
            }
        }
    }

    pub(super) fn move_sokay_selection_up(&mut self) {
        if let Some(log) = self.state.get_daily_log(self.state.selected_date) {
            let list_len = log.sokay_entries.len();
            let is_focused = self.state.sokay_list_focused;

            if !is_focused && list_len > 0 {
                self.state.sokay_list_focused = true;
                self.sokay_list_state.select(Some(list_len - 1));
            } else {
                let new_selection = NavigationHandler::move_selection_up(
                    self.sokay_list_state.selected(),
                    list_len,
                );
                self.sokay_list_state.select(new_selection);
            }
        }
    }

    pub(super) fn handle_enter(&mut self) {
        if let AppScreen::Home = self.state.current_screen {
            ActionHandler::handle_home_enter(&mut self.state, self.list_state.selected());
        }
    }

    pub(super) fn strength_mobility_max_scroll(&self) -> u16 {
        let text = self
            .state
            .get_daily_log(self.state.selected_date)
            .and_then(|l| l.strength_mobility.clone())
            .unwrap_or_default();
        screens::max_scroll_offset(&text, self.state.frame_width, self.state.frame_height)
    }

    pub(super) fn notes_max_scroll(&self) -> u16 {
        let text = self
            .state
            .get_daily_log(self.state.selected_date)
            .and_then(|l| l.notes.clone())
            .unwrap_or_default();
        screens::max_scroll_offset(&text, self.state.frame_width, self.state.frame_height)
    }

    pub(super) fn handle_escape(&mut self) {
        match self.state.current_screen {
            AppScreen::Statistics => {
                self.state.current_screen = AppScreen::Startup;
            }
            AppScreen::Home => {
                self.list_state.select(None);
            }
            AppScreen::ShortcutsHelp => {
                self.state.current_screen = AppScreen::DailyView;
            }
            // Match guards fold the "is this list focused?" test into the pattern
            // itself. The first Esc on a focused Food/Sokay list just unfocuses it;
            // every other case (guard fails, or any other section) falls through to
            // the `_` arm and returns Home, so the shared behavior lives in one place.
            AppScreen::DailyView => match self.state.focused_section {
                FocusedSection::FoodItems if self.state.food_list_focused => {
                    self.state.food_list_focused = false;
                    self.food_list_state.select(None);
                }
                FocusedSection::Sokay if self.state.sokay_list_focused => {
                    self.state.sokay_list_focused = false;
                    self.sokay_list_state.select(None);
                }
                _ => {
                    self.state.go_home_from_daily_view();
                }
            },
            _ => {}
        }
    }

    /// DailyView keys pass through `SectionNavigator::daily_view_key_enabled`
    /// so simple mode can disable the ones that reach hidden sections.
    fn daily_view_key_enabled(&self, key: char) -> bool {
        SectionNavigator::daily_view_key_enabled(key, self.state.simple_mode)
    }

    pub(super) fn handle_edit_food(&mut self) {
        if !self.state.food_list_focused {
            return;
        }

        if let Some(selected_index) = self.food_list_state.selected()
            && let Some(current_name) = ActionHandler::start_edit_food(&self.state, selected_index)
        {
            self.input_handler.set_input(current_name);
            self.state.current_screen = AppScreen::EditFood(selected_index);
        }
    }

    /// Moves section focus. Scroll offsets belong to the section being left, so
    /// they clear only on a real move — re-focusing the current section (a quick
    /// -access key aimed at where you already are) keeps your place in it.
    pub(super) fn focus_section(&mut self, section: FocusedSection) {
        if self.state.focused_section != section {
            self.state.strength_mobility_scroll = 0;
            self.state.notes_scroll = 0;
        }
        self.state.focused_section = section;
    }

    /// Opens `field`'s editor and moves focus onto it, so the daily view behind
    /// the editor marks where the typing lands. `origin` records how we got
    /// here; the save reads it to decide whether to stay or advance.
    pub(super) fn handle_edit_field(&mut self, field: FieldType, origin: EditOrigin) {
        let current_value = ActionHandler::start_edit_field(&self.state, field);
        self.input_handler.set_input(current_value);
        self.focus_section(SectionNavigator::field_section(field));
        self.state.edit_origin = origin;
        self.state.current_screen = AppScreen::InputField(field);
    }

    pub(super) fn handle_edit_sokay(&mut self) {
        if !self.state.sokay_list_focused {
            return;
        }

        if let Some(selected_index) = self.sokay_list_state.selected()
            && let Some(current_text) = ActionHandler::start_edit_sokay(&self.state, selected_index)
        {
            self.input_handler.set_input(current_text);
            self.state.current_screen = AppScreen::EditSokay(selected_index);
        }
    }

    pub(super) fn handle_delete_day_confirmation(&mut self) {
        use crate::models::DeleteTarget;
        if let Some(selected_index) = self.list_state.selected()
            && selected_index < self.state.daily_logs.len()
        {
            self.state.selected_date = self.state.daily_logs[selected_index].date;
            self.state.current_screen = AppScreen::ConfirmDelete(DeleteTarget::Day);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_support::test_app;
    use crate::models::field_accessor::FieldType;
    use crossterm::event::KeyModifiers;
    use tempfile::TempDir;

    async fn daily_view_app(dir: &TempDir) -> App {
        let mut app = test_app(dir).await;
        let date = app.state.selected_date;
        app.state.get_or_create_daily_log(date);
        app.state.current_screen = AppScreen::DailyView;
        app.state.focused_section = FocusedSection::FoodItems;
        app
    }

    #[tokio::test]
    async fn add_food_shortcut_focuses_food_without_selecting_a_row() {
        let dir = TempDir::new().unwrap();
        let mut app = daily_view_app(&dir).await;
        app.state.focused_section = FocusedSection::Notes;

        app.handle_key_event_with_modifiers(KeyCode::Char('f'), KeyModifiers::NONE)
            .await
            .unwrap();

        assert!(matches!(app.state.current_screen, AppScreen::AddFood));
        assert_eq!(app.state.focused_section, FocusedSection::FoodItems);
        assert!(!app.state.food_list_focused);
        assert_eq!(app.food_list_state.selected(), None);
    }

    #[tokio::test]
    async fn add_sokay_shortcut_focuses_sokay_without_selecting_a_row() {
        let dir = TempDir::new().unwrap();
        let mut app = daily_view_app(&dir).await;
        app.state.focused_section = FocusedSection::Notes;

        app.handle_key_event_with_modifiers(KeyCode::Char('c'), KeyModifiers::NONE)
            .await
            .unwrap();

        assert!(matches!(app.state.current_screen, AppScreen::AddSokay));
        assert_eq!(app.state.focused_section, FocusedSection::Sokay);
        assert!(!app.state.sokay_list_focused);
        assert_eq!(app.sokay_list_state.selected(), None);
    }

    #[tokio::test]
    async fn field_shortcuts_focus_the_field_they_open() {
        let dir = TempDir::new().unwrap();
        let cases = [
            (
                'w',
                FieldType::Weight,
                FocusedSection::Measurements {
                    focused_field: MeasurementField::Weight,
                },
            ),
            (
                's',
                FieldType::Waist,
                FocusedSection::Measurements {
                    focused_field: MeasurementField::Waist,
                },
            ),
            (
                'm',
                FieldType::Miles,
                FocusedSection::Running {
                    focused_field: RunningField::Miles,
                },
            ),
            (
                'l',
                FieldType::Elevation,
                FocusedSection::Running {
                    focused_field: RunningField::Elevation,
                },
            ),
            (
                't',
                FieldType::StrengthMobility,
                FocusedSection::StrengthMobility,
            ),
            ('n', FieldType::Notes, FocusedSection::Notes),
        ];

        for (key, field, expected_focus) in cases {
            let mut app = daily_view_app(&dir).await;

            app.handle_key_event_with_modifiers(KeyCode::Char(key), KeyModifiers::NONE)
                .await
                .unwrap();

            assert!(
                matches!(app.state.current_screen, AppScreen::InputField(f) if f == field),
                "'{key}' should open {field:?}"
            );
            assert_eq!(app.state.focused_section, expected_focus, "'{key}' focus");
        }
    }

    #[tokio::test]
    async fn opening_the_editor_for_the_focused_section_keeps_its_scroll() {
        let dir = TempDir::new().unwrap();
        let mut app = daily_view_app(&dir).await;
        app.state.focused_section = FocusedSection::Notes;
        app.state.notes_scroll = 3;

        app.handle_key_event_with_modifiers(KeyCode::Char('n'), KeyModifiers::NONE)
            .await
            .unwrap();

        assert_eq!(app.state.notes_scroll, 3);
    }

    #[tokio::test]
    async fn focusing_a_different_section_resets_scroll() {
        let dir = TempDir::new().unwrap();
        let mut app = daily_view_app(&dir).await;
        app.state.focused_section = FocusedSection::Notes;
        app.state.notes_scroll = 3;

        app.handle_key_event_with_modifiers(KeyCode::Char('w'), KeyModifiers::NONE)
            .await
            .unwrap();

        assert_eq!(app.state.notes_scroll, 0);
    }
}
