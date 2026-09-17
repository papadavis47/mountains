use super::*;
use crate::models::field_accessor::FieldType;

impl App {
    pub(super) async fn handle_add_food_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Enter => {
                let saved = ActionHandler::save_food_entry(
                    &mut self.state,
                    self.input_handler.input_buffer.clone(),
                );
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;
                if let Some(log) = saved {
                    // Land on the entry just added: `e`/`d` act on it without
                    // first pressing `j`. An empty save selects nothing.
                    self.state.food_list_focused = true;
                    self.food_list_state
                        .select(Some(log.food_entries.len() - 1));
                    self.spawn_persist(log);
                }
            }
            KeyCode::Esc => {
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;
            }
            _ => {
                self.input_handler.handle_text_input(key);
            }
        }
        Ok(())
    }

    pub(super) async fn handle_edit_food_input(
        &mut self,
        key: KeyCode,
        food_index: usize,
    ) -> Result<()> {
        match key {
            KeyCode::Enter => {
                if let Some(log) = ActionHandler::update_food_entry(
                    &mut self.state,
                    food_index,
                    self.input_handler.input_buffer.clone(),
                ) {
                    self.input_handler.clear();
                    self.state.current_screen = AppScreen::DailyView;

                    self.spawn_persist(log);
                } else {
                    self.input_handler.clear();
                    self.state.current_screen = AppScreen::DailyView;
                }
            }
            KeyCode::Esc => {
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;
            }
            _ => {
                self.input_handler.handle_text_input(key);
            }
        }
        Ok(())
    }

    /// Generic handler for all field inputs - consolidates 6 separate handlers
    pub(super) async fn handle_field_input(
        &mut self,
        key: KeyCode,
        modifiers: crossterm::event::KeyModifiers,
        field_type: crate::models::field_accessor::FieldType,
    ) -> Result<()> {
        match key {
            KeyCode::Enter => {
                let is_multiline =
                    matches!(field_type, FieldType::StrengthMobility | FieldType::Notes);
                // Use Alt modifier for newline insertion (most reliable across terminals)
                let has_alt = modifiers.contains(crossterm::event::KeyModifiers::ALT);

                // Alt+Enter in multiline inputs inserts newline, regular Enter saves
                if is_multiline && has_alt {
                    // Insert newline and stay in edit mode
                    self.input_handler.insert_newline();
                } else {
                    let entered = !self.input_handler.input_buffer.trim().is_empty();
                    // Reached by Enter after Shift+J/K, entering data moves focus
                    // to the next field so a pass over the day flows top-to-bottom
                    // without manual Shift+J. A field opened by its own shortcut is
                    // a targeted edit, so focus stays on it; so does an empty save.
                    // Focus-only — the next field isn't auto-opened.
                    let advance = entered && self.state.edit_origin == EditOrigin::Navigation;
                    let next_focus = if advance {
                        SectionNavigator::advance_field(field_type, self.state.simple_mode)
                    } else {
                        SectionNavigator::field_section(field_type)
                    };
                    self.save_field_and_focus(field_type, next_focus);
                }
            }
            KeyCode::Tab
                if matches!(
                    field_type,
                    FieldType::Weight | FieldType::Waist | FieldType::Miles | FieldType::Elevation
                ) =>
            {
                let current_focus = SectionNavigator::field_section(field_type);
                let paired_focus = SectionNavigator::toggle_internal_focus(&current_focus);
                self.save_field_and_focus(field_type, paired_focus);
            }
            KeyCode::Esc => {
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;
            }
            _ => match field_type {
                FieldType::Weight | FieldType::Waist | FieldType::Miles => {
                    self.input_handler.handle_numeric_input(key);
                }
                FieldType::Elevation => {
                    self.input_handler.handle_integer_input(key);
                }
                FieldType::StrengthMobility | FieldType::Notes => {
                    self.input_handler
                        .handle_multiline_text_input(key, modifiers);
                }
            },
        }
        Ok(())
    }

    fn save_field_and_focus(&mut self, field_type: FieldType, next_focus: FocusedSection) {
        let log = ActionHandler::update_field(
            &mut self.state,
            field_type,
            self.input_handler.input_buffer.clone(),
        );
        self.input_handler.clear();
        // Unconditional, unlike `focus_section`: the text just changed, so an old
        // offset into it could point past the new end.
        self.state.strength_mobility_scroll = 0;
        self.state.notes_scroll = 0;
        self.state.focused_section = next_focus;
        self.state.current_screen = AppScreen::DailyView;
        self.spawn_persist(log);
    }

    pub(super) async fn handle_add_sokay_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Enter => {
                let saved = ActionHandler::save_sokay_entry(
                    &mut self.state,
                    self.input_handler.input_buffer.clone(),
                );
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;
                if let Some(log) = saved {
                    self.state.sokay_list_focused = true;
                    self.sokay_list_state
                        .select(Some(log.sokay_entries.len() - 1));
                    self.spawn_persist(log);
                }
            }
            KeyCode::Esc => {
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;
            }
            _ => {
                self.input_handler.handle_text_input(key);
            }
        }
        Ok(())
    }

    pub(super) async fn handle_edit_sokay_input(
        &mut self,
        key: KeyCode,
        sokay_index: usize,
    ) -> Result<()> {
        match key {
            KeyCode::Enter => {
                if let Some(log) = ActionHandler::update_sokay_entry(
                    &mut self.state,
                    sokay_index,
                    self.input_handler.input_buffer.clone(),
                ) {
                    self.input_handler.clear();
                    self.state.current_screen = AppScreen::DailyView;

                    self.spawn_persist(log);
                } else {
                    self.input_handler.clear();
                    self.state.current_screen = AppScreen::DailyView;
                }
            }
            KeyCode::Esc => {
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;
            }
            _ => {
                self.input_handler.handle_text_input(key);
            }
        }
        Ok(())
    }

    pub(super) async fn handle_date_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Enter => {
                let input = self.input_handler.input_buffer.clone();
                match chrono::NaiveDate::parse_from_str(&input, "%m.%d.%Y") {
                    Ok(date) => {
                        let today = chrono::Local::now().date_naive();
                        if date > today {
                            self.state.date_input_error =
                                Some("Future dates not allowed".to_string());
                        } else {
                            self.input_handler.clear();
                            self.state.date_input_error = None;
                            self.state.selected_date = date;
                            self.state.get_or_create_daily_log(date);
                            self.state.current_screen = AppScreen::DailyView;
                        }
                    }
                    Err(_) => {
                        self.state.date_input_error = Some("Invalid date format".to_string());
                    }
                }
            }
            KeyCode::Esc => {
                self.input_handler.clear();
                self.state.date_input_error = None;
                self.state.current_screen = AppScreen::Home;
            }
            KeyCode::Char(c) => {
                if c.is_ascii_digit() || c == '.' {
                    self.state.date_input_error = None;
                    self.input_handler.handle_text_input(key);
                }
            }
            _ => {
                self.state.date_input_error = None;
                self.input_handler.handle_text_input(key);
            }
        }
        Ok(())
    }

    pub(super) async fn handle_config_sync_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Tab => {
                let next = match self.state.config_sync_focused_field {
                    ConfigSyncField::DbUrl => ConfigSyncField::AuthToken,
                    ConfigSyncField::AuthToken => ConfigSyncField::EnableToggle,
                    ConfigSyncField::EnableToggle => ConfigSyncField::DbUrl,
                };
                self.focus_config_sync_field(next);
            }
            KeyCode::Enter => {
                // Save current field buffer
                match self.state.config_sync_focused_field {
                    ConfigSyncField::DbUrl => {
                        self.config_url_buffer = self.input_handler.input_buffer.clone();
                    }
                    ConfigSyncField::AuthToken => {
                        self.config_token_buffer = self.input_handler.input_buffer.clone();
                    }
                    ConfigSyncField::EnableToggle => {}
                }

                // Build updated config
                let token = if self.config_token_buffer.is_empty() {
                    self.config.sync.auth_token.clone()
                } else {
                    self.config_token_buffer.clone()
                };

                self.config.sync.db_url = self.config_url_buffer.clone();
                self.config.sync.auth_token = token;
                self.config.sync.enabled = self.config_sync_enabled;

                match self.config.save() {
                    Ok(()) => {
                        self.state.config_sync_status = Some("Saved!".to_string());
                    }
                    Err(e) => {
                        self.state.config_sync_status = Some(format!("Error: {}", e));
                        return Ok(());
                    }
                }

                // If newly configured, spawn background cloud connection
                if self.config.sync.is_configured() {
                    let db_manager_clone = Arc::clone(&self.db_manager);
                    let home_dir = dirs::home_dir().context("Could not find home directory")?;
                    let mountains_dir = home_dir.join(".mountains");
                    let url = self.config.sync.db_url.clone();
                    let token = self.config.sync.auth_token.clone();
                    tokio::spawn(async move {
                        let db_path = mountains_dir.join("mountains.db");
                        if let Some(db_path_str) = db_path.to_str() {
                            let mut db = db_manager_clone.write().await;
                            let _ = db.upgrade_to_remote_replica(db_path_str, url, token).await;
                        }
                    });
                }

                self.input_handler.clear();
                self.state.current_screen = AppScreen::Startup;
            }
            KeyCode::Esc => {
                self.input_handler.clear();
                self.state.config_sync_status = None;
                self.state.current_screen = AppScreen::Startup;
            }
            _ => match self.state.config_sync_focused_field {
                ConfigSyncField::DbUrl | ConfigSyncField::AuthToken => {
                    self.input_handler.handle_text_input(key);
                }
                ConfigSyncField::EnableToggle => {
                    if matches!(key, KeyCode::Char(' ')) {
                        self.config_sync_enabled = !self.config_sync_enabled;
                    }
                }
            },
        }
        Ok(())
    }

    /// Generic handler for all delete confirmations - consolidates 3 separate handlers
    pub(super) async fn handle_delete_confirmation_input(
        &mut self,
        key: KeyCode,
        target: crate::models::DeleteTarget,
    ) -> Result<()> {
        use crate::models::DeleteTarget;

        match key {
            KeyCode::Char('y') => match target {
                DeleteTarget::Day => {
                    let date_to_delete = self.state.selected_date;
                    {
                        let mut db = self.db_manager.write().await;
                        ActionHandler::delete_daily_log(
                            &mut self.state,
                            &mut db,
                            &self.file_manager,
                            date_to_delete,
                        )
                        .await?;
                    }
                    self.spawn_sync();
                    self.state.current_screen = AppScreen::Home;
                    self.list_state.select(None);
                }
                DeleteTarget::Food(food_index) => {
                    if let Some(log) = ActionHandler::delete_food_entry(&mut self.state, food_index)
                    {
                        if let Some(current_log) =
                            self.state.get_daily_log(self.state.selected_date)
                        {
                            if current_log.food_entries.is_empty() {
                                self.food_list_state.select(None);
                            } else if food_index >= current_log.food_entries.len() {
                                self.food_list_state
                                    .select(Some(current_log.food_entries.len() - 1));
                            }
                        }
                        self.state.current_screen = AppScreen::DailyView;

                        self.spawn_persist(log);
                    } else {
                        self.state.current_screen = AppScreen::DailyView;
                    }
                }
                DeleteTarget::Sokay(sokay_index) => {
                    if let Some(log) =
                        ActionHandler::delete_sokay_entry(&mut self.state, sokay_index)
                    {
                        if let Some(current_log) =
                            self.state.get_daily_log(self.state.selected_date)
                        {
                            if current_log.sokay_entries.is_empty() {
                                self.sokay_list_state.select(None);
                            } else if sokay_index >= current_log.sokay_entries.len() {
                                self.sokay_list_state
                                    .select(Some(current_log.sokay_entries.len() - 1));
                            }
                        }
                        self.state.current_screen = AppScreen::DailyView;

                        self.spawn_persist(log);
                    } else {
                        self.state.current_screen = AppScreen::DailyView;
                    }
                }
            },
            KeyCode::Char('n') | KeyCode::Esc => match target {
                DeleteTarget::Day => {
                    self.state.current_screen = AppScreen::Home;
                }
                DeleteTarget::Food(_) | DeleteTarget::Sokay(_) => {
                    self.state.current_screen = AppScreen::DailyView;
                }
            },
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_support::test_app;
    use crossterm::event::KeyModifiers;
    use tempfile::TempDir;

    #[tokio::test]
    async fn tab_saves_numeric_fields_and_toggles_within_each_pair() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir).await;
        let cases = [
            (
                FieldType::Weight,
                "175.5",
                FocusedSection::Measurements {
                    focused_field: MeasurementField::Waist,
                },
            ),
            (
                FieldType::Waist,
                "34.25",
                FocusedSection::Measurements {
                    focused_field: MeasurementField::Weight,
                },
            ),
            (
                FieldType::Miles,
                "7.2",
                FocusedSection::Running {
                    focused_field: RunningField::Elevation,
                },
            ),
            (
                FieldType::Elevation,
                "1400",
                FocusedSection::Running {
                    focused_field: RunningField::Miles,
                },
            ),
        ];

        for (field, input, expected_focus) in cases {
            app.state.current_screen = AppScreen::InputField(field);
            app.input_handler.set_input(input.to_string());

            app.handle_field_input(KeyCode::Tab, KeyModifiers::NONE, field)
                .await
                .unwrap();

            assert_eq!(field.get_value(&app.state), input);
            assert_eq!(app.state.focused_section, expected_focus);
            assert!(matches!(app.state.current_screen, AppScreen::DailyView));
            assert!(app.input_handler.input_buffer.is_empty());
        }
    }

    /// Entering a field by Enter is a top-to-bottom entry pass, so saving steps
    /// to the next field. Contrast `a_shortcut_opened_field_keeps_focus_after_saving`.
    #[tokio::test]
    async fn enter_keeps_advancing_to_the_next_field() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir).await;
        let date = app.state.selected_date;
        app.state.get_or_create_daily_log(date);
        app.state.current_screen = AppScreen::DailyView;
        app.state.focused_section = FocusedSection::Measurements {
            focused_field: MeasurementField::Waist,
        };

        app.handle_key_event_with_modifiers(KeyCode::Enter, KeyModifiers::NONE)
            .await
            .unwrap();
        app.input_handler.set_input("34.25".to_string());
        app.handle_key_event_with_modifiers(KeyCode::Enter, KeyModifiers::NONE)
            .await
            .unwrap();

        assert_eq!(FieldType::Waist.get_value(&app.state), "34.25");
        assert_eq!(
            app.state.focused_section,
            FocusedSection::Running {
                focused_field: RunningField::Miles,
            }
        );
        assert!(matches!(app.state.current_screen, AppScreen::DailyView));
    }

    /// A field opened by its own shortcut is a targeted edit, so saving leaves
    /// focus on that field rather than stepping to the next one.
    #[tokio::test]
    async fn a_shortcut_opened_field_keeps_focus_after_saving() {
        let dir = TempDir::new().unwrap();
        let cases = [
            (
                'w',
                FieldType::Weight,
                "175.5",
                FocusedSection::Measurements {
                    focused_field: MeasurementField::Weight,
                },
            ),
            (
                's',
                FieldType::Waist,
                "34.25",
                FocusedSection::Measurements {
                    focused_field: MeasurementField::Waist,
                },
            ),
            (
                'm',
                FieldType::Miles,
                "7.2",
                FocusedSection::Running {
                    focused_field: RunningField::Miles,
                },
            ),
            (
                'l',
                FieldType::Elevation,
                "1400",
                FocusedSection::Running {
                    focused_field: RunningField::Elevation,
                },
            ),
            (
                't',
                FieldType::StrengthMobility,
                "squats",
                FocusedSection::StrengthMobility,
            ),
            ('n', FieldType::Notes, "felt strong", FocusedSection::Notes),
        ];

        for (key, field, input, expected_focus) in cases {
            let mut app = test_app(&dir).await;
            let date = app.state.selected_date;
            app.state.get_or_create_daily_log(date);
            app.state.current_screen = AppScreen::DailyView;
            app.state.focused_section = FocusedSection::FoodItems;

            app.handle_key_event_with_modifiers(KeyCode::Char(key), KeyModifiers::NONE)
                .await
                .unwrap();
            app.input_handler.set_input(input.to_string());
            app.handle_key_event_with_modifiers(KeyCode::Enter, KeyModifiers::NONE)
                .await
                .unwrap();

            assert_eq!(field.get_value(&app.state), input, "'{key}' value");
            assert_eq!(app.state.focused_section, expected_focus, "'{key}' focus");
            assert!(matches!(app.state.current_screen, AppScreen::DailyView));
        }
    }

    #[tokio::test]
    async fn a_shortcut_opened_field_keeps_focus_after_escaping() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir).await;
        let date = app.state.selected_date;
        app.state.get_or_create_daily_log(date);
        app.state.current_screen = AppScreen::DailyView;
        app.state.focused_section = FocusedSection::Notes;

        app.handle_key_event_with_modifiers(KeyCode::Char('w'), KeyModifiers::NONE)
            .await
            .unwrap();
        app.handle_key_event_with_modifiers(KeyCode::Esc, KeyModifiers::NONE)
            .await
            .unwrap();

        assert_eq!(
            app.state.focused_section,
            FocusedSection::Measurements {
                focused_field: MeasurementField::Weight,
            }
        );
        assert!(matches!(app.state.current_screen, AppScreen::DailyView));
    }

    #[tokio::test]
    async fn saving_a_new_food_entry_selects_it_in_the_list() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir).await;
        let date = app.state.selected_date;
        app.state.get_or_create_daily_log(date);
        app.state.current_screen = AppScreen::DailyView;

        for name in ["Eggs", "Toast"] {
            app.handle_key_event_with_modifiers(KeyCode::Char('f'), KeyModifiers::NONE)
                .await
                .unwrap();
            app.input_handler.set_input(name.to_string());
            app.handle_key_event_with_modifiers(KeyCode::Enter, KeyModifiers::NONE)
                .await
                .unwrap();
        }

        assert_eq!(app.state.focused_section, FocusedSection::FoodItems);
        assert!(app.state.food_list_focused);
        assert_eq!(app.food_list_state.selected(), Some(1));
    }

    #[tokio::test]
    async fn saving_a_new_sokay_entry_selects_it_in_the_list() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir).await;
        let date = app.state.selected_date;
        app.state.get_or_create_daily_log(date);
        app.state.current_screen = AppScreen::DailyView;

        app.handle_key_event_with_modifiers(KeyCode::Char('c'), KeyModifiers::NONE)
            .await
            .unwrap();
        app.input_handler.set_input("donut".to_string());
        app.handle_key_event_with_modifiers(KeyCode::Enter, KeyModifiers::NONE)
            .await
            .unwrap();

        assert_eq!(app.state.focused_section, FocusedSection::Sokay);
        assert!(app.state.sokay_list_focused);
        assert_eq!(app.sokay_list_state.selected(), Some(0));
    }

    #[tokio::test]
    async fn an_empty_save_leaves_the_food_list_unselected() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir).await;
        let date = app.state.selected_date;
        app.state.get_or_create_daily_log(date);
        app.state.current_screen = AppScreen::DailyView;

        app.handle_key_event_with_modifiers(KeyCode::Char('f'), KeyModifiers::NONE)
            .await
            .unwrap();
        app.handle_key_event_with_modifiers(KeyCode::Enter, KeyModifiers::NONE)
            .await
            .unwrap();

        assert_eq!(app.state.focused_section, FocusedSection::FoodItems);
        assert!(!app.state.food_list_focused);
        assert_eq!(app.food_list_state.selected(), None);
    }

    /// Simple mode keeps its own traversal (Miles → Elevation → Food → Notes →
    /// Miles) for the Enter-driven pass; only the direct-shortcut route stays put.
    #[tokio::test]
    async fn simple_mode_advances_on_enter_but_stays_put_on_a_shortcut() {
        let dir = TempDir::new().unwrap();

        let mut app = test_app(&dir).await;
        app.apply_launch_mode(crate::models::LaunchMode::Simple);
        app.state.focused_section = FocusedSection::Notes;
        app.handle_key_event_with_modifiers(KeyCode::Enter, KeyModifiers::NONE)
            .await
            .unwrap();
        app.input_handler.set_input("long climb".to_string());
        app.handle_key_event_with_modifiers(KeyCode::Enter, KeyModifiers::NONE)
            .await
            .unwrap();
        assert_eq!(
            app.state.focused_section,
            FocusedSection::Running {
                focused_field: RunningField::Miles,
            },
            "Enter from Notes wraps to Miles in simple mode"
        );

        let mut app = test_app(&dir).await;
        app.apply_launch_mode(crate::models::LaunchMode::Simple);
        app.handle_key_event_with_modifiers(KeyCode::Char('n'), KeyModifiers::NONE)
            .await
            .unwrap();
        app.input_handler.set_input("long climb".to_string());
        app.handle_key_event_with_modifiers(KeyCode::Enter, KeyModifiers::NONE)
            .await
            .unwrap();
        assert_eq!(
            app.state.focused_section,
            FocusedSection::Notes,
            "'n' is a targeted edit, so focus stays on Notes"
        );
    }
}
