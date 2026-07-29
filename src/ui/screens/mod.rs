pub mod config_sync;
pub mod confirmations;
pub mod daily_view;
pub mod help;
pub mod home;
pub mod inputs;
pub mod startup;
pub mod statistics;

// Re-export all public functions for backward compatibility
pub use config_sync::render_config_sync_screen;
pub use confirmations::{
    render_confirm_delete_day_screen, render_confirm_delete_food_screen,
    render_confirm_delete_sokay_screen,
};
pub use daily_view::{InPlaceEdit, max_scroll_offset, render_daily_view_screen};
pub use help::{render_shortcuts_help_screen, render_syncing_screen};
pub use home::render_home_screen;
pub use inputs::{
    calculate_cursor_in_wrapped_text, render_add_food_screen, render_add_sokay_screen,
    render_date_input_screen, render_edit_food_screen, render_edit_notes_screen,
    render_edit_sokay_screen, render_edit_strength_mobility_screen, wrap_at_width,
};
pub use startup::render_startup_screen;
pub use statistics::render_statistics_screen;
