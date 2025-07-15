// TUI event handling and input processing
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::io;

/// TUI events that can be processed by the application
#[derive(Debug, Clone)]
pub enum TuiEvent {
    // Navigation events
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    
    // Action events
    Select,
    Execute,
    Back,
    
    // UI events
    NextColumn,
    PrevColumn,
    ScrollUp,
    ScrollDown,
    
    // System events
    Quit,
    Resize(u16, u16),
}

impl TuiEvent {
    /// Get the next event from the terminal
    pub fn next() -> Result<TuiEvent, io::Error> {
        match event::read()? {
            Event::Key(key_event) => Ok(Self::from_key_event(key_event)),
            Event::Resize(width, height) => Ok(TuiEvent::Resize(width, height)),
            _ => Self::next(), // Ignore other events and try again
        }
    }

    /// Convert a crossterm KeyEvent to a TuiEvent
    fn from_key_event(key_event: KeyEvent) -> TuiEvent {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => TuiEvent::Quit,
            KeyCode::Up => TuiEvent::MoveUp,
            KeyCode::Down => TuiEvent::MoveDown,
            KeyCode::Left | KeyCode::Backspace => TuiEvent::MoveLeft,
            KeyCode::Right | KeyCode::Enter => TuiEvent::MoveRight,
            KeyCode::Tab => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    TuiEvent::PrevColumn
                } else {
                    TuiEvent::NextColumn
                }
            }
            KeyCode::PageUp => TuiEvent::ScrollUp,
            KeyCode::PageDown => TuiEvent::ScrollDown,
            KeyCode::Home => TuiEvent::MoveUp, // Will be refined in later tasks
            KeyCode::End => TuiEvent::MoveDown, // Will be refined in later tasks
            _ => TuiEvent::Select, // Default fallback
        }
    }
}