use chrono::{Datelike, NaiveDate};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::miles_stats::{calculate_monthly_miles, calculate_yearly_miles};
use crate::models::field_accessor::FieldType;
use crate::models::{AppState, DailyLog, FocusedSection, MeasurementField, RunningField};
use crate::ui::components::{create_highlight_style, render_help, render_title};
use crate::ui::{ClickAction, ClickTarget};

/// Active in-place edit of a numeric field, rendered directly inside its section
/// row (Measurements / Running) instead of in a popup modal.
pub struct InPlaceEdit<'a> {
    pub field: FieldType,
    pub buffer: &'a str,
    pub cursor: usize,
}

/// The read-only slice of app state every section renderer needs: which day is
/// shown, all logs (to locate that day), and which section holds focus. Bundling
/// these three keeps each `render_*_section` signature short. It's `Copy` — a
/// date plus two shared references — so it's passed by value freely.
#[derive(Clone, Copy)]
pub struct SectionCtx<'a> {
    pub selected_date: NaiveDate,
    pub daily_logs: &'a [DailyLog],
    pub focused_section: &'a FocusedSection,
}

/// Renders the daily view screen for a specific date
pub fn render_daily_view_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    edit: Option<InPlaceEdit>,
    click_targets: Option<&mut Vec<ClickTarget>>,
) {
    let mut click_targets = click_targets;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(5), // Title (increased for vertical padding)
            Constraint::Length(3), // Measurements (Weight, Waist)
            Constraint::Length(3), // Running (Miles, Elevation)
            Constraint::Min(4),    // Food list (scrollable)
            Constraint::Min(4),    // Sokay list (scrollable, same size as food)
            Constraint::Length(4), // Strength & Mobility section
            Constraint::Length(4), // Notes section
            Constraint::Length(3), // Help
        ])
        .split(f.area());

    let title = format!(
        "Mountains Training Log - {} {}",
        state.selected_date.format("%B %d, %Y"),
        sync_status
    );
    render_title(f, chunks[0], &title);

    // Shared read-model for every section renderer this frame.
    let ctx = SectionCtx {
        selected_date: state.selected_date,
        daily_logs: &state.daily_logs,
        focused_section: &state.focused_section,
    };

    render_measurements_section(
        f,
        chunks[1],
        ctx,
        edit.as_ref(),
        click_targets.as_deref_mut(),
    );

    let today = chrono::Local::now().date_naive();
    let yearly_miles = calculate_yearly_miles(&state.daily_logs, today);
    let monthly_miles = calculate_monthly_miles(&state.daily_logs, today);
    render_running_section(
        f,
        chunks[2],
        ctx,
        yearly_miles,
        monthly_miles,
        edit.as_ref(),
        click_targets.as_deref_mut(),
    );

    render_food_list_section(
        f,
        chunks[3],
        ctx,
        food_list_state,
        state.food_list_focused,
        click_targets.as_deref_mut(),
    );

    render_sokay_section(
        f,
        chunks[4],
        ctx,
        sokay_list_state,
        state.sokay_list_focused,
        click_targets.as_deref_mut(),
    );

    render_strength_mobility_section(f, chunks[5], ctx, click_targets.as_deref_mut());

    render_notes_section(f, chunks[6], ctx, click_targets.as_deref_mut());

    let help_tiers: &[&str] = if edit.is_some() {
        &[
            " Editing — type value | Enter: Save | Esc: Cancel",
            " Enter: Save | Esc: Cancel",
        ]
    } else {
        &[
            " Shift+J/K: Section | Tab: Toggle Num Fields | Enter: Add | j/k: List | e: Edit Item | d: Delete Item | Space: Shortcuts | S: Startup Screen | Esc: Back",
            " Shift+J/K: Section | Tab: Fields | Enter: Add | j/k: List | e: Edit | d: Delete | Space: Shortcuts | S: Startup | Esc: Back",
            " Shift+J/K: Section | Enter: Add | e: Edit | d: Delete | Space: More | Esc: Back",
            " Space: Shortcuts | Esc: Back",
        ]
    };
    render_help(f, chunks[7], help_tiers, true, false);

    // Render expanded overlay for multi-line sections when focused
    match &state.focused_section {
        FocusedSection::StrengthMobility => {
            render_strength_mobility_expanded(
                f,
                chunks[5],
                state.selected_date,
                &state.daily_logs,
                state.strength_mobility_scroll,
                click_targets,
            );
        }
        FocusedSection::Notes => {
            render_notes_expanded(
                f,
                chunks[6],
                state.selected_date,
                &state.daily_logs,
                state.notes_scroll,
                click_targets,
            );
        }
        _ => {}
    }
}

/// Renders the measurements display section
fn render_measurements_section(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    ctx: SectionCtx,
    edit: Option<&InPlaceEdit>,
    click_targets: Option<&mut Vec<ClickTarget>>,
) {
    let log = ctx
        .daily_logs
        .iter()
        .find(|log| log.date == ctx.selected_date);

    // A field in this section being actively edited in place (Weight or Waist).
    let editing_field = match edit.map(|e| e.field) {
        Some(FieldType::Weight) => Some(MeasurementField::Weight),
        Some(FieldType::Waist) => Some(MeasurementField::Waist),
        _ => None,
    };

    let section_focused = matches!(ctx.focused_section, FocusedSection::Measurements { .. });
    let has_focus = section_focused || editing_field.is_some();

    // The field showing the ► marker: the edited field while editing, else the
    // section's focused field.
    let marked_field: Option<MeasurementField> =
        editing_field.clone().or_else(|| match ctx.focused_section {
            FocusedSection::Measurements { focused_field } => Some(focused_field.clone()),
            _ => None,
        });

    let weight_value = log.and_then(|l| l.weight).map(|w| format!("{} lbs", w));
    let waist_value = log.and_then(|l| l.waist).map(|w| format!("{} in", w));

    let mut row = RowBuilder::new(Style::default().fg(Color::Yellow));
    let weight_region = row.push_field(
        marked_field.as_ref() == Some(&MeasurementField::Weight),
        "Weight: ",
        if editing_field == Some(MeasurementField::Weight) {
            edit
        } else {
            None
        },
        weight_value.as_deref(),
        " lbs",
        "Press 'w' to add",
    );
    row.push(" | ".to_string());
    let waist_region = row.push_field(
        marked_field.as_ref() == Some(&MeasurementField::Waist),
        "Waist Size: ",
        if editing_field == Some(MeasurementField::Waist) {
            edit
        } else {
            None
        },
        waist_value.as_deref(),
        " in",
        "Press 's' to add",
    );

    let border_style = if has_focus {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title("Measurements")
        .padding(ratatui::widgets::Padding::horizontal(1));
    let inner = block.inner(area);

    let caret_col = row.caret_col;
    let measurements_widget = Paragraph::new(Line::from(row.spans)).block(block);
    f.render_widget(measurements_widget, area);

    if let Some(click_targets) = click_targets {
        push_field_target(click_targets, inner, weight_region, FieldType::Weight);
        push_field_target(click_targets, inner, waist_region, FieldType::Waist);
    }

    if let Some(col) = caret_col {
        f.set_cursor_position((inner.x + col, inner.y));
    }
}

/// Dimmed style for inline "Press 'x' to add" placeholders shown when a numeric
/// field is unset, matching the dimmed placeholders used by the list sections.
fn placeholder_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

/// Accumulates the styled spans for one section row (Measurements / Running),
/// tracking the running display width (in cells) for caret math and the caret
/// column when a field is edited in place. Bundling these mutable accumulators
/// lets `push_field` take `&mut self` instead of three out-parameters.
struct RowBuilder {
    spans: Vec<Span<'static>>,
    width: u16,
    caret_col: Option<u16>,
    base: Style,
}

impl RowBuilder {
    fn new(base: Style) -> Self {
        Self {
            spans: Vec::new(),
            width: 0,
            caret_col: None,
            base,
        }
    }

    /// Pushes text in the row's base style, advancing the display width.
    fn push(&mut self, text: String) {
        let style = self.base;
        self.push_styled(text, style);
    }

    fn push_styled(&mut self, text: String, style: Style) {
        self.width += Span::raw(text.as_str()).width() as u16;
        self.spans.push(Span::styled(text, style));
    }

    /// Appends one labelled field to the row, recording the caret column when the
    /// field is being edited in place. `marked` adds the ► focus marker; `edit`
    /// (when `Some`) substitutes the input buffer for the value and sets the
    /// caret. When `value` is `None` and the field isn't being edited, the dimmed
    /// `help` placeholder is shown instead. Returns the field's `(start, width)`
    /// for click-target geometry.
    fn push_field(
        &mut self,
        marked: bool,
        label: &str,
        edit: Option<&InPlaceEdit>,
        value: Option<&str>,
        unit: &str,
        help: &str,
    ) -> (u16, u16) {
        let start = self.width;
        if marked {
            self.push("► ".to_string());
        }
        self.push(label.to_string());

        if let Some(edit) = edit {
            // Caret sits within the buffer; digits/dots are width-1 so char count == cells.
            self.caret_col = Some(self.width + edit.cursor as u16);
            self.push(edit.buffer.to_string());
            // Extra leading space so the block cursor doesn't sit flush against the unit.
            self.push(format!(" {}", unit));
        } else if let Some(value) = value {
            self.push(value.to_string());
        } else {
            self.push_styled(help.to_string(), placeholder_style());
        }
        (start, self.width.saturating_sub(start))
    }
}

fn push_field_target(
    click_targets: &mut Vec<ClickTarget>,
    inner: ratatui::layout::Rect,
    (start, width): (u16, u16),
    field: FieldType,
) {
    let area =
        ratatui::layout::Rect::new(inner.x.saturating_add(start), inner.y, width, inner.height)
            .intersection(inner);
    if area.width > 0 && area.height > 0 {
        click_targets.push(ClickTarget::new(area, ClickAction::EditField(field)));
    }
}

/// Renders the running activity display section
fn render_running_section(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    ctx: SectionCtx,
    yearly_miles: f32,
    monthly_miles: f32,
    edit: Option<&InPlaceEdit>,
    click_targets: Option<&mut Vec<ClickTarget>>,
) {
    let log = ctx
        .daily_logs
        .iter()
        .find(|log| log.date == ctx.selected_date);

    let editing_field = match edit.map(|e| e.field) {
        Some(FieldType::Miles) => Some(RunningField::Miles),
        Some(FieldType::Elevation) => Some(RunningField::Elevation),
        _ => None,
    };

    let section_focused = matches!(ctx.focused_section, FocusedSection::Running { .. });
    let has_focus = section_focused || editing_field.is_some();

    let marked_field: Option<RunningField> =
        editing_field.clone().or_else(|| match ctx.focused_section {
            FocusedSection::Running { focused_field } => Some(focused_field.clone()),
            _ => None,
        });

    let now = chrono::Local::now();
    let current_year = now.year();
    let current_month = now.month();

    let month_name = match current_month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    };

    let yearly_text = format!(
        "You have {:.1} miles covered for {}",
        yearly_miles, current_year
    );
    let monthly_text = if monthly_miles == 0.0 {
        format!("No miles covered yet for the month of {}", month_name)
    } else {
        format!(
            "{:.1} miles covered for the month of {}",
            monthly_miles, month_name
        )
    };

    let miles_value = log
        .and_then(|l| l.miles_covered)
        .map(|m| format!("{} mi", m));
    let elevation_value = log
        .and_then(|l| l.elevation_gain)
        .map(|e| format!("{} ft", e));

    let mut row = RowBuilder::new(Style::default().fg(Color::LightRed));
    let miles_region = row.push_field(
        marked_field.as_ref() == Some(&RunningField::Miles),
        "Miles: ",
        if editing_field == Some(RunningField::Miles) {
            edit
        } else {
            None
        },
        miles_value.as_deref(),
        " mi",
        "Press 'm' to add",
    );
    row.push(" | ".to_string());
    let elevation_region = row.push_field(
        marked_field.as_ref() == Some(&RunningField::Elevation),
        "Elevation: ",
        if editing_field == Some(RunningField::Elevation) {
            edit
        } else {
            None
        },
        elevation_value.as_deref(),
        " ft",
        "Press 'l' to add",
    );
    row.push(format!(" | {} | {}", yearly_text, monthly_text));

    let border_style = if has_focus {
        Style::default().fg(Color::LightRed)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title("Running")
        .padding(ratatui::widgets::Padding::horizontal(1));
    let inner = block.inner(area);

    let caret_col = row.caret_col;
    let running_widget = Paragraph::new(Line::from(row.spans)).block(block);
    f.render_widget(running_widget, area);

    if let Some(click_targets) = click_targets {
        push_field_target(click_targets, inner, miles_region, FieldType::Miles);
        push_field_target(click_targets, inner, elevation_region, FieldType::Elevation);
    }

    if let Some(col) = caret_col {
        f.set_cursor_position((inner.x + col, inner.y));
    }
}

/// Renders the food items list section
fn render_food_list_section(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    ctx: SectionCtx,
    food_list_state: &mut ListState,
    food_list_focused: bool,
    click_targets: Option<&mut Vec<ClickTarget>>,
) {
    let log = ctx
        .daily_logs
        .iter()
        .find(|log| log.date == ctx.selected_date);
    let entry_count = log.map_or(0, |log| log.food_entries.len());

    let items: Vec<ListItem> = if let Some(log) = log {
        if log.food_entries.is_empty() {
            vec![ListItem::new("No food entries yet. Press 'f' to add one.")]
        } else {
            log.food_entries
                .iter()
                .map(|entry| {
                    let display = format!("- {}", entry.name);
                    ListItem::new(display)
                })
                .collect()
        }
    } else {
        vec![ListItem::new("No food entries yet. Press 'f' to add one.")]
    };

    let border_style = if matches!(ctx.focused_section, FocusedSection::FoodItems) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let highlight_style =
        if matches!(ctx.focused_section, FocusedSection::FoodItems) && food_list_focused {
            create_highlight_style()
        } else {
            Style::default()
        };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title("Food Items")
        .padding(ratatui::widgets::Padding::uniform(1));
    let inner = block.inner(area);
    let list = List::new(items)
        .block(block)
        .highlight_style(highlight_style);
    f.render_stateful_widget(list, area, food_list_state);

    if let Some(click_targets) = click_targets {
        if entry_count == 0 {
            click_targets.push(ClickTarget::new(inner, ClickAction::AddFood));
        } else {
            push_visible_list_targets(
                click_targets,
                inner,
                food_list_state.offset(),
                entry_count,
                ClickAction::SelectFood,
            );
        }
    }
}

/// Renders the sokay display section
fn render_sokay_section(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    ctx: SectionCtx,
    sokay_list_state: &mut ListState,
    sokay_list_focused: bool,
    click_targets: Option<&mut Vec<ClickTarget>>,
) {
    let log = ctx
        .daily_logs
        .iter()
        .find(|log| log.date == ctx.selected_date);
    let entry_count = log.map_or(0, |log| log.sokay_entries.len());

    // Calculate cumulative sokay count up to selected date
    let cumulative_sokay = crate::events::handlers::ActionHandler::calculate_cumulative_sokay(
        &crate::models::AppState {
            current_screen: crate::models::AppScreen::DailyView,
            selected_date: ctx.selected_date,
            daily_logs: ctx.daily_logs.to_vec(),
            focused_section: FocusedSection::FoodItems,
            food_list_focused: false,
            sokay_list_focused: false,
            strength_mobility_scroll: 0,
            notes_scroll: 0,
            date_input_error: None,
            config_sync_focused_field: crate::models::ConfigSyncField::DbUrl,
            config_sync_status: None,
            frame_width: 0,
            frame_height: 0,
        },
        ctx.selected_date,
    );

    let title = format!("Sokay (Total: {})", cumulative_sokay);

    let items: Vec<ListItem> = if let Some(log) = log {
        if log.sokay_entries.is_empty() {
            vec![ListItem::new("No sokay entries yet. Press 'c' to add one.")]
        } else {
            log.sokay_entries
                .iter()
                .map(|entry| {
                    let display = format!("- {}", entry);
                    ListItem::new(display)
                })
                .collect()
        }
    } else {
        vec![ListItem::new("No sokay entries yet. Press 'c' to add one.")]
    };

    let border_style = if matches!(ctx.focused_section, FocusedSection::Sokay) {
        Style::default().fg(Color::Magenta)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let highlight_style =
        if matches!(ctx.focused_section, FocusedSection::Sokay) && sokay_list_focused {
            create_highlight_style()
        } else {
            Style::default()
        };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title)
        .padding(ratatui::widgets::Padding::uniform(1));
    let inner = block.inner(area);
    let list = List::new(items)
        .block(block)
        .highlight_style(highlight_style);
    f.render_stateful_widget(list, area, sokay_list_state);

    if let Some(click_targets) = click_targets {
        if entry_count == 0 {
            click_targets.push(ClickTarget::new(inner, ClickAction::AddSokay));
        } else {
            push_visible_list_targets(
                click_targets,
                inner,
                sokay_list_state.offset(),
                entry_count,
                ClickAction::SelectSokay,
            );
        }
    }
}

fn push_visible_list_targets(
    click_targets: &mut Vec<ClickTarget>,
    inner: ratatui::layout::Rect,
    offset: usize,
    item_count: usize,
    action: fn(usize) -> ClickAction,
) {
    for row in 0..inner.height as usize {
        let index = offset + row;
        if index >= item_count {
            break;
        }
        click_targets.push(ClickTarget::new(
            ratatui::layout::Rect::new(inner.x, inner.y + row as u16, inner.width, 1),
            action(index),
        ));
    }
}

/// Renders the strength & mobility display section
fn render_strength_mobility_section(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    ctx: SectionCtx,
    click_targets: Option<&mut Vec<ClickTarget>>,
) {
    let log = ctx
        .daily_logs
        .iter()
        .find(|log| log.date == ctx.selected_date);

    let has_focus = matches!(ctx.focused_section, FocusedSection::StrengthMobility);

    let sm_text = if let Some(log) = log {
        if let Some(sm) = &log.strength_mobility {
            if sm.trim().is_empty() {
                "No exercises recorded yet. Press 't' to add training info.".to_string()
            } else {
                sm.clone()
            }
        } else {
            "No exercises recorded yet. Press 't' to add training info.".to_string()
        }
    } else {
        "No exercises recorded yet. Press 't' to add training info.".to_string()
    };

    let border_style = if has_focus {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title("Strength & Mobility")
        .padding(ratatui::widgets::Padding::horizontal(1));
    let inner = block.inner(area);
    let sm_widget = Paragraph::new(sm_text)
        .style(Style::default().fg(Color::Cyan))
        .block(block)
        .wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(sm_widget, area);
    if let Some(click_targets) = click_targets {
        click_targets.push(ClickTarget::new(inner, ClickAction::StrengthMobility));
    }
}

/// Renders the notes display section
fn render_notes_section(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    ctx: SectionCtx,
    click_targets: Option<&mut Vec<ClickTarget>>,
) {
    let log = ctx
        .daily_logs
        .iter()
        .find(|log| log.date == ctx.selected_date);

    let has_focus = matches!(ctx.focused_section, FocusedSection::Notes);

    let notes_text = if let Some(log) = log {
        if let Some(notes) = &log.notes {
            if notes.trim().is_empty() {
                "No notes for this day yet. Press 'n' to add notes.".to_string()
            } else {
                notes.clone()
            }
        } else {
            "No notes for this day yet. Press 'n' to add notes.".to_string()
        }
    } else {
        "No notes for this day yet. Press 'n' to add notes.".to_string()
    };

    let border_style = if has_focus {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title("Notes")
        .padding(ratatui::widgets::Padding::horizontal(1));
    let inner = block.inner(area);
    let notes_widget = Paragraph::new(notes_text)
        .style(Style::default().fg(Color::Green))
        .block(block)
        .wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(notes_widget, area);
    if let Some(click_targets) = click_targets {
        click_targets.push(ClickTarget::new(inner, ClickAction::Notes));
    }
}

/// Calculates the number of display lines needed for text at given width
fn calculate_text_height(text: &str, width: usize) -> usize {
    if text.is_empty() || width == 0 {
        return 1;
    }

    let mut total_lines = 0;
    for line in text.lines() {
        if line.is_empty() {
            total_lines += 1;
        } else {
            let line_len = line.chars().count();
            let wrapped_lines = line_len.div_ceil(width);
            total_lines += wrapped_lines.max(1);
        }
    }

    total_lines.max(1)
}

/// Maximum vertical scroll offset (in lines) for an expandable multi-line
/// section, given the full frame size. Mirrors the geometry used by the
/// expanded renderers so scrolling can be clamped to the content; returns 0
/// when the content fits without scrolling.
pub fn max_scroll_offset(text: &str, frame_width: u16, frame_height: u16) -> u16 {
    let default_height = 4;
    // The daily view layout insets the frame by a 1-cell margin on each side,
    // and each section block has a border + 1 cell of horizontal padding.
    let section_width = frame_width.saturating_sub(2);
    let width = section_width.saturating_sub(4) as usize;
    let content_height = calculate_text_height(text, width) as u16;
    let needed_height = content_height + 2;

    if needed_height <= default_height {
        return 0;
    }

    let max_height = (frame_height * 60 / 100).max(default_height);
    let expanded_height = needed_height.min(max_height);
    let visible_text_lines = expanded_height.saturating_sub(2); // top + bottom border
    // `content_height` is a character-based estimate; word wrapping can render a
    // line or two more than estimated, so add a 1-line buffer to guarantee the
    // last line is always reachable.
    content_height.saturating_sub(visible_text_lines) + 1
}

/// Renders expanded Strength & Mobility section when focused
fn render_strength_mobility_expanded(
    f: &mut Frame,
    original_area: ratatui::layout::Rect,
    selected_date: NaiveDate,
    daily_logs: &[DailyLog],
    scroll_offset: u16,
    click_targets: Option<&mut Vec<ClickTarget>>,
) {
    let log = daily_logs.iter().find(|log| log.date == selected_date);

    let text = if let Some(log) = log {
        if let Some(sm) = &log.strength_mobility {
            if sm.trim().is_empty() {
                "No exercises recorded yet. Press 't' to add training info.".to_string()
            } else {
                sm.clone()
            }
        } else {
            "No exercises recorded yet. Press 't' to add training info.".to_string()
        }
    } else {
        "No exercises recorded yet. Press 't' to add training info.".to_string()
    };

    let default_height = 4;
    let width = original_area.width.saturating_sub(4) as usize;
    let content_height = calculate_text_height(&text, width);
    let needed_height = (content_height as u16) + 2;

    if needed_height <= default_height {
        return;
    }

    let max_height = (f.area().height * 60 / 100).max(default_height);
    let expanded_height = needed_height.min(max_height);

    let expanded_area = ratatui::layout::Rect {
        x: original_area.x,
        y: original_area.y + original_area.height - expanded_height,
        width: original_area.width,
        height: expanded_height,
    };

    f.render_widget(Clear, expanded_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title("Strength & Mobility")
        .padding(ratatui::widgets::Padding::horizontal(1));

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Cyan))
        .block(block)
        .wrap(ratatui::widgets::Wrap { trim: false })
        .scroll((scroll_offset, 0));

    f.render_widget(paragraph, expanded_area);
    if let Some(click_targets) = click_targets {
        click_targets.push(ClickTarget::new(
            expanded_area,
            ClickAction::StrengthMobility,
        ));
    }
}

/// Renders expanded Notes section when focused
fn render_notes_expanded(
    f: &mut Frame,
    original_area: ratatui::layout::Rect,
    selected_date: NaiveDate,
    daily_logs: &[DailyLog],
    scroll_offset: u16,
    click_targets: Option<&mut Vec<ClickTarget>>,
) {
    let log = daily_logs.iter().find(|log| log.date == selected_date);

    let text = if let Some(log) = log {
        if let Some(notes) = &log.notes {
            if notes.trim().is_empty() {
                "No notes for this day yet. Press 'n' to add notes.".to_string()
            } else {
                notes.clone()
            }
        } else {
            "No notes for this day yet. Press 'n' to add notes.".to_string()
        }
    } else {
        "No notes for this day yet. Press 'n' to add notes.".to_string()
    };

    let default_height = 4;
    let width = original_area.width.saturating_sub(4) as usize;
    let content_height = calculate_text_height(&text, width);
    let needed_height = (content_height as u16) + 2;

    if needed_height <= default_height {
        return;
    }

    let max_height = (f.area().height * 60 / 100).max(default_height);
    let expanded_height = needed_height.min(max_height);

    let expanded_area = ratatui::layout::Rect {
        x: original_area.x,
        y: original_area.y + original_area.height - expanded_height,
        width: original_area.width,
        height: expanded_height,
    };

    f.render_widget(Clear, expanded_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .title("Notes")
        .padding(ratatui::widgets::Padding::horizontal(1));

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Green))
        .block(block)
        .wrap(ratatui::widgets::Wrap { trim: false })
        .scroll((scroll_offset, 0));

    f.render_widget(paragraph, expanded_area);
    if let Some(click_targets) = click_targets {
        click_targets.push(ClickTarget::new(expanded_area, ClickAction::Notes));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn daily_view_registers_each_numeric_field() {
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        let state = AppState::new();
        let mut food_state = ListState::default();
        let mut sokay_state = ListState::default();
        let mut targets = Vec::new();

        terminal
            .draw(|frame| {
                render_daily_view_screen(
                    frame,
                    &state,
                    &mut food_state,
                    &mut sokay_state,
                    "",
                    None,
                    Some(&mut targets),
                );
            })
            .unwrap();

        for field in [
            FieldType::Weight,
            FieldType::Waist,
            FieldType::Miles,
            FieldType::Elevation,
        ] {
            assert!(
                targets
                    .iter()
                    .any(|target| target.action == ClickAction::EditField(field))
            );
        }
    }

    #[test]
    fn expanded_notes_target_is_registered_after_underlying_sections() {
        let backend = TestBackend::new(100, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = AppState::new();
        state.focused_section = FocusedSection::Notes;
        let date = state.selected_date;
        state.daily_logs.push(DailyLog {
            date,
            notes: Some("long notes ".repeat(200)),
            ..DailyLog::new(date)
        });
        let mut food_state = ListState::default();
        let mut sokay_state = ListState::default();
        let mut targets = Vec::new();

        terminal
            .draw(|frame| {
                render_daily_view_screen(
                    frame,
                    &state,
                    &mut food_state,
                    &mut sokay_state,
                    "",
                    None,
                    Some(&mut targets),
                );
            })
            .unwrap();

        let notes_targets: Vec<&ClickTarget> = targets
            .iter()
            .filter(|target| target.action == ClickAction::Notes)
            .collect();
        assert_eq!(notes_targets.len(), 2);
        assert!(notes_targets[1].area.height > notes_targets[0].area.height);
        assert_eq!(
            targets.last().map(|target| &target.action),
            Some(&ClickAction::Notes)
        );
    }

    #[test]
    fn visible_list_targets_include_scroll_offset() {
        let mut targets = Vec::new();
        push_visible_list_targets(
            &mut targets,
            ratatui::layout::Rect::new(5, 10, 20, 3),
            4,
            10,
            ClickAction::SelectFood,
        );

        assert_eq!(
            targets
                .iter()
                .map(|target| target.action.clone())
                .collect::<Vec<_>>(),
            vec![
                ClickAction::SelectFood(4),
                ClickAction::SelectFood(5),
                ClickAction::SelectFood(6),
            ]
        );
        assert_eq!(targets[0].area.y, 10);
        assert_eq!(targets[2].area.y, 12);
    }
}
