use super::*;

impl App {
    pub(super) fn ui(&mut self, f: &mut Frame) {
        self.state.frame_width = f.area().width;
        self.state.frame_height = f.area().height;
        self.click_targets.clear();
        match self.state.current_screen {
            AppScreen::Startup => {
                screens::render_startup_screen(f, &self.state, Some(&mut self.click_targets));
            }
            AppScreen::Statistics => {
                screens::render_statistics_screen(
                    f,
                    &self.state,
                    chrono::Local::now().date_naive(),
                    &mut self.click_targets,
                );
            }
            AppScreen::Home => {
                screens::render_home_screen(
                    f,
                    &self.state,
                    &mut self.list_state,
                    &self.sync_status,
                    Some(&mut self.click_targets),
                );
            }
            AppScreen::DailyView => {
                screens::render_daily_view_screen(
                    f,
                    &self.state,
                    &mut self.food_list_state,
                    &mut self.sokay_list_state,
                    &self.sync_status,
                    None,
                    Some(&mut self.click_targets),
                );
            }
            AppScreen::AddFood => {
                screens::render_add_food_screen(
                    f,
                    &self.state,
                    &mut self.food_list_state,
                    &mut self.sokay_list_state,
                    &self.sync_status,
                    &self.input_handler.input_buffer,
                    self.input_handler.cursor_position,
                );
            }
            AppScreen::EditFood(_) => {
                screens::render_edit_food_screen(
                    f,
                    &self.state,
                    &mut self.food_list_state,
                    &mut self.sokay_list_state,
                    &self.sync_status,
                    &self.input_handler.input_buffer,
                    self.input_handler.cursor_position,
                );
            }
            AppScreen::AddSokay => {
                screens::render_add_sokay_screen(
                    f,
                    &self.state,
                    &mut self.food_list_state,
                    &mut self.sokay_list_state,
                    &self.sync_status,
                    &self.input_handler.input_buffer,
                    self.input_handler.cursor_position,
                );
            }
            AppScreen::EditSokay(_) => {
                screens::render_edit_sokay_screen(
                    f,
                    &self.state,
                    &mut self.food_list_state,
                    &mut self.sokay_list_state,
                    &self.sync_status,
                    &self.input_handler.input_buffer,
                    self.input_handler.cursor_position,
                );
            }
            AppScreen::InputField(field_type) => {
                use crate::models::field_accessor::FieldType;
                match field_type {
                    // Numeric fields edit in place inside their daily-view row.
                    FieldType::Weight
                    | FieldType::Waist
                    | FieldType::Miles
                    | FieldType::Elevation => {
                        let edit = screens::InPlaceEdit {
                            field: field_type,
                            buffer: &self.input_handler.input_buffer,
                            cursor: self.input_handler.cursor_position,
                        };
                        screens::render_daily_view_screen(
                            f,
                            &self.state,
                            &mut self.food_list_state,
                            &mut self.sokay_list_state,
                            &self.sync_status,
                            Some(edit),
                            None,
                        );
                    }
                    FieldType::StrengthMobility => screens::render_edit_strength_mobility_screen(
                        f,
                        &self.state,
                        &mut self.food_list_state,
                        &mut self.sokay_list_state,
                        &self.sync_status,
                        &self.input_handler.input_buffer,
                        self.input_handler.cursor_position,
                    ),
                    FieldType::Notes => screens::render_edit_notes_screen(
                        f,
                        &self.state,
                        &mut self.food_list_state,
                        &mut self.sokay_list_state,
                        &self.sync_status,
                        &self.input_handler.input_buffer,
                        self.input_handler.cursor_position,
                    ),
                }
            }
            AppScreen::ConfirmDelete(target) => {
                use crate::models::DeleteTarget;
                match target {
                    DeleteTarget::Day => {
                        screens::render_confirm_delete_day_screen(f, self.state.selected_date);
                    }
                    DeleteTarget::Food(food_index) => {
                        screens::render_confirm_delete_food_screen(
                            f,
                            &self.state,
                            &mut self.food_list_state,
                            &mut self.sokay_list_state,
                            &self.sync_status,
                            food_index,
                        );
                    }
                    DeleteTarget::Sokay(sokay_index) => {
                        screens::render_confirm_delete_sokay_screen(
                            f,
                            &self.state,
                            &mut self.food_list_state,
                            &mut self.sokay_list_state,
                            &self.sync_status,
                            sokay_index,
                        );
                    }
                }
            }
            AppScreen::DateInput => {
                screens::render_date_input_screen(
                    f,
                    &self.state,
                    &mut self.list_state,
                    &self.sync_status,
                    &self.input_handler.input_buffer,
                    self.input_handler.cursor_position,
                );
            }
            AppScreen::ShortcutsHelp => {
                screens::render_shortcuts_help_screen(
                    f,
                    &self.state,
                    &mut self.food_list_state,
                    &mut self.sokay_list_state,
                    &self.sync_status,
                );
            }
            AppScreen::ConfigSync => {
                screens::render_config_sync_screen(
                    f,
                    &self.state,
                    &self.config_url_buffer,
                    &self.config_token_buffer,
                    self.config_sync_enabled,
                    !self.config.sync.auth_token.is_empty(),
                    Some(&mut self.click_targets),
                );
            }
            AppScreen::Syncing => {
                screens::render_syncing_screen(f, &self.sync_status);
            }
        }
    }
}
