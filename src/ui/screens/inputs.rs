use ratatui::{Frame, style::Color, widgets::ListState};

use super::daily_view::render_daily_view_screen;
use super::home::render_home_screen;
use crate::models::AppState;
use crate::ui::modals::{InputModalConfig, render_input_modal};

/// Renders the add food entry screen as a centered modal dialog
pub fn render_add_food_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
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

    let title = format!("Add Food - {}", state.selected_date.format("%B %d, %Y"));
    let config = InputModalConfig::text(title, Color::Yellow);
    render_input_modal(f, config, input_buffer, cursor_position);
}

/// Renders the edit food entry screen as a centered modal dialog
pub fn render_edit_food_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
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

    let title = format!("Edit Food - {}", state.selected_date.format("%B %d, %Y"));
    let config = InputModalConfig::text(title, Color::Yellow);
    render_input_modal(f, config, input_buffer, cursor_position);
}

/// Renders the edit strength & mobility screen as a centered modal dialog
pub fn render_edit_strength_mobility_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
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

    let title = format!(
        "Edit Strength & Mobility - {}",
        state.selected_date.format("%B %d, %Y")
    );
    let config = InputModalConfig::multiline(title, Color::Cyan);
    render_input_modal(f, config, input_buffer, cursor_position);
}

/// Renders the edit notes screen as a centered modal dialog
pub fn render_edit_notes_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
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

    let title = format!("Edit Notes - {}", state.selected_date.format("%B %d, %Y"));
    let config = InputModalConfig::multiline(title, Color::Green);
    render_input_modal(f, config, input_buffer, cursor_position);
}

/// Renders the add sokay screen as a centered modal dialog
pub fn render_add_sokay_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
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

    let title = format!(
        "Add Sokay Entry - {}",
        state.selected_date.format("%B %d, %Y")
    );
    let config = InputModalConfig::text(title, Color::Magenta);
    render_input_modal(f, config, input_buffer, cursor_position);
}

/// Renders the date input screen as a modal over the home screen
pub fn render_date_input_screen(
    f: &mut Frame,
    state: &AppState,
    list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
) {
    render_home_screen(f, state, list_state, sync_status, None);

    let (title, color) = match &state.date_input_error {
        Some(err) => (format!("Add Entry (MM.DD.YYYY) - {}", err), Color::Red),
        None => ("Add Entry (MM.DD.YYYY)".to_string(), Color::Cyan),
    };
    let config = InputModalConfig::text(title, color).with_width_percent(25);
    render_input_modal(f, config, input_buffer, cursor_position);
}

/// Renders the edit sokay screen as a centered modal dialog
pub fn render_edit_sokay_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
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

    let title = format!(
        "Edit Sokay Entry - {}",
        state.selected_date.format("%B %d, %Y")
    );
    let config = InputModalConfig::text(title, Color::Magenta);
    render_input_modal(f, config, input_buffer, cursor_position);
}

/// Wraps text at word boundaries to fit within a given width
pub fn wrap_at_width(text: &str, width: usize) -> String {
    if width == 0 {
        return text.to_string();
    }

    let mut result = String::new();
    let mut current_line = String::new();
    let mut current_line_width = 0;

    for word in text.split_inclusive(|c: char| c.is_whitespace() || c == '\n') {
        if word.contains('\n') {
            let parts: Vec<&str> = word.split('\n').collect();

            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    result.push_str(&current_line);
                    result.push('\n');
                    current_line.clear();
                    current_line_width = 0;
                }

                if !part.is_empty() {
                    let part_width = part.chars().count();

                    if current_line_width + part_width > width && current_line_width > 0 {
                        result.push_str(&current_line);
                        result.push('\n');
                        current_line.clear();
                        current_line_width = 0;
                    }

                    if part_width > width {
                        for ch in part.chars() {
                            if current_line_width >= width {
                                result.push_str(&current_line);
                                result.push('\n');
                                current_line.clear();
                                current_line_width = 0;
                            }
                            current_line.push(ch);
                            current_line_width += 1;
                        }
                    } else {
                        current_line.push_str(part);
                        current_line_width += part_width;
                    }
                }
            }
        } else {
            let word_width = word.chars().count();

            if current_line_width + word_width > width && current_line_width > 0 {
                result.push_str(&current_line);
                result.push('\n');
                current_line.clear();
                current_line_width = 0;
            }

            if word_width > width {
                for ch in word.chars() {
                    if current_line_width >= width {
                        result.push_str(&current_line);
                        result.push('\n');
                        current_line.clear();
                        current_line_width = 0;
                    }
                    current_line.push(ch);
                    current_line_width += 1;
                }
            } else {
                current_line.push_str(word);
                current_line_width += word_width;
            }
        }
    }

    if !current_line.is_empty() {
        result.push_str(&current_line);
    }

    result
}

/// Calculates cursor position in manually-wrapped text with word wrapping
pub fn calculate_cursor_in_wrapped_text(
    area: ratatui::layout::Rect,
    original_text: &str,
    cursor_pos_bytes: usize,
    width: usize,
) -> (u16, u16) {
    if width == 0 {
        return (area.x, area.y);
    }

    let cursor_pos_chars = if cursor_pos_bytes <= original_text.len() {
        original_text[..cursor_pos_bytes].chars().count()
    } else {
        original_text.chars().count()
    };

    let mut line = 0;
    let mut col = 0;
    let mut char_count = 0;
    let mut current_line_width = 0;

    for word in original_text.split_inclusive(|c: char| c.is_whitespace() || c == '\n') {
        let word_char_count = word.chars().count();

        if char_count + word_char_count > cursor_pos_chars {
            let chars_into_word = cursor_pos_chars - char_count;

            if word.contains('\n') {
                let parts: Vec<&str> = word.split('\n').collect();
                let mut chars_processed = 0;

                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        line += 1;
                        col = 0;
                        current_line_width = 0;
                        chars_processed += 1;

                        if chars_processed >= chars_into_word {
                            break;
                        }
                    }

                    if !part.is_empty() {
                        let part_width = part.chars().count();
                        let chars_to_take = (chars_into_word - chars_processed).min(part_width);

                        if current_line_width + part_width > width && current_line_width > 0 {
                            line += 1;
                            current_line_width = 0;
                        }

                        if part_width > width {
                            for (idx, _ch) in part.chars().enumerate() {
                                if idx >= chars_to_take {
                                    break;
                                }
                                if current_line_width >= width {
                                    line += 1;
                                    current_line_width = 0;
                                }
                                current_line_width += 1;
                                chars_processed += 1;
                            }
                        } else {
                            current_line_width += chars_to_take;
                            chars_processed += chars_to_take;
                        }

                        col = current_line_width;

                        if chars_processed >= chars_into_word {
                            break;
                        }
                    }
                }
            } else {
                let word_width = word.chars().count();

                if current_line_width + word_width > width && current_line_width > 0 {
                    line += 1;
                    current_line_width = 0;
                }

                if word_width > width {
                    for (idx, _ch) in word.chars().enumerate() {
                        if idx >= chars_into_word {
                            break;
                        }
                        if current_line_width >= width {
                            line += 1;
                            current_line_width = 0;
                        }
                        current_line_width += 1;
                    }
                } else {
                    current_line_width += chars_into_word;
                }

                col = current_line_width;
            }

            break;
        }

        char_count += word_char_count;

        if word.contains('\n') {
            let parts: Vec<&str> = word.split('\n').collect();

            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    line += 1;
                    current_line_width = 0;
                }

                if !part.is_empty() {
                    let part_width = part.chars().count();

                    if current_line_width + part_width > width && current_line_width > 0 {
                        line += 1;
                        current_line_width = 0;
                    }

                    if part_width > width {
                        for _ch in part.chars() {
                            if current_line_width >= width {
                                line += 1;
                                current_line_width = 0;
                            }
                            current_line_width += 1;
                        }
                    } else {
                        current_line_width += part_width;
                    }
                }
            }

            col = current_line_width;
        } else {
            let word_width = word.chars().count();

            if current_line_width + word_width > width && current_line_width > 0 {
                line += 1;
                current_line_width = 0;
            }

            if word_width > width {
                for _ch in word.chars() {
                    if current_line_width >= width {
                        line += 1;
                        current_line_width = 0;
                    }
                    current_line_width += 1;
                }
            } else {
                current_line_width += word_width;
            }

            col = current_line_width;
        }
    }

    let cursor_x = area.x + col as u16;
    let cursor_y = area.y + line as u16;

    (cursor_x, cursor_y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    // -- wrap_at_width -------------------------------------------------------

    #[test]
    fn wrap_width_zero_returns_input_unchanged() {
        assert_eq!(wrap_at_width("abc def", 0), "abc def");
    }

    #[test]
    fn wrap_empty_string_stays_empty() {
        assert_eq!(wrap_at_width("", 10), "");
    }

    #[test]
    fn wrap_text_that_fits_is_unchanged_and_single_line() {
        let wrapped = wrap_at_width("hello", 10);
        assert_eq!(wrapped, "hello");
        assert!(!wrapped.contains('\n'));
    }

    #[test]
    fn wrap_preserves_explicit_newline() {
        assert_eq!(wrap_at_width("a\nb", 10), "a\nb");
    }

    #[test]
    fn wrap_preserves_blank_line_between_newlines() {
        assert_eq!(wrap_at_width("a\n\nb", 10), "a\n\nb");
    }

    #[test]
    fn wrap_hard_breaks_word_longer_than_width() {
        // No spaces to break on, so the word is split at exactly `width` chars.
        assert_eq!(wrap_at_width("abcdefgh", 3), "abc\ndef\ngh");
    }

    #[test]
    fn wrap_no_rendered_line_exceeds_width() {
        // Property: for varied inputs, every produced line fits within `width`.
        let width = 7;
        for input in ["aaa bbb ccc", "the quick brown fox jumped", "abcdefghij k"] {
            for line in wrap_at_width(input, width).split('\n') {
                assert!(
                    line.chars().count() <= width,
                    "line {line:?} exceeds width {width} for input {input:?}"
                );
            }
        }
    }

    // -- calculate_cursor_in_wrapped_text ------------------------------------

    fn origin(w: u16, h: u16) -> Rect {
        Rect {
            x: 0,
            y: 0,
            width: w,
            height: h,
        }
    }

    #[test]
    fn cursor_width_zero_returns_area_origin() {
        let area = Rect {
            x: 5,
            y: 7,
            width: 20,
            height: 4,
        };
        assert_eq!(
            calculate_cursor_in_wrapped_text(area, "hello", 3, 0),
            (5, 7)
        );
    }

    #[test]
    fn cursor_at_start_is_top_left() {
        assert_eq!(
            calculate_cursor_in_wrapped_text(origin(20, 4), "hello world", 0, 10),
            (0, 0)
        );
    }

    #[test]
    fn cursor_mid_line_tracks_column() {
        // Byte 6 == just after "hello " on a line wide enough not to wrap.
        assert_eq!(
            calculate_cursor_in_wrapped_text(origin(20, 4), "hello world", 6, 20),
            (6, 0)
        );
    }

    #[test]
    fn cursor_at_end_of_single_line() {
        assert_eq!(
            calculate_cursor_in_wrapped_text(origin(20, 4), "hello world", 11, 20),
            (11, 0)
        );
    }

    #[test]
    fn cursor_after_newline_moves_to_next_line() {
        // Cursor on 'b' (byte 2) sits at the start of line 1.
        assert_eq!(
            calculate_cursor_in_wrapped_text(origin(20, 4), "a\nb", 2, 10),
            (0, 1)
        );
    }

    #[test]
    fn cursor_on_wrapped_line() {
        // "aaa bbb" at width 4 wraps to "aaa \nbbb"; cursor at end is line 1 col 3.
        assert_eq!(
            calculate_cursor_in_wrapped_text(origin(20, 4), "aaa bbb", 7, 4),
            (3, 1)
        );
    }

    #[test]
    fn cursor_offset_applies_area_origin() {
        let area = Rect {
            x: 4,
            y: 2,
            width: 20,
            height: 4,
        };
        assert_eq!(
            calculate_cursor_in_wrapped_text(area, "hello", 5, 20),
            (9, 2)
        );
    }

    #[test]
    fn cursor_handles_multibyte_chars_without_panicking() {
        // "café": é is 2 bytes, so byte 5 == 4 chars. Column is char-based, not byte.
        assert_eq!(
            calculate_cursor_in_wrapped_text(origin(20, 4), "café", 5, 10),
            (4, 0)
        );
        // Cursor between "ca" and "fé" (byte 2) lands at column 2.
        assert_eq!(
            calculate_cursor_in_wrapped_text(origin(20, 4), "café", 2, 10),
            (2, 0)
        );
    }

    #[test]
    fn cursor_byte_past_end_clamps_to_text_end() {
        assert_eq!(
            calculate_cursor_in_wrapped_text(origin(20, 4), "ab", 999, 10),
            (2, 0)
        );
    }
}
