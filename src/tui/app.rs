// TUI application state and main event loop
use crate::tui::{events::TuiEvent, ui, commands::{CommandProvider, CommandColumn, UniCommandProvider}};
use crate::{load_manifests};
use ratatui::{backend::Backend, Terminal};

/// Main TUI application state
pub struct TuiApp {
    pub should_quit: bool,
    pub columns: Vec<CommandColumn>,
    pub active_column: usize,
    pub command_path: Vec<String>,
    pub command_provider: UniCommandProvider,
    pub status_message: Option<String>,
}

impl TuiApp {
    /// Create a new TUI application instance
    pub fn new() -> Self {
        let manifests = load_manifests();
        let command_provider = UniCommandProvider::new(manifests);
        
        // Initialize with root commands
        let root_commands = command_provider.get_root_commands();
        let root_column = CommandColumn {
            title: "Commands".to_string(),
            items: root_commands,
            scroll_offset: 0,
            selected_index: 0,
        };
        
        Self {
            should_quit: false,
            columns: vec![root_column],
            active_column: 0,
            command_path: vec![],
            command_provider,
            status_message: None,
        }
    }

    /// Main application run loop
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // Render the UI
            terminal.draw(|f| ui::render(f, self))?;

            // Handle events
            if let Ok(event) = TuiEvent::next() {
                self.handle_event(event)?;
            }

            // Check if we should quit
            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    /// Handle a TUI event
    fn handle_event(&mut self, event: TuiEvent) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            TuiEvent::Quit => {
                self.should_quit = true;
            }
            TuiEvent::MoveUp => {
                self.move_selection_up();
            }
            TuiEvent::MoveDown => {
                self.move_selection_down();
            }
            _ => {
                // Other events will be handled in later tasks
            }
        }
        Ok(())
    }

    /// Move selection up in the current column
    fn move_selection_up(&mut self) {
        if let Some(column) = self.columns.get_mut(self.active_column) {
            if column.selected_index > 0 {
                column.selected_index -= 1;
            }
        }
    }

    /// Move selection down in the current column
    fn move_selection_down(&mut self) {
        if let Some(column) = self.columns.get_mut(self.active_column) {
            if column.selected_index < column.items.len().saturating_sub(1) {
                column.selected_index += 1;
            }
        }
    }

    /// Get the currently selected command item
    pub fn get_selected_command(&self) -> Option<&crate::tui::commands::CommandItem> {
        self.columns
            .get(self.active_column)
            .and_then(|col| col.items.get(col.selected_index))
    }

    /// Get help text for the currently selected command
    pub fn get_current_help_text(&self) -> String {
        if let Some(selected) = self.get_selected_command() {
            selected.description.clone()
        } else {
            "No command selected".to_string()
        }
    }
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new()
    }
}