// TUI event handling and input processing
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, KeyEventKind};
use std::io;
use std::time::Duration;

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
    
    // Text input events
    Char(char),
    Backspace,
    Delete,
    
    // System events
    Quit,
    Resize(u16, u16),
    
    // Special navigation events
    Home,
    End,
}

/// Errors that can occur during event processing
#[derive(Debug)]
pub enum EventError {
    IoError(io::Error),
    Timeout,
}

impl From<io::Error> for EventError {
    fn from(err: io::Error) -> Self {
        EventError::IoError(err)
    }
}

impl std::fmt::Display for EventError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventError::IoError(err) => write!(f, "IO error: {}", err),
            EventError::Timeout => write!(f, "Event timeout"),
        }
    }
}

impl std::error::Error for EventError {}

impl TuiEvent {
    /// Get the next event from the terminal with timeout
    pub fn next() -> Result<TuiEvent, EventError> {
        Self::next_with_timeout(Duration::from_millis(100))
    }
    
    /// Get the next event from the terminal with a specified timeout
    pub fn next_with_timeout(timeout: Duration) -> Result<TuiEvent, EventError> {
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key_event) => {
                    // Only process key press events, ignore key release events
                    if key_event.kind == KeyEventKind::Press {
                        Ok(Self::from_key_event(key_event))
                    } else {
                        // Recursively try to get the next event if this was a key release
                        Self::next_with_timeout(timeout)
                    }
                }
                Event::Resize(width, height) => Ok(TuiEvent::Resize(width, height)),
                _ => {
                    // Ignore other events (mouse, focus, etc.) and try again
                    Self::next_with_timeout(timeout)
                }
            }
        } else {
            Err(EventError::Timeout)
        }
    }

    /// Convert a crossterm KeyEvent to a TuiEvent
    fn from_key_event(key_event: KeyEvent) -> TuiEvent {
        match key_event.code {
            // Navigation events
            KeyCode::Up => TuiEvent::MoveUp,
            KeyCode::Down => TuiEvent::MoveDown,
            KeyCode::Left => TuiEvent::MoveLeft,
            KeyCode::Right => TuiEvent::MoveRight,
            
            // Action events
            KeyCode::Enter => TuiEvent::Select,
            
            // Text input events
            KeyCode::Backspace => TuiEvent::Backspace,
            KeyCode::Delete => TuiEvent::Delete,
            
            // Column navigation
            KeyCode::Tab => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    TuiEvent::PrevColumn
                } else {
                    TuiEvent::NextColumn
                }
            }
            
            // Scrolling events
            KeyCode::PageUp => TuiEvent::ScrollUp,
            KeyCode::PageDown => TuiEvent::ScrollDown,
            
            // Special navigation
            KeyCode::Home => TuiEvent::Home,
            KeyCode::End => TuiEvent::End,
            
            // Quit events (Esc always quits, Ctrl+C for safety)
            KeyCode::Esc => TuiEvent::Quit,
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => TuiEvent::Quit,
            
            // Character input
            KeyCode::Char(c) => TuiEvent::Char(c),
            
            // Default fallback
            _ => TuiEvent::Back,
        }
    }
}