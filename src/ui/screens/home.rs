use ratatui::{
    Frame,
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::models::AppState;
use crate::ui::components::{
    create_standard_layout, render_help, render_title, selectable_list_item,
};
use crate::ui::{ClickAction, ClickTarget};

/// Renders the home screen showing all available daily logs
pub fn render_home_screen(
    f: &mut Frame,
    state: &AppState,
    list_state: &mut ListState,
    sync_status: &str,
    click_targets: Option<&mut Vec<ClickTarget>>,
) {
    let chunks = create_standard_layout(f.area());

    // Render title with sync status
    let title = format!("Mountains - A Trail Running Training Log {}", sync_status);
    render_title(f, chunks[0], &title);

    // Create the list of daily logs
    let items: Vec<ListItem> = if state.daily_logs.is_empty() {
        vec![ListItem::new(
            "No training logs yet. Press Enter to create one for today.",
        )]
    } else {
        state
            .daily_logs
            .iter()
            .enumerate()
            .map(|(index, log)| {
                let date_str = log.date.format("%B %d, %Y").to_string();
                selectable_list_item(date_str, list_state.selected() == Some(index))
            })
            .collect()
    };

    // Create the List widget with styling
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Daily Training Logs")
        .padding(ratatui::widgets::Padding::uniform(1));
    let list_inner = block.inner(chunks[1]);
    let list = List::new(items).block(block);

    f.render_stateful_widget(list, chunks[1], list_state);

    if let Some(click_targets) = click_targets {
        let first_visible = list_state.offset();
        for row in 0..list_inner.height as usize {
            let index = first_visible + row;
            if index >= state.daily_logs.len() {
                break;
            }
            click_targets.push(ClickTarget::new(
                ratatui::layout::Rect::new(
                    list_inner.x,
                    list_inner.y + row as u16,
                    list_inner.width,
                    1,
                ),
                ClickAction::OpenLog(index),
            ));
        }
    }

    // Render help text
    render_help(
        f,
        chunks[2],
        &[
            " ↑/k: Up | ↓/j: Down | Enter: Select/Today | a: Add Date | Esc: Unfocus | d: Delete Day | S: Startup Screen | q: Quit",
            " ↑/k: Up | ↓/j: Down | Enter: Select | a: Add | Esc: Unfocus | d: Delete | S: Startup | q: Quit",
            " ↑↓/jk: Move | Enter: Select | a: Add | d: Delete | S: Startup | q: Quit",
            " jk: Move | Enter: Select | a: Add | q: Quit",
        ],
        true,
        false,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use ratatui::{Terminal, backend::TestBackend, style::Modifier};

    #[test]
    fn click_targets_follow_the_stateful_lists_scroll_offset() {
        let mut state = AppState::new();
        state.daily_logs = (1..=10)
            .rev()
            .map(|day| crate::models::DailyLog::new(NaiveDate::from_ymd_opt(2026, 7, day).unwrap()))
            .collect();
        let mut list_state = ListState::default();
        list_state.select(Some(7));
        let backend = TestBackend::new(80, 16);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut targets = Vec::new();

        terminal
            .draw(|frame| {
                render_home_screen(frame, &state, &mut list_state, "", Some(&mut targets));
            })
            .unwrap();

        let indices: Vec<usize> = targets
            .iter()
            .filter_map(|target| match target.action {
                ClickAction::OpenLog(index) => Some(index),
                _ => None,
            })
            .collect();
        assert!(!indices.is_empty());
        assert_eq!(indices[0], list_state.offset());
        assert_eq!(
            indices.last().copied(),
            Some(list_state.offset() + indices.len() - 1)
        );
    }

    #[test]
    fn empty_list_placeholder_is_not_clickable() {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut list_state = ListState::default();
        let mut targets = Vec::new();

        terminal
            .draw(|frame| {
                render_home_screen(
                    frame,
                    &AppState::new(),
                    &mut list_state,
                    "",
                    Some(&mut targets),
                );
            })
            .unwrap();

        assert!(targets.is_empty());
    }

    #[test]
    fn selection_highlight_hugs_the_row_text() {
        let mut state = AppState::new();
        state.daily_logs = (1..=3)
            .rev()
            .map(|day| crate::models::DailyLog::new(NaiveDate::from_ymd_opt(2026, 7, day).unwrap()))
            .collect();
        let mut list_state = ListState::default();
        list_state.select(Some(1));
        let backend = TestBackend::new(80, 16);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                render_home_screen(frame, &state, &mut list_state, "", None);
            })
            .unwrap();

        let highlighted = reversed_cells(terminal.backend().buffer());
        let rows: std::collections::BTreeSet<u16> = highlighted.iter().map(|&(_, y)| y).collect();
        assert_eq!(rows.len(), 1, "only the selected row should be highlighted");
        assert_eq!(
            highlighted.len(),
            "July 02, 2026".len() + 1,
            "highlight should cover the text plus a single trailing space"
        );
    }

    fn reversed_cells(buffer: &ratatui::buffer::Buffer) -> Vec<(u16, u16)> {
        let area = buffer.area;
        let mut cells = Vec::new();
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if let Some(cell) = buffer.cell((x, y))
                    && cell.modifier.contains(Modifier::REVERSED)
                {
                    cells.push((x, y));
                }
            }
        }
        cells
    }
}
