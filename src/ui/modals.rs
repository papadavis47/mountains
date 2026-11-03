use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::ui::components::{centered_rect, create_input_style, format_input_with_cursor};
use crate::ui::screens::{calculate_cursor_in_wrapped_text, vertical_scroll_offset, wrap_at_width};

/// Types of input modals
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputModalType {
    /// Single-line text input (50% x 13%)
    Text,
    /// Multi-line text input with wrapping (60% x 40%)
    Multiline,
}

impl InputModalType {
    fn dimensions(&self) -> (u16, u16) {
        match self {
            InputModalType::Text => (50, 13),
            InputModalType::Multiline => (60, 40),
        }
    }

    fn padding(&self) -> ratatui::widgets::Padding {
        match self {
            InputModalType::Text => ratatui::widgets::Padding {
                left: 1,
                right: 1,
                top: 1,
                bottom: 0,
            },
            InputModalType::Multiline => ratatui::widgets::Padding::uniform(1),
        }
    }
}

/// Configuration for rendering an input modal
pub struct InputModalConfig {
    pub title: String,
    pub border_color: Color,
    pub modal_type: InputModalType,
    /// Overrides the modal type's default width percentage when set.
    pub width_percent: Option<u16>,
}

impl InputModalConfig {
    pub fn new(title: String, border_color: Color, modal_type: InputModalType) -> Self {
        Self {
            title,
            border_color,
            modal_type,
            width_percent: None,
        }
    }

    /// Helper for text input modals
    pub fn text(title: String, border_color: Color) -> Self {
        Self::new(title, border_color, InputModalType::Text)
    }

    /// Helper for multiline input modals
    pub fn multiline(title: String, border_color: Color) -> Self {
        Self::new(title, border_color, InputModalType::Multiline)
    }

    /// Overrides the default width with a percentage of the screen width.
    pub fn with_width_percent(mut self, width_percent: u16) -> Self {
        self.width_percent = Some(width_percent);
        self
    }
}

/// Renders a generic input modal over the current screen
pub fn render_input_modal(
    f: &mut Frame,
    config: InputModalConfig,
    input_buffer: &str,
    cursor_position: usize,
) {
    let (default_width, height_percent) = config.modal_type.dimensions();
    let width_percent = config.width_percent.unwrap_or(default_width);
    let popup_area = centered_rect(f.area(), width_percent, height_percent);

    // Clear the popup area to prevent visual artifacts
    f.render_widget(Clear, popup_area);

    // Create the dialog block with title and padding
    let block = Block::default()
        .borders(Borders::ALL)
        .title(config.title)
        .style(Style::default().fg(config.border_color))
        .padding(config.modal_type.padding());

    // Get the inner area for the input text (after borders and padding)
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Render based on modal type
    match config.modal_type {
        InputModalType::Text => {
            // Single-line input rendering
            let input_text = format_input_with_cursor(input_buffer);
            let input = Paragraph::new(input_text).style(create_input_style());
            f.render_widget(input, inner_area);

            // Set cursor position (inner area already accounts for borders and padding)
            f.set_cursor_position((inner_area.x + cursor_position as u16, inner_area.y));
        }
        InputModalType::Multiline => {
            // Multi-line input rendering with word wrapping
            let width = inner_area.width as usize;
            let wrapped_text = if input_buffer.is_empty() {
                " ".to_string()
            } else {
                wrap_at_width(input_buffer, width)
            };

            // Calculate cursor position on the wrapped text
            let (cursor_x, cursor_y) =
                calculate_cursor_in_wrapped_text(inner_area, input_buffer, cursor_position, width);

            // Scroll so the cursor line stays visible
            let height = inner_area.height;
            let cursor_line = cursor_y.saturating_sub(inner_area.y);
            let scroll = vertical_scroll_offset(cursor_line, height);
            let total_lines = wrapped_text.split('\n').count() as u16;

            let input = Paragraph::new(wrapped_text)
                .style(create_input_style())
                .scroll((scroll, 0));
            f.render_widget(input, inner_area);

            // Scroll indicators on the top/bottom borders
            let indicator_style = Style::default().fg(config.border_color);
            if scroll > 0 {
                render_scroll_indicator(f, popup_area, popup_area.y, " ↑ more ", indicator_style);
            }
            if total_lines > scroll + height {
                let bottom = popup_area.y + popup_area.height.saturating_sub(1);
                render_scroll_indicator(f, popup_area, bottom, " ↓ more ", indicator_style);
            }

            f.set_cursor_position((cursor_x, cursor_y - scroll));
        }
    }
}

/// Draws a scroll indicator right-aligned on a border row of the popup
fn render_scroll_indicator(f: &mut Frame, popup: Rect, y: u16, label: &str, style: Style) {
    let label_width = label.chars().count() as u16;
    if popup.width < label_width + 4 {
        return;
    }
    let area = Rect::new(popup.x + popup.width - label_width - 2, y, label_width, 1);
    f.render_widget(Paragraph::new(label).style(style), area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn render(buffer: &str, cursor: usize) -> (String, (u16, u16)) {
        let mut term = Terminal::new(TestBackend::new(80, 50)).unwrap();
        term.draw(|f| {
            let config = InputModalConfig::multiline("Notes".into(), Color::White);
            render_input_modal(f, config, buffer, cursor);
        })
        .unwrap();
        let pos = term.get_cursor_position().unwrap();
        let screen = term
            .backend()
            .buffer()
            .content()
            .chunks(80)
            .map(|row| row.iter().map(|c| c.symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");
        (screen, (pos.x, pos.y))
    }

    fn long_note() -> String {
        (1..=30)
            .map(|i| format!("line{i}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn multiline_scrolls_to_show_cursor_line_past_box_height() {
        let note = long_note();
        let (screen, (_, y)) = render(&note, note.len());
        assert!(screen.contains("line30"), "last typed line must be visible");
        assert!(!screen.contains("line1\u{20}"), "top lines scrolled away");
        assert!(screen.contains("↑ more"));
        assert!(!screen.contains("↓ more"));
        // 50 rows * 40% = 20-row popup at y=15; inner bottom row = 15+20-3
        assert!(y < 15 + 20 - 2, "cursor inside box, got y={y}");
    }

    #[test]
    fn multiline_at_top_shows_down_indicator_only() {
        let (screen, _) = render(&long_note(), 0);
        assert!(screen.contains("line1 "));
        assert!(screen.contains("↓ more"));
        assert!(!screen.contains("↑ more"));
    }

    #[test]
    fn multiline_short_text_has_no_indicators() {
        let (screen, _) = render("hello", 5);
        assert!(!screen.contains("more"));
    }
}
