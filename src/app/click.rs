use super::*;

impl App {
    pub(super) fn handle_mouse_event(&mut self, mouse: MouseEvent) {
        let Some((column, row)) = left_click_position(mouse) else {
            return;
        };
        if !matches!(
            self.state.current_screen,
            AppScreen::Startup
                | AppScreen::Statistics
                | AppScreen::Home
                | AppScreen::DailyView
                | AppScreen::ConfigSync
        ) {
            return;
        }

        if let Some(action) = hit_test(&self.click_targets, column, row) {
            self.handle_click_action(action);
        }
    }

    pub(super) fn handle_click_action(&mut self, action: ClickAction) {
        match action {
            ClickAction::StartupToday
                if matches!(self.state.current_screen, AppScreen::Startup) =>
            {
                self.state.selected_date = chrono::Local::now().date_naive();
                self.state.get_or_create_daily_log(self.state.selected_date);
                self.state.current_screen = AppScreen::DailyView;
            }
            ClickAction::StartupLogs if matches!(self.state.current_screen, AppScreen::Startup) => {
                self.state.current_screen = AppScreen::Home;
            }
            ClickAction::StartupAddDate
                if matches!(self.state.current_screen, AppScreen::Startup) =>
            {
                self.input_handler.clear();
                self.state.date_input_error = None;
                self.state.current_screen = AppScreen::DateInput;
            }
            ClickAction::OpenStatistics
                if matches!(self.state.current_screen, AppScreen::Startup) =>
            {
                self.state.current_screen = AppScreen::Statistics;
            }
            ClickAction::OpenCloudSync
                if matches!(self.state.current_screen, AppScreen::Startup) =>
            {
                self.open_config_sync();
            }
            ClickAction::Quit
                if matches!(
                    self.state.current_screen,
                    AppScreen::Startup | AppScreen::Statistics
                ) =>
            {
                self.state.current_screen = AppScreen::Syncing;
            }
            ClickAction::BackToStartup
                if matches!(self.state.current_screen, AppScreen::Statistics) =>
            {
                self.state.current_screen = AppScreen::Startup;
            }
            ClickAction::OpenLog(index) if matches!(self.state.current_screen, AppScreen::Home) => {
                self.list_state.select(Some(index));
                ActionHandler::handle_home_enter(&mut self.state, Some(index));
            }
            ClickAction::EditField(field)
                if matches!(self.state.current_screen, AppScreen::DailyView)
                    && matches!(
                        field,
                        crate::models::field_accessor::FieldType::Weight
                            | crate::models::field_accessor::FieldType::Waist
                            | crate::models::field_accessor::FieldType::Miles
                            | crate::models::field_accessor::FieldType::Elevation
                    ) =>
            {
                self.state.focused_section = SectionNavigator::field_section(field);
                self.handle_edit_field(field);
            }
            ClickAction::AddFood if matches!(self.state.current_screen, AppScreen::DailyView) => {
                self.state.focused_section = FocusedSection::FoodItems;
                self.state.current_screen = AppScreen::AddFood;
            }
            ClickAction::SelectFood(index)
                if matches!(self.state.current_screen, AppScreen::DailyView) =>
            {
                let edit_selected = matches!(self.state.focused_section, FocusedSection::FoodItems)
                    && self.state.food_list_focused
                    && self.food_list_state.selected() == Some(index);
                self.state.focused_section = FocusedSection::FoodItems;
                self.state.food_list_focused = true;
                self.food_list_state.select(Some(index));
                if edit_selected {
                    self.handle_edit_food();
                }
            }
            ClickAction::AddSokay if matches!(self.state.current_screen, AppScreen::DailyView) => {
                self.state.focused_section = FocusedSection::Sokay;
                self.state.current_screen = AppScreen::AddSokay;
            }
            ClickAction::SelectSokay(index)
                if matches!(self.state.current_screen, AppScreen::DailyView) =>
            {
                let edit_selected = matches!(self.state.focused_section, FocusedSection::Sokay)
                    && self.state.sokay_list_focused
                    && self.sokay_list_state.selected() == Some(index);
                self.state.focused_section = FocusedSection::Sokay;
                self.state.sokay_list_focused = true;
                self.sokay_list_state.select(Some(index));
                if edit_selected {
                    self.handle_edit_sokay();
                }
            }
            ClickAction::StrengthMobility
                if matches!(self.state.current_screen, AppScreen::DailyView) =>
            {
                if matches!(self.state.focused_section, FocusedSection::StrengthMobility) {
                    self.handle_edit_strength_mobility();
                } else {
                    self.state.strength_mobility_scroll = 0;
                    self.state.notes_scroll = 0;
                    self.state.focused_section = FocusedSection::StrengthMobility;
                }
            }
            ClickAction::Notes if matches!(self.state.current_screen, AppScreen::DailyView) => {
                if matches!(self.state.focused_section, FocusedSection::Notes) {
                    self.handle_edit_notes();
                } else {
                    self.state.strength_mobility_scroll = 0;
                    self.state.notes_scroll = 0;
                    self.state.focused_section = FocusedSection::Notes;
                }
            }
            ClickAction::FocusConfigField(field)
                if matches!(self.state.current_screen, AppScreen::ConfigSync) =>
            {
                self.focus_config_sync_field(field);
            }
            ClickAction::ToggleConfigSync
                if matches!(self.state.current_screen, AppScreen::ConfigSync) =>
            {
                self.focus_config_sync_field(ConfigSyncField::EnableToggle);
                self.config_sync_enabled = !self.config_sync_enabled;
            }
            _ => {}
        }
    }
}
