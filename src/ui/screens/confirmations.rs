use chrono::NaiveDate;
use ratatui::{
    Frame,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, ListState, Paragraph},
};

use super::daily_view::render_daily_view_screen;
use crate::models::AppState;
use crate::ui::components::{centered_rect, create_standard_layout, render_help, render_title};

/// Renders the delete day confirmation screen
pub fn render_confirm_delete_day_screen(f: &mut Frame, selected_date: NaiveDate) {
    let chunks = create_standard_layout(f.area());

    let title = "Delete Day - Confirmation Required";
    render_title(f, chunks[0], title);

    let warning_text = format!(
        "Are you sure you want to delete the entire log for {}?\n\n\
        This will permanently delete:\n\
        - All food entries\n\
        - All sokay entries\n\
        - All measurements (weight, waist size, miles, elevation)\n\
        - Strength & mobility exercises\n\
        - Daily notes\n\n\
        This action cannot be undone.\n\n\
        Type 'y' to confirm deletion or 'n' to cancel.",
        selected_date.format("%B %d, %Y")
    );

    let warning_widget = Paragraph::new(warning_text)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red))
                .title("Warning: Permanent Deletion")
                .padding(ratatui::widgets::Padding::new(1, 0, 1, 0)),
        )
        .wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(warning_widget, chunks[1]);

    render_help(
        f,
        chunks[2],
        &["y: Delete Day | n/Esc: Cancel"],
        true,
        false,
    );
}

/// Renders the delete food item confirmation dialog as a centered modal
pub fn render_confirm_delete_food_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    food_index: usize,
) {
    render_daily_view_screen(
        f,
        state,
        food_list_state,
        sokay_list_state,
        sync_status,
        None,
        None,
    );

    let food_name = if let Some(log) = state.get_daily_log(state.selected_date) {
        if food_index < log.food_entries.len() {
            log.food_entries[food_index].name.clone()
        } else {
            "Unknown".to_string()
        }
    } else {
        "Unknown".to_string()
    };

    let popup_area = centered_rect(f.area(), 60, 20);

    f.render_widget(Clear, popup_area);

    let message = format!(
        "Delete this food item?\n\n\
        \"{}\"\n\n\
        Press 'y' to confirm or 'n' to cancel.",
        food_name
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title("Confirm Deletion")
        .padding(ratatui::widgets::Padding::uniform(1));

    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let text = Paragraph::new(message)
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(text, inner_area);
}

/// Renders the delete sokay item confirmation dialog as a centered modal
pub fn render_confirm_delete_sokay_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    sokay_index: usize,
) {
    render_daily_view_screen(
        f,
        state,
        food_list_state,
        sokay_list_state,
        sync_status,
        None,
        None,
    );

    let sokay_text = if let Some(log) = state.get_daily_log(state.selected_date) {
        if sokay_index < log.sokay_entries.len() {
            log.sokay_entries[sokay_index].clone()
        } else {
            "Unknown".to_string()
        }
    } else {
        "Unknown".to_string()
    };

    let popup_area = centered_rect(f.area(), 60, 20);

    f.render_widget(Clear, popup_area);

    let message = format!(
        "Delete this sokay item?\n\n\
        \"{}\"\n\n\
        Press 'y' to confirm or 'n' to cancel.",
        sokay_text
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title("Confirm Deletion")
        .padding(ratatui::widgets::Padding::uniform(1));

    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let text = Paragraph::new(message)
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(text, inner_area);
}
