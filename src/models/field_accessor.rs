use crate::models::{AppState, DailyLog};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldType {
    Weight,
    Waist,
    Miles,
    Elevation,
    StrengthMobility,
    Notes,
}

impl FieldType {
    /// Gets the current value of this field as a String
    pub fn get_value(&self, state: &AppState) -> String {
        if let Some(log) = state.get_daily_log(state.selected_date) {
            match self {
                FieldType::Weight => log.weight.map(|w| w.to_string()).unwrap_or_default(),
                FieldType::Waist => log.waist.map(|w| w.to_string()).unwrap_or_default(),
                FieldType::Miles => log.miles_covered.map(|m| m.to_string()).unwrap_or_default(),
                FieldType::Elevation => log
                    .elevation_gain
                    .map(|e| e.to_string())
                    .unwrap_or_default(),
                FieldType::StrengthMobility => log.strength_mobility.clone().unwrap_or_default(),
                FieldType::Notes => log.notes.clone().unwrap_or_default(),
            }
        } else {
            String::new()
        }
    }

    /// Updates this field with the provided input and returns the modified log
    pub fn update_value(&self, state: &mut AppState, input: String) -> DailyLog {
        let log = state.get_or_create_daily_log(state.selected_date);

        match self {
            FieldType::Weight => {
                log.weight = if input.is_empty() {
                    None
                } else {
                    input.parse().ok()
                };
            }
            FieldType::Waist => {
                log.waist = if input.is_empty() {
                    None
                } else {
                    input.parse().ok()
                };
            }
            FieldType::Miles => {
                log.miles_covered = if input.is_empty() {
                    None
                } else {
                    input.parse().ok()
                };
            }
            FieldType::Elevation => {
                log.elevation_gain = if input.is_empty() {
                    None
                } else {
                    input.parse().ok()
                };
            }
            FieldType::StrengthMobility => {
                log.strength_mobility = if input.trim().is_empty() {
                    None
                } else {
                    Some(input)
                };
            }
            FieldType::Notes => {
                log.notes = if input.trim().is_empty() {
                    None
                } else {
                    Some(input)
                };
            }
        }

        log.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AppState;

    #[test]
    fn test_weight_field_accessor() {
        let mut state = AppState::new();

        // Initially empty
        assert_eq!(FieldType::Weight.get_value(&state), "");

        // Update with value
        FieldType::Weight.update_value(&mut state, "175.5".to_string());
        assert_eq!(FieldType::Weight.get_value(&state), "175.5");

        // Update with empty (clears value)
        FieldType::Weight.update_value(&mut state, "".to_string());
        assert_eq!(FieldType::Weight.get_value(&state), "");
    }

    #[test]
    fn test_waist_field_accessor() {
        let mut state = AppState::new();

        assert_eq!(FieldType::Waist.get_value(&state), "");

        FieldType::Waist.update_value(&mut state, "34.2".to_string());
        assert_eq!(FieldType::Waist.get_value(&state), "34.2");
    }

    #[test]
    fn test_miles_field_accessor() {
        let mut state = AppState::new();

        assert_eq!(FieldType::Miles.get_value(&state), "");

        FieldType::Miles.update_value(&mut state, "5.3".to_string());
        assert_eq!(FieldType::Miles.get_value(&state), "5.3");
    }

    #[test]
    fn test_elevation_field_accessor() {
        let mut state = AppState::new();

        assert_eq!(FieldType::Elevation.get_value(&state), "");

        FieldType::Elevation.update_value(&mut state, "1200".to_string());
        assert_eq!(FieldType::Elevation.get_value(&state), "1200");
    }

    #[test]
    fn test_strength_mobility_field_accessor() {
        let mut state = AppState::new();

        assert_eq!(FieldType::StrengthMobility.get_value(&state), "");

        let exercises = "Pull-ups: 3x8\nPush-ups: 3x15".to_string();
        FieldType::StrengthMobility.update_value(&mut state, exercises.clone());
        assert_eq!(FieldType::StrengthMobility.get_value(&state), exercises);

        // Empty/whitespace clears it
        FieldType::StrengthMobility.update_value(&mut state, "   ".to_string());
        assert_eq!(FieldType::StrengthMobility.get_value(&state), "");
    }

    #[test]
    fn test_notes_field_accessor() {
        let mut state = AppState::new();

        assert_eq!(FieldType::Notes.get_value(&state), "");

        let note = "Great workout today!".to_string();
        FieldType::Notes.update_value(&mut state, note.clone());
        assert_eq!(FieldType::Notes.get_value(&state), note);
    }

    #[test]
    fn test_invalid_numeric_input() {
        let mut state = AppState::new();

        // Invalid numeric input should result in None (empty string)
        FieldType::Weight.update_value(&mut state, "not_a_number".to_string());
        assert_eq!(FieldType::Weight.get_value(&state), "");

        FieldType::Elevation.update_value(&mut state, "12.5".to_string()); // decimal not allowed for elevation
        assert_eq!(FieldType::Elevation.get_value(&state), "");
    }
}
