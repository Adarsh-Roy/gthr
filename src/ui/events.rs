use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::Duration;
use anyhow::Result;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    Quit,
}

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn next_event(&self, timeout: Duration) -> Result<Option<AppEvent>> {
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key_event) => {
                    if key_event.kind == KeyEventKind::Press {
                        Ok(Some(AppEvent::Key(key_event)))
                    } else {
                        Ok(None)
                    }
                }
                _ => Ok(None),
            }
        } else {
            Ok(Some(AppEvent::Tick))
        }
    }
}

pub fn handle_key_event(key_event: KeyEvent) -> Option<AppAction> {
    // Check for Ctrl combinations first
    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
        match key_event.code {
            KeyCode::Char('e') => return Some(AppAction::Export),  // Ctrl+E for export output
            KeyCode::Char('h') => return Some(AppAction::ShowHelp),  // Ctrl+H for help
            KeyCode::Char('j') => return Some(AppAction::MoveDown),  // Ctrl+J for moving down
            KeyCode::Char('k') => return Some(AppAction::MoveUp),  // Ctrl+K for moving up
            _ => return None,  // Ignore other Ctrl combinations
        }
    }

    // Handle regular keys (no modifiers)
    match key_event.code {
        KeyCode::Esc => Some(AppAction::Escape),
        KeyCode::Enter => Some(AppAction::ToggleSelection),
        KeyCode::Backspace => Some(AppAction::SearchBackspace),

        // Arrow keys for navigation
        KeyCode::Up => Some(AppAction::MoveUp),
        KeyCode::Down => Some(AppAction::MoveDown),
        KeyCode::Left => Some(AppAction::MoveUp),
        KeyCode::Right => Some(AppAction::MoveDown),
        KeyCode::PageUp => Some(AppAction::PageUp),
        KeyCode::PageDown => Some(AppAction::PageDown),
        KeyCode::Home => Some(AppAction::MoveToTop),
        KeyCode::End => Some(AppAction::MoveToBottom),

        // Characters type into search (only if no modifiers)
        KeyCode::Char(c) if key_event.modifiers == KeyModifiers::NONE => Some(AppAction::SearchChar(c)),

        _ => None,
    }
}

#[derive(Debug, Clone)]
pub enum AppAction {
    Escape,
    ToggleSelection,
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    MoveToTop,
    MoveToBottom,
    Export,
    ShowHelp,
    SearchChar(char),
    SearchBackspace,
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
