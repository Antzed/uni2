// TUI rendering and layout management
use crate::tui::app::TuiApp;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render the main TUI interface
pub fn render(f: &mut Frame, app: &TuiApp) {
    let size = f.area();

    // Create basic layout - will be expanded in later tasks
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(size);

    // Main content area (left side)
    let main_block = Block::default()
        .title("Commands")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));
    
    let main_content = Paragraph::new("TUI Interface - Press 'q' or 'Esc' to quit")
        .block(main_block);
    f.render_widget(main_content, chunks[0]);

    // Help area (right side)
    let help_block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));
    
    let help_content = Paragraph::new("Navigation help will appear here")
        .block(help_block);
    f.render_widget(help_content, chunks[1]);
}