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

pub fn handle_key_event(key_event: KeyEvent, mode: &crate::ui::app::AppMode) -> Option<AppAction> {
    use crate::ui::app::AppMode;

    // Handle file save mode differently
    if *mode == AppMode::FileSave {
        match key_event.code {
            KeyCode::Esc => return Some(AppAction::Escape),
            KeyCode::Enter => return Some(AppAction::FileSaveConfirm),
            KeyCode::Backspace => return Some(AppAction::FileSaveBackspace),
            KeyCode::Char(c) if key_event.modifiers == KeyModifiers::NONE => {
                return Some(AppAction::FileSaveChar(c));
            }
            _ => return None,
        }
    }
    // Check for Ctrl combinations
    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
        match key_event.code {
            KeyCode::Char('e') => return Some(AppAction::Export),
            KeyCode::Char('h') => return Some(AppAction::ShowHelp),
            KeyCode::Char('j') => return Some(AppAction::MoveDown),
            KeyCode::Char('k') => return Some(AppAction::MoveUp),
            KeyCode::Char('d') => return Some(AppAction::HalfPageDown),
            KeyCode::Char('u') => return Some(AppAction::HalfPageUp),
            KeyCode::Char('f') => return Some(AppAction::PageDown),
            KeyCode::Char('b') => return Some(AppAction::PageUp),
            KeyCode::Char('t') => return Some(AppAction::MoveToTop),
            KeyCode::Char('g') => return Some(AppAction::MoveToBottom),
            _ => return None,
        }
    }

    // Handle regular keys (no modifiers)
    match key_event.code {
        KeyCode::Esc => Some(AppAction::Escape),
        KeyCode::Enter => Some(AppAction::ToggleSelection),
        KeyCode::Backspace => Some(AppAction::SearchBackspace),

        KeyCode::Up => Some(AppAction::MoveUp),
        KeyCode::Down => Some(AppAction::MoveDown),
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
    HalfPageUp,
    HalfPageDown,
    PageUp,
    PageDown,
    MoveToTop,
    MoveToBottom,
    Export,
    ShowHelp,
    SearchChar(char),
    SearchBackspace,
    FileSaveChar(char),
    FileSaveBackspace,
    FileSaveConfirm,
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
