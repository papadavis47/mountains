use super::*;

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
                        // Reset scroll when leaving expanded sections
                        self.state.strength_mobility_scroll = 0;
                        self.state.notes_scroll = 0;
                        self.state.focused_section =
                            SectionNavigator::move_focus_down(&self.state.focused_section);
                    }
                    return Ok(());
                }
                KeyCode::Char('K') => {
                    if matches!(self.state.current_screen, AppScreen::DailyView) {
                        // Reset scroll when leaving expanded sections
                        self.state.strength_mobility_scroll = 0;
                        self.state.notes_scroll = 0;
                        self.state.focused_section =
                            SectionNavigator::move_focus_up(&self.state.focused_section);
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
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_edit_weight();
                }
            }
            KeyCode::Char('s') => {
                if matches!(self.state.current_screen, AppScreen::Startup) {
                    self.state.current_screen = AppScreen::Statistics;
                } else if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_edit_waist();
                }
            }
            KeyCode::Char('t') => {
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_edit_strength_mobility();
                }
            }
            KeyCode::Char('n') => {
                if matches!(self.state.current_screen, AppScreen::Startup) {
                    self.state.selected_date = chrono::Local::now().date_naive();
                    self.state.get_or_create_daily_log(self.state.selected_date);
                    self.state.current_screen = AppScreen::DailyView;
                } else if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_edit_notes();
                }
            }
            KeyCode::Char('m') => {
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_edit_miles();
                }
            }
            KeyCode::Char('l') => {
                if matches!(self.state.current_screen, AppScreen::Startup) {
                    self.state.current_screen = AppScreen::Home;
                } else if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_edit_elevation();
                }
            }
            KeyCode::Char('c') => {
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.state.current_screen = AppScreen::AddSokay;
                } else if matches!(self.state.current_screen, AppScreen::Startup) {
                    self.open_config_sync();
                }
            }
            KeyCode::Char('S') => {
                if matches!(
                    self.state.current_screen,
                    AppScreen::Home | AppScreen::DailyView
                ) {
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
                MeasurementField::Weight => self.handle_edit_weight(),
                MeasurementField::Waist => self.handle_edit_waist(),
            },
            FocusedSection::Running { focused_field } => match focused_field {
                RunningField::Miles => self.handle_edit_miles(),
                RunningField::Elevation => self.handle_edit_elevation(),
            },
            FocusedSection::FoodItems => {
                self.state.current_screen = AppScreen::AddFood;
            }
            FocusedSection::Sokay => {
                self.state.current_screen = AppScreen::AddSokay;
            }
            FocusedSection::StrengthMobility => {
                self.handle_edit_strength_mobility();
            }
            FocusedSection::Notes => {
                self.handle_edit_notes();
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
                    self.state.current_screen = AppScreen::Home;
                }
            },
            _ => {}
        }
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

    pub(super) fn handle_edit_weight(&mut self) {
        use crate::models::field_accessor::FieldType;
        self.handle_edit_field(FieldType::Weight);
    }

    pub(super) fn handle_edit_field(&mut self, field: crate::models::field_accessor::FieldType) {
        let current_value = ActionHandler::start_edit_field(&self.state, field);
        self.input_handler.set_input(current_value);
        self.state.current_screen = AppScreen::InputField(field);
    }

    pub(super) fn handle_edit_waist(&mut self) {
        use crate::models::field_accessor::FieldType;
        self.handle_edit_field(FieldType::Waist);
    }

    pub(super) fn handle_edit_strength_mobility(&mut self) {
        use crate::models::field_accessor::FieldType;
        self.handle_edit_field(FieldType::StrengthMobility);
    }

    pub(super) fn handle_edit_notes(&mut self) {
        use crate::models::field_accessor::FieldType;
        self.handle_edit_field(FieldType::Notes);
    }

    pub(super) fn handle_edit_miles(&mut self) {
        use crate::models::field_accessor::FieldType;
        self.handle_edit_field(FieldType::Miles);
    }

    pub(super) fn handle_edit_elevation(&mut self) {
        use crate::models::field_accessor::FieldType;
        self.handle_edit_field(FieldType::Elevation);
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
