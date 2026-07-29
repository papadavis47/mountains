use anyhow::{Context, Result};
use crossterm::event::{Event, KeyCode, MouseEvent};
use ratatui::{Frame, Terminal, backend::CrosstermBackend, widgets::ListState};
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::RwLock;

use crate::config::AppConfig;
use crate::db_manager::{ConnectionState, DbManager};
use crate::events::handlers::{ActionHandler, InputHandler, NavigationHandler, SectionNavigator};
use crate::file_manager::FileManager;
use crate::models::{
    AppScreen, AppState, ConfigSyncField, DailyLog, FocusedSection, MeasurementField, RunningField,
};
use crate::ui::screens;
use crate::ui::{ClickAction, ClickTarget, hit_test, left_click_position};

pub struct App {
    state: AppState,
    config: AppConfig,
    db_manager: Arc<RwLock<DbManager>>,
    file_manager: FileManager,
    input_handler: InputHandler,
    list_state: ListState,
    food_list_state: ListState,
    sokay_list_state: ListState,
    should_quit: bool,
    sync_status: String,
    config_url_buffer: String,
    config_token_buffer: String,
    config_sync_enabled: bool,
    click_targets: Vec<ClickTarget>,
    /// Set by the background cloud-sync task after it pulls from the primary,
    /// signalling the event loop to reload the in-memory daily_logs cache.
    needs_reload: Arc<AtomicBool>,
}

// `App` is one type whose `impl` is spread across sibling submodules to keep each
// file focused: `render` draws the current screen, `click` maps mouse hits to
// actions, `input` handles text/modal entry, `navigation` handles key-driven
// movement and edits. Those files add more `impl App` blocks — legal because they
// are descendants of this module, so they can reach `App`'s private fields. Their
// methods are `pub(super)` (visible only within this `app` subtree) so parent and
// siblings can call across the split without exposing anything crate-wide. Core
// lifecycle methods (new/run/event dispatch/sync) stay here in the parent.
mod click;
mod input;
mod navigation;
mod render;

impl App {
    /// Creates app with instant startup, spawns background cloud sync if configured
    pub async fn new(config: AppConfig) -> Result<Self> {
        let mountains_dir = crate::config::data_dir()?;

        if !mountains_dir.exists() {
            std::fs::create_dir_all(&mountains_dir)
                .context("Failed to create .mountains directory")?;
        }

        let db_manager = DbManager::new_local_first(&mountains_dir).await?;
        let file_manager = FileManager::new()?;

        let mut state = AppState::new();
        state.daily_logs = db_manager.load_all_daily_logs().await?;

        let db_manager = Arc::new(RwLock::new(db_manager));
        let needs_reload = Arc::new(AtomicBool::new(false));

        // Spawn background cloud sync only if config has valid credentials
        if config.sync.is_configured() {
            let db_manager_clone = Arc::clone(&db_manager);
            let needs_reload_clone = Arc::clone(&needs_reload);
            let mountains_dir_clone = mountains_dir.clone();
            let url = config.sync.db_url.clone();
            let token = config.sync.auth_token.clone();
            tokio::spawn(async move {
                let db_path = mountains_dir_clone.join("mountains.db");
                if let Some(db_path_str) = db_path.to_str() {
                    let mut db = db_manager_clone.write().await;
                    if db
                        .upgrade_to_remote_replica(db_path_str, url, token)
                        .await
                        .is_ok()
                    {
                        // Local replica now holds the pulled rows; ask the loop to reload.
                        needs_reload_clone.store(true, Ordering::Release);
                    }
                }
            });
        }

        Ok(Self {
            state,
            config,
            db_manager,
            file_manager,
            input_handler: InputHandler::new(),
            list_state: ListState::default(),
            food_list_state: ListState::default(),
            sokay_list_state: ListState::default(),
            should_quit: false,
            sync_status: String::new(),
            config_url_buffer: String::new(),
            config_token_buffer: String::new(),
            config_sync_enabled: false,
            click_targets: Vec::new(),
            needs_reload,
        })
    }

    /// Main event loop
    pub async fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<()> {
        loop {
            self.update_sync_status().await;
            self.reload_logs_if_needed().await?;

            // Handle syncing screen
            if matches!(self.state.current_screen, AppScreen::Syncing) {
                terminal.draw(|f| self.ui(f))?;
                self.perform_shutdown_sync().await;
                terminal.draw(|f| self.ui(f))?;
                std::thread::sleep(Duration::from_millis(1000));
            }

            terminal.draw(|f| self.ui(f))?;

            if crossterm::event::poll(Duration::from_millis(100))? {
                match crossterm::event::read()? {
                    Event::Key(key) => {
                        self.handle_key_event_with_modifiers(key.code, key.modifiers)
                            .await?;
                    }
                    Event::Mouse(mouse) => self.handle_mouse_event(mouse),
                    _ => {}
                }
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    async fn handle_key_event_with_modifiers(
        &mut self,
        key: KeyCode,
        modifiers: crossterm::event::KeyModifiers,
    ) -> Result<()> {
        match self.state.current_screen {
            AppScreen::AddFood => self.handle_add_food_input(key).await?,
            AppScreen::EditFood(food_index) => self.handle_edit_food_input(key, food_index).await?,
            AppScreen::AddSokay => self.handle_add_sokay_input(key).await?,
            AppScreen::EditSokay(sokay_index) => {
                self.handle_edit_sokay_input(key, sokay_index).await?
            }
            AppScreen::InputField(field_type) => {
                self.handle_field_input(key, modifiers, field_type).await?;
            }
            AppScreen::ConfirmDelete(target) => {
                self.handle_delete_confirmation_input(key, target).await?;
            }
            AppScreen::DateInput => self.handle_date_input(key).await?,
            AppScreen::ConfigSync => self.handle_config_sync_input(key).await?,
            _ => self.handle_navigation_input(key, modifiers).await?,
        }
        Ok(())
    }

    /// Persists `log` to the database and its markdown backup on a detached task,
    /// so a keystroke that saves data never blocks the render loop on I/O. The DB
    /// handle is an `Arc`; cloning it bumps a refcount (cheap) rather than copying
    /// the manager, giving the spawned task an owned handle to move across threads.
    fn spawn_persist(&self, log: DailyLog) {
        let db_manager = Arc::clone(&self.db_manager);
        let file_manager = self.file_manager.clone();
        tokio::spawn(async move {
            ActionHandler::persist_daily_log(db_manager, &file_manager, log).await;
        });
    }

    /// Reloads the daily_logs cache from the local replica once the background
    /// cloud-sync task signals it has pulled new rows from the primary. Cheap
    /// no-op on every other iteration; the local read only runs when flagged.
    async fn reload_logs_if_needed(&mut self) -> Result<()> {
        if self.needs_reload.swap(false, Ordering::AcqRel) {
            let db = self.db_manager.read().await;
            self.state.daily_logs = db.load_all_daily_logs().await?;
        }
        Ok(())
    }

    async fn update_sync_status(&mut self) {
        let db = self.db_manager.read().await;
        let state = db.get_connection_state().await;

        self.sync_status = match state {
            ConnectionState::Disconnected => "⚪ Offline".to_string(),
            ConnectionState::Connected => "✓ Synced".to_string(),
            ConnectionState::Error(_) => "⚠️ Sync Error".to_string(),
        };
    }

    /// Performs shutdown sync and updates sync_status with result
    pub async fn perform_shutdown_sync(&mut self) {
        let db = self.db_manager.read().await;
        let connection_state = db.get_connection_state().await;

        match connection_state {
            ConnectionState::Connected => {
                self.sync_status = "Syncing with Turso Cloud...".to_string();
                drop(db);

                let db = self.db_manager.read().await;
                match db.sync_now().await {
                    Ok(_) => {
                        self.sync_status = "Sync complete!".to_string();
                    }
                    Err(_) => {
                        self.sync_status =
                            "Offline - changes will sync when network is available".to_string();
                    }
                }
            }
            _ => {
                self.sync_status =
                    "Offline - changes will sync when network is available".to_string();
            }
        }

        self.should_quit = true;
    }
}
