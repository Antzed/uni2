// TUI module for interactive terminal interface
pub mod app;
pub mod ui;
pub mod events;
pub mod commands;

pub use app::TuiApp;
pub use events::TuiEvent;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

/// Initialize and run the TUI interface
pub fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = TuiApp::new();
    let result = app.run(&mut terminal);

    // Restore terminal - ensure cleanup happens even if app.run() fails
    let cleanup_result = cleanup_terminal(&mut terminal);
    
    // Return the first error that occurred, prioritizing app errors over cleanup errors
    match (result, cleanup_result) {
        (Err(app_err), _) => Err(app_err),
        (Ok(_), Err(cleanup_err)) => Err(cleanup_err),
        (Ok(_), Ok(_)) => Ok(()),
    }
}

/// Clean up terminal state and restore normal mode
fn cleanup_terminal<B: ratatui::backend::Backend + std::io::Write>(
    terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}