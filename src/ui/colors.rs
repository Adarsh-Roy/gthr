use ratatui::style::{Color, Style};
use crate::directory::state::SelectionState;

pub struct ColorScheme {
    pub included: Style,
    pub excluded: Style,
    pub partial: Style,
    pub selected: Style,
    pub search_match: Style,
    pub background: Style,
    pub border: Style,
    pub text: Style,
    pub help_text: Style,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            included: Style::default().fg(Color::Green),
            excluded: Style::default().fg(Color::Red),
            partial: Style::default().fg(Color::Yellow),
            selected: Style::default().bg(Color::Blue).fg(Color::White),
            search_match: Style::default().fg(Color::Cyan),
            background: Style::default().bg(Color::Black),
            border: Style::default().fg(Color::White),
            text: Style::default().fg(Color::White),
            help_text: Style::default().fg(Color::Gray),
        }
    }
}

impl ColorScheme {
    pub fn get_state_style(&self, state: SelectionState) -> Style {
        match state {
            SelectionState::Included => self.included,
            SelectionState::Excluded => self.excluded,
            SelectionState::Partial => self.partial,
        }
    }

    pub fn get_item_style(&self, state: SelectionState, is_selected: bool) -> Style {
        if is_selected {
            // Override with selected style for clear visibility
            self.selected
        } else {
            self.get_state_style(state)
        }
    }
}