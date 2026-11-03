//! Multi-select for batch delete: Ctrl+click or `x` marks rows in the Food,
//! Sokay, or log list; `d` then asks once and deletes them all.

use super::*;
use crate::models::{DeleteTarget, Marks, RowList};

impl App {
    /// Ctrl+click on a list row: mark/unmark it without opening or editing.
    /// Returns false when `action` isn't a markable row on this screen.
    pub(super) fn handle_mark_click(&mut self, action: &ClickAction) -> bool {
        match (action, &self.state.current_screen) {
            (&ClickAction::SelectFood(index), AppScreen::DailyView) => {
                self.focus_section(FocusedSection::FoodItems);
                self.state.food_list_focused = true;
                self.food_list_state.select(Some(index));
            }
            (&ClickAction::SelectSokay(index), AppScreen::DailyView) => {
                self.focus_section(FocusedSection::Sokay);
                self.state.sokay_list_focused = true;
                self.sokay_list_state.select(Some(index));
            }
            (&ClickAction::OpenLog(index), AppScreen::Home) => {
                self.list_state.select(Some(index));
            }
            _ => return false,
        }
        self.toggle_mark_at_cursor();
        true
    }

    /// `x`: mark/unmark the highlighted row of the focused list
    pub(super) fn toggle_mark_at_cursor(&mut self) {
        match self.state.current_screen {
            AppScreen::Home => {
                if let Some(date) = self
                    .list_state
                    .selected()
                    .and_then(|i| self.state.daily_logs.get(i))
                    .map(|log| log.date)
                {
                    self.state.marks.toggle_log(date);
                }
            }
            AppScreen::DailyView => {
                if let Some((list, row)) = self.focused_row() {
                    let date = self.state.selected_date;
                    self.state.marks.toggle_row(list, date, row);
                }
            }
            _ => {}
        }
    }

    /// The focused DailyView list and its highlighted row, if it points at an entry
    fn focused_row(&self) -> Option<(RowList, usize)> {
        let log = self.state.get_daily_log(self.state.selected_date)?;
        match self.state.focused_section {
            FocusedSection::FoodItems if self.state.food_list_focused => self
                .food_list_state
                .selected()
                .filter(|&i| i < log.food_entries.len())
                .map(|i| (RowList::Food, i)),
            FocusedSection::Sokay if self.state.sokay_list_focused => self
                .sokay_list_state
                .selected()
                .filter(|&i| i < log.sokay_entries.len())
                .map(|i| (RowList::Sokay, i)),
            _ => None,
        }
    }

    /// `d` with marks: ask once for all of them. Returns false when nothing is marked.
    pub(super) fn request_delete_marked(&mut self) -> bool {
        if self.state.marks.is_empty() {
            return false;
        }
        self.state.current_screen = AppScreen::ConfirmDelete(DeleteTarget::Marked);
        true
    }

    /// Drops marks once their list is no longer the one in use: another
    /// screen, day, or section, or the list lost focus. Runs after every event
    /// so each way of leaving clears them without special-casing it.
    pub(super) fn prune_marks(&mut self) {
        let screen = &self.state.current_screen;
        let keep = match &self.state.marks {
            Marks::None => return,
            Marks::Rows { list, date, .. } => {
                *date == self.state.selected_date
                    && matches!(
                        screen,
                        AppScreen::DailyView
                            | AppScreen::ConfirmDelete(DeleteTarget::Marked)
                            | AppScreen::ShortcutsHelp
                    )
                    && match list {
                        RowList::Food => {
                            self.state.focused_section == FocusedSection::FoodItems
                                && self.state.food_list_focused
                        }
                        RowList::Sokay => {
                            self.state.focused_section == FocusedSection::Sokay
                                && self.state.sokay_list_focused
                        }
                    }
            }
            Marks::Logs(_) => matches!(
                screen,
                AppScreen::Home | AppScreen::ConfirmDelete(DeleteTarget::Marked)
            ),
        };
        if !keep {
            self.state.marks.clear();
        }
    }

    /// `y` on the batch confirmation: delete every marked row in one save/sync
    pub(super) async fn delete_marked(&mut self) -> Result<()> {
        match std::mem::take(&mut self.state.marks) {
            Marks::None => {}
            Marks::Rows { list, rows, .. } => {
                let log = match list {
                    RowList::Food => ActionHandler::delete_food_entries(&mut self.state, &rows),
                    RowList::Sokay => ActionHandler::delete_sokay_entries(&mut self.state, &rows),
                };
                if let Some(log) = log {
                    let (len, list_state) = match list {
                        RowList::Food => (log.food_entries.len(), &mut self.food_list_state),
                        RowList::Sokay => (log.sokay_entries.len(), &mut self.sokay_list_state),
                    };
                    let selected = list_state.selected().unwrap_or(0);
                    list_state.select(len.checked_sub(1).map(|last| selected.min(last)));
                    self.spawn_persist(log);
                }
                self.state.current_screen = AppScreen::DailyView;
            }
            Marks::Logs(dates) => {
                {
                    let mut db = self.db_manager.write().await;
                    for date in dates {
                        ActionHandler::delete_daily_log(
                            &mut self.state,
                            &mut db,
                            &self.file_manager,
                            date,
                        )
                        .await?;
                    }
                }
                self.spawn_sync();
                self.list_state.select(None);
                self.state.current_screen = AppScreen::Home;
            }
        }
        Ok(())
    }

    /// `n`/Esc on the batch confirmation: back to the list, marks kept
    pub(super) fn cancel_delete_marked(&mut self) {
        self.state.current_screen = match self.state.marks {
            Marks::Logs(_) => AppScreen::Home,
            _ => AppScreen::DailyView,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::app::test_support::test_app;
    use crate::models::{DeleteTarget, FoodEntry, Marks, RowList};
    use crossterm::event::{KeyModifiers, MouseButton, MouseEventKind};
    use ratatui::layout::Rect;
    use tempfile::TempDir;

    fn day(d: u32) -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(2026, 11, d).unwrap()
    }

    /// DailyView on day 1 with five food items and two sokay items, Food list focused.
    async fn food_app(dir: &TempDir) -> App {
        let mut app = test_app(dir).await;
        app.state.selected_date = day(1);
        let log = app.state.get_or_create_daily_log(day(1));
        for name in ["a", "b", "c", "d", "e"] {
            log.add_food_entry(FoodEntry::new(name.to_string()));
        }
        log.add_sokay_entry("donut".to_string());
        log.add_sokay_entry("chips".to_string());
        app.state.current_screen = AppScreen::DailyView;
        app.state.focused_section = FocusedSection::FoodItems;
        app.state.food_list_focused = true;
        app.food_list_state.select(Some(0));
        app
    }

    async fn key(app: &mut App, code: KeyCode) {
        app.handle_key_event_with_modifiers(code, KeyModifiers::NONE)
            .await
            .unwrap();
    }

    fn click(app: &mut App, action: ClickAction, modifiers: KeyModifiers) {
        app.click_targets = vec![ClickTarget::new(Rect::new(0, 0, 10, 1), action)];
        app.handle_mouse_event(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 1,
            row: 0,
            modifiers,
        });
    }

    fn food_names(app: &App) -> Vec<String> {
        app.state
            .get_daily_log(day(1))
            .unwrap()
            .food_entries
            .iter()
            .map(|e| e.name.clone())
            .collect()
    }

    #[tokio::test]
    async fn x_marks_rows_and_one_confirmation_deletes_them_all() {
        let dir = TempDir::new().unwrap();
        let mut app = food_app(&dir).await;

        key(&mut app, KeyCode::Char('x')).await; // a
        app.food_list_state.select(Some(2));
        key(&mut app, KeyCode::Char('x')).await; // c
        app.food_list_state.select(Some(4));
        key(&mut app, KeyCode::Char('x')).await; // e
        assert_eq!(app.state.marks.count(), 3);

        key(&mut app, KeyCode::Char('d')).await;
        assert!(matches!(
            app.state.current_screen,
            AppScreen::ConfirmDelete(DeleteTarget::Marked)
        ));
        key(&mut app, KeyCode::Char('y')).await;

        assert_eq!(food_names(&app), ["b", "d"]);
        assert!(app.state.marks.is_empty());
        assert!(matches!(app.state.current_screen, AppScreen::DailyView));
        let sel = app.food_list_state.selected().unwrap();
        assert!(sel < 2, "selection clamped, got {sel}");
    }

    #[tokio::test]
    async fn x_twice_unmarks() {
        let dir = TempDir::new().unwrap();
        let mut app = food_app(&dir).await;
        key(&mut app, KeyCode::Char('x')).await;
        key(&mut app, KeyCode::Char('x')).await;
        assert!(app.state.marks.is_empty());
    }

    #[tokio::test]
    async fn d_without_marks_still_deletes_highlighted_row() {
        let dir = TempDir::new().unwrap();
        let mut app = food_app(&dir).await;
        app.food_list_state.select(Some(1));
        key(&mut app, KeyCode::Char('d')).await;
        assert!(matches!(
            app.state.current_screen,
            AppScreen::ConfirmDelete(DeleteTarget::Food(1))
        ));
    }

    #[tokio::test]
    async fn cancel_keeps_marks() {
        let dir = TempDir::new().unwrap();
        let mut app = food_app(&dir).await;
        key(&mut app, KeyCode::Char('x')).await;
        key(&mut app, KeyCode::Char('d')).await;
        key(&mut app, KeyCode::Char('n')).await;
        assert!(matches!(app.state.current_screen, AppScreen::DailyView));
        assert_eq!(app.state.marks.count(), 1);
        assert_eq!(food_names(&app).len(), 5);
    }

    #[tokio::test]
    async fn esc_clears_marks_before_unfocusing() {
        let dir = TempDir::new().unwrap();
        let mut app = food_app(&dir).await;
        key(&mut app, KeyCode::Char('x')).await;
        key(&mut app, KeyCode::Esc).await;
        assert!(app.state.marks.is_empty());
        assert!(app.state.food_list_focused, "first Esc only clears marks");
        key(&mut app, KeyCode::Esc).await;
        assert!(!app.state.food_list_focused);
    }

    #[tokio::test]
    async fn ctrl_click_toggles_marks_and_plain_click_clears() {
        let dir = TempDir::new().unwrap();
        let mut app = food_app(&dir).await;
        click(&mut app, ClickAction::SelectFood(1), KeyModifiers::CONTROL);
        click(&mut app, ClickAction::SelectFood(3), KeyModifiers::CONTROL);
        assert!(app.state.marks.is_row_marked(RowList::Food, day(1), 1));
        assert!(app.state.marks.is_row_marked(RowList::Food, day(1), 3));
        assert!(matches!(app.state.current_screen, AppScreen::DailyView));

        click(&mut app, ClickAction::SelectFood(3), KeyModifiers::CONTROL);
        assert_eq!(app.state.marks.count(), 1);

        click(&mut app, ClickAction::SelectFood(2), KeyModifiers::NONE);
        assert!(app.state.marks.is_empty());
        assert_eq!(app.food_list_state.selected(), Some(2));
    }

    #[tokio::test]
    async fn ctrl_click_on_sokay_moves_marks_to_sokay_list() {
        let dir = TempDir::new().unwrap();
        let mut app = food_app(&dir).await;
        key(&mut app, KeyCode::Char('x')).await;
        click(&mut app, ClickAction::SelectSokay(1), KeyModifiers::CONTROL);
        assert_eq!(app.state.focused_section, FocusedSection::Sokay);
        assert!(app.state.marks.rows(RowList::Food, day(1)).is_none());
        assert!(app.state.marks.is_row_marked(RowList::Sokay, day(1), 1));

        key(&mut app, KeyCode::Char('d')).await;
        key(&mut app, KeyCode::Char('y')).await;
        let log = app.state.get_daily_log(day(1)).unwrap();
        assert_eq!(log.sokay_entries, ["donut"]);
        assert_eq!(log.food_entries.len(), 5);
    }

    #[tokio::test]
    async fn leaving_the_list_clears_marks() {
        let dir = TempDir::new().unwrap();
        let mut app = food_app(&dir).await;
        key(&mut app, KeyCode::Char('x')).await;
        app.handle_key_event_with_modifiers(KeyCode::Char('J'), KeyModifiers::SHIFT)
            .await
            .unwrap(); // next section
        assert_ne!(app.state.focused_section, FocusedSection::FoodItems);
        assert!(app.state.marks.is_empty());
    }

    #[tokio::test]
    async fn log_list_marks_delete_whole_days() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir).await;
        for d in 1..=4 {
            app.state.get_or_create_daily_log(day(d));
        }
        // daily_logs sorted newest first: 4, 3, 2, 1
        app.state.current_screen = AppScreen::Home;
        app.list_state.select(Some(0));
        key(&mut app, KeyCode::Char('x')).await; // day 4
        click(&mut app, ClickAction::OpenLog(2), KeyModifiers::CONTROL); // day 2
        assert!(matches!(app.state.current_screen, AppScreen::Home));
        assert_eq!(app.state.marks, Marks::Logs([day(4), day(2)].into()));

        key(&mut app, KeyCode::Char('d')).await;
        assert!(matches!(
            app.state.current_screen,
            AppScreen::ConfirmDelete(DeleteTarget::Marked)
        ));
        key(&mut app, KeyCode::Char('y')).await;

        let left: Vec<_> = app.state.daily_logs.iter().map(|l| l.date).collect();
        assert_eq!(left, [day(3), day(1)]);
        assert!(app.state.marks.is_empty());
        assert!(matches!(app.state.current_screen, AppScreen::Home));
    }

    #[tokio::test]
    async fn opening_a_log_clears_log_marks() {
        let dir = TempDir::new().unwrap();
        let mut app = test_app(&dir).await;
        app.state.get_or_create_daily_log(day(1));
        app.state.current_screen = AppScreen::Home;
        app.list_state.select(Some(0));
        key(&mut app, KeyCode::Char('x')).await;
        key(&mut app, KeyCode::Enter).await;
        assert!(matches!(app.state.current_screen, AppScreen::DailyView));
        assert!(app.state.marks.is_empty());
    }
}
