use chrono::NaiveDate;
use ratatui::{
    Frame,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, ListState, Paragraph},
};

use super::daily_view::render_daily_view_screen;
use crate::models::{AppState, Marks, RowList};
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

/// Most marked items listed by name before "...and N more"
const MARKED_LIST_LIMIT: usize = 8;

/// Bullet lines for `items`, capped at `MARKED_LIST_LIMIT`
fn marked_item_lines(items: &[String]) -> String {
    let mut lines: Vec<String> = items
        .iter()
        .take(MARKED_LIST_LIMIT)
        .map(|item| format!("- {item}"))
        .collect();
    if items.len() > MARKED_LIST_LIMIT {
        lines.push(format!("...and {} more", items.len() - MARKED_LIST_LIMIT));
    }
    lines.join("\n")
}

/// Renders the one-confirmation dialog for everything in `state.marks`
pub fn render_confirm_delete_marked_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
) {
    match &state.marks {
        Marks::None => {}
        Marks::Logs(dates) => {
            // Newest first, matching the log list
            let names: Vec<String> = dates
                .iter()
                .rev()
                .map(|date| date.format("%B %d, %Y").to_string())
                .collect();
            render_confirm_delete_days(f, &names);
        }
        Marks::Rows { list, date, rows } => {
            render_daily_view_screen(
                f,
                state,
                food_list_state,
                sokay_list_state,
                sync_status,
                None,
                None,
            );
            let log = state.get_daily_log(*date);
            let (noun, names): (&str, Vec<String>) = match list {
                RowList::Food => (
                    "food",
                    rows.iter()
                        .filter_map(|&i| log?.food_entries.get(i).map(|e| e.name.clone()))
                        .collect(),
                ),
                RowList::Sokay => (
                    "sokay",
                    rows.iter()
                        .filter_map(|&i| log?.sokay_entries.get(i).cloned())
                        .collect(),
                ),
            };
            let count = state.marks.count();
            let plural = if count == 1 { "item" } else { "items" };
            let message = format!(
                "Delete {count} {noun} {plural}?\n\n{}\n\nPress 'y' to confirm or 'n' to cancel.",
                marked_item_lines(&names)
            );

            let popup_area = centered_rect(f.area(), 60, 50);
            f.render_widget(Clear, popup_area);
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
    }
}

/// Full-screen warning for deleting several whole days
fn render_confirm_delete_days(f: &mut Frame, dates: &[String]) {
    let chunks = create_standard_layout(f.area());
    render_title(f, chunks[0], "Delete Days - Confirmation Required");

    let warning_text = format!(
        "Are you sure you want to delete the entire log for these {} days?\n\n\
        {}\n\n\
        This will permanently delete every entry, measurement, and note on them.\n\
        This action cannot be undone.\n\n\
        Type 'y' to confirm deletion or 'n' to cancel.",
        dates.len(),
        marked_item_lines(dates)
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
        &["y: Delete Days | n/Esc: Cancel"],
        true,
        false,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FoodEntry;
    use chrono::NaiveDate;
    use ratatui::{Terminal, backend::TestBackend};

    fn screen(state: &AppState) -> String {
        let mut term = Terminal::new(TestBackend::new(100, 40)).unwrap();
        term.draw(|f| {
            render_confirm_delete_marked_screen(
                f,
                state,
                &mut ListState::default(),
                &mut ListState::default(),
                "",
            )
        })
        .unwrap();
        term.backend()
            .buffer()
            .content()
            .chunks(100)
            .map(|row| row.iter().map(|c| c.symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn marked_food_dialog_names_each_item() {
        let mut state = AppState::new();
        let date = NaiveDate::from_ymd_opt(2026, 11, 1).unwrap();
        state.selected_date = date;
        let log = state.get_or_create_daily_log(date);
        for name in ["Eggs", "Toast", "Jam"] {
            log.add_food_entry(FoodEntry::new(name.to_string()));
        }
        state.marks.toggle_row(RowList::Food, date, 0);
        state.marks.toggle_row(RowList::Food, date, 2);
        let s = screen(&state);
        assert!(s.contains("Delete 2 food items?"));
        assert!(s.contains("- Eggs"));
        assert!(s.contains("- Jam"));
        // The list behind the modal shows marked rows with the bullet
        assert!(s.contains("● Eggs"));
        assert!(!s.contains("● Toast"));
    }

    #[test]
    fn marked_days_dialog_lists_dates_newest_first() {
        let mut state = AppState::new();
        for d in [3, 1] {
            state
                .marks
                .toggle_log(NaiveDate::from_ymd_opt(2026, 11, d).unwrap());
        }
        let s = screen(&state);
        assert!(s.contains("these 2 days"));
        let first = s.find("November 03, 2026").unwrap();
        let second = s.find("November 01, 2026").unwrap();
        assert!(first < second);
    }

    #[test]
    fn long_mark_lists_are_capped() {
        let items: Vec<String> = (0..11).map(|i| format!("item{i}")).collect();
        let lines = marked_item_lines(&items);
        assert!(lines.contains("item7"));
        assert!(!lines.contains("item8"));
        assert!(lines.ends_with("...and 3 more"));
    }
}
