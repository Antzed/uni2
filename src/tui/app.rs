// TUI application state and main event loop
use crate::tui::{events::{TuiEvent, EventError}, ui, commands::{CommandProvider, CommandColumn, UniCommandProvider}};
use crate::{load_manifests};
use ratatui::{backend::Backend, Terminal};

/// Execution state for command output display
#[derive(Debug, Clone)]
pub enum ExecutionState {
    /// Normal navigation mode
    Browsing,
    /// Collecting arguments for command execution
    InputtingArguments {
        command_path: Vec<String>,
        prompt: String,
        input: String,
        cursor_position: usize,
    },
    /// Showing command execution output
    ShowingOutput {
        command: String,
        output: String,
        is_error: bool,
    },
    /// Transition to CLI mode for command execution
    TransitionToCli {
        command: String,
    },
}

/// Main TUI application state
pub struct TuiApp {
    pub should_quit: bool,
    pub columns: Vec<CommandColumn>,
    pub active_column: usize,
    pub command_path: Vec<String>,
    pub command_provider: UniCommandProvider,
    pub status_message: Option<String>,
    pub execution_state: ExecutionState,
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
            execution_state: ExecutionState::Browsing,
        }
    }

    /// Main application run loop
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // Render the UI
            terminal.draw(|f| ui::render(f, self))?;

            // Handle events with proper error handling
            match TuiEvent::next() {
                Ok(event) => {
                    self.handle_event(event)?;
                }
                Err(EventError::Timeout) => {
                    // Timeout is expected and not an error - continue the loop
                    continue;
                }
                Err(EventError::IoError(err)) => {
                    // IO errors are more serious and should be propagated
                    return Err(Box::new(err));
                }
            }

            // Check if we should quit
            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    /// Handle a TUI event
    pub fn handle_event(&mut self, event: TuiEvent) -> Result<(), Box<dyn std::error::Error>> {
        // Handle events differently based on execution state
        match &self.execution_state {
            ExecutionState::Browsing => {
                self.handle_browsing_event(event)
            }
            ExecutionState::InputtingArguments { .. } => {
                self.handle_input_event(event)
            }
            ExecutionState::ShowingOutput { .. } => {
                self.handle_output_event(event)
            }
            ExecutionState::TransitionToCli { .. } => {
                // In transition state, just quit immediately
                self.should_quit = true;
                Ok(())
            }
        }
    }

    /// Handle events while in browsing mode
    fn handle_browsing_event(&mut self, event: TuiEvent) -> Result<(), Box<dyn std::error::Error>> {
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
            TuiEvent::MoveRight => {
                self.navigate_forward();
            }
            TuiEvent::Select => {
                self.handle_select();
            }
            TuiEvent::MoveLeft | TuiEvent::Back => {
                self.navigate_back();
            }
            TuiEvent::NextColumn => {
                self.move_to_next_column();
            }
            TuiEvent::PrevColumn => {
                self.move_to_prev_column();
            }
            TuiEvent::Home => {
                self.move_selection_to_top();
            }
            TuiEvent::End => {
                self.move_selection_to_bottom();
            }
            TuiEvent::Resize(width, height) => {
                // Handle terminal resize - for now just continue, 
                // detailed resize handling will be in later tasks
                self.status_message = Some(format!("Terminal resized to {}x{}", width, height));
            }
            _ => {
                // Other events will be handled in later tasks
            }
        }
        Ok(())
    }

    /// Handle events while showing command output
    fn handle_output_event(&mut self, event: TuiEvent) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            TuiEvent::Quit => {
                self.should_quit = true;
            }
            TuiEvent::Back | TuiEvent::MoveLeft | TuiEvent::Select => {
                // Return to browsing mode
                self.execution_state = ExecutionState::Browsing;
                self.update_status_message("Returned to navigation".to_string());
            }
            TuiEvent::Resize(width, height) => {
                self.status_message = Some(format!("Terminal resized to {}x{}", width, height));
            }
            _ => {
                // Ignore other events while showing output
            }
        }
        Ok(())
    }

    /// Handle events while inputting arguments
    fn handle_input_event(&mut self, event: TuiEvent) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            TuiEvent::Quit => {
                // Cancel argument input and return to browsing
                self.execution_state = ExecutionState::Browsing;
                self.update_status_message("Command execution cancelled".to_string());
            }
            TuiEvent::Select => {
                // Execute command with the entered arguments
                self.execute_with_arguments();
            }
            TuiEvent::Char(c) => {
                // Add character to input
                if let ExecutionState::InputtingArguments { input, cursor_position, .. } = &mut self.execution_state {
                    input.insert(*cursor_position, c);
                    *cursor_position += 1;
                }
            }
            TuiEvent::Backspace => {
                // Remove character before cursor
                if let ExecutionState::InputtingArguments { input, cursor_position, .. } = &mut self.execution_state {
                    if *cursor_position > 0 {
                        *cursor_position -= 1;
                        input.remove(*cursor_position);
                    }
                }
            }
            TuiEvent::Delete => {
                // Remove character at cursor
                if let ExecutionState::InputtingArguments { input, cursor_position, .. } = &mut self.execution_state {
                    if *cursor_position < input.len() {
                        input.remove(*cursor_position);
                    }
                }
            }
            TuiEvent::MoveLeft => {
                // Move cursor left
                if let ExecutionState::InputtingArguments { cursor_position, .. } = &mut self.execution_state {
                    if *cursor_position > 0 {
                        *cursor_position -= 1;
                    }
                }
            }
            TuiEvent::MoveRight => {
                // Move cursor right
                if let ExecutionState::InputtingArguments { input, cursor_position, .. } = &mut self.execution_state {
                    if *cursor_position < input.len() {
                        *cursor_position += 1;
                    }
                }
            }
            TuiEvent::Home => {
                // Move cursor to beginning
                if let ExecutionState::InputtingArguments { cursor_position, .. } = &mut self.execution_state {
                    *cursor_position = 0;
                }
            }
            TuiEvent::End => {
                // Move cursor to end
                if let ExecutionState::InputtingArguments { input, cursor_position, .. } = &mut self.execution_state {
                    *cursor_position = input.len();
                }
            }
            TuiEvent::Resize(width, height) => {
                self.status_message = Some(format!("Terminal resized to {}x{}", width, height));
            }
            _ => {
                // Ignore other events during input
            }
        }
        Ok(())
    }

    /// Move selection up in the current column
    fn move_selection_up(&mut self) {
        let mut status_msg = None;
        
        if let Some(column) = self.columns.get_mut(self.active_column) {
            if !column.items.is_empty() && column.selected_index > 0 {
                column.selected_index -= 1;
                if let Some(item) = column.items.get(column.selected_index) {
                    status_msg = Some(format!("Selected: {}", item.name));
                }
            }
        }
        
        if let Some(msg) = status_msg {
            self.update_status_message(msg);
        }
    }

    /// Move selection down in the current column
    fn move_selection_down(&mut self) {
        let mut status_msg = None;
        
        if let Some(column) = self.columns.get_mut(self.active_column) {
            if !column.items.is_empty() && column.selected_index < column.items.len().saturating_sub(1) {
                column.selected_index += 1;
                if let Some(item) = column.items.get(column.selected_index) {
                    status_msg = Some(format!("Selected: {}", item.name));
                }
            }
        }
        
        if let Some(msg) = status_msg {
            self.update_status_message(msg);
        }
    }

    /// Move selection to the top of the current column
    fn move_selection_to_top(&mut self) {
        let mut status_msg = None;
        
        if let Some(column) = self.columns.get_mut(self.active_column) {
            if !column.items.is_empty() {
                column.selected_index = 0;
                if let Some(item) = column.items.get(0) {
                    status_msg = Some(format!("Selected: {}", item.name));
                }
            }
        }
        
        if let Some(msg) = status_msg {
            self.update_status_message(msg);
        }
    }

    /// Move selection to the bottom of the current column
    fn move_selection_to_bottom(&mut self) {
        let mut status_msg = None;
        
        if let Some(column) = self.columns.get_mut(self.active_column) {
            if !column.items.is_empty() {
                let last_index = column.items.len() - 1;
                column.selected_index = last_index;
                if let Some(item) = column.items.get(last_index) {
                    status_msg = Some(format!("Selected: {}", item.name));
                }
            }
        }
        
        if let Some(msg) = status_msg {
            self.update_status_message(msg);
        }
    }

    /// Update the status message
    fn update_status_message(&mut self, message: String) {
        self.status_message = Some(message);
    }

    /// Clear the status message
    pub fn clear_status_message(&mut self) {
        self.status_message = None;
    }

    /// Get the current selection info for display
    pub fn get_selection_info(&self) -> String {
        if let Some(column) = self.columns.get(self.active_column) {
            if let Some(selected) = column.items.get(column.selected_index) {
                format!("Column {}/{}: {} ({}/{})", 
                    self.active_column + 1, 
                    self.columns.len(),
                    selected.name,
                    column.selected_index + 1,
                    column.items.len())
            } else {
                format!("Column {}/{}: No selection", 
                    self.active_column + 1, 
                    self.columns.len())
            }
        } else {
            "No active column".to_string()
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
            // Build the current command path including the selected command
            let mut current_path = self.command_path.clone();
            current_path.push(selected.name.clone());
            
            // Get detailed help text from the command provider
            let help_text = self.command_provider.get_help_text(&current_path);
            
            // Add additional context about the command
            let mut full_help = format!("Command: {}\n\n{}", selected.name, help_text);
            
            // Add execution status
            if selected.is_executable {
                full_help.push_str("\n\n[Press Enter to execute]");
            }
            
            // Add navigation hint for commands with subcommands
            if selected.has_subcommands {
                full_help.push_str("\n\n[Press → or Enter to explore subcommands]");
            }
            
            full_help
        } else if self.columns.is_empty() {
            "No commands available".to_string()
        } else {
            "Use ↑/↓ to select a command".to_string()
        }
    }

    /// Navigate forward into a command with subcommands
    fn navigate_forward(&mut self) {
        // Clone the selected command to avoid borrowing issues
        if let Some(selected) = self.get_selected_command().cloned() {
            if selected.has_subcommands {
                // Add the selected command to the path
                self.command_path.push(selected.name.clone());
                
                // Get subcommands for the new path
                let subcommands = self.command_provider.get_subcommands(&self.command_path);
                
                if !subcommands.is_empty() {
                    // Create a new column for the subcommands
                    let new_column = CommandColumn {
                        title: selected.name.clone(),
                        items: subcommands,
                        scroll_offset: 0,
                        selected_index: 0,
                    };
                    
                    // Add the new column
                    self.columns.push(new_column);
                    
                    // Move active column to the new column
                    self.active_column = self.columns.len() - 1;
                    
                    self.update_status_message(format!("Navigated to {}", selected.name));
                } else {
                    // No subcommands found, remove from path
                    self.command_path.pop();
                    self.update_status_message(format!("No subcommands found for {}", selected.name));
                }
            } else if selected.is_executable {
                // For executable commands, we could execute them here
                // For now, just show a message
                self.update_status_message(format!("Command {} is executable (execution not implemented yet)", selected.name));
            } else {
                self.update_status_message(format!("{} has no subcommands", selected.name));
            }
        }
    }

    /// Navigate back to the previous level
    fn navigate_back(&mut self) {
        if self.columns.len() > 1 {
            // Remove the rightmost column
            self.columns.pop();
            
            // Remove the last command from the path
            if let Some(removed_command) = self.command_path.pop() {
                // Move active column to the new rightmost column
                self.active_column = self.columns.len().saturating_sub(1);
                
                self.update_status_message(format!("Navigated back from {}", removed_command));
            }
        } else {
            // Already at root level
            self.update_status_message("Already at root level".to_string());
        }
    }

    /// Move focus to the next column (Tab navigation)
    fn move_to_next_column(&mut self) {
        if self.columns.len() > 1 {
            self.active_column = (self.active_column + 1) % self.columns.len();
            self.update_status_message(format!("Moved to column {}", self.active_column + 1));
        }
    }

    /// Move focus to the previous column (Shift+Tab navigation)
    fn move_to_prev_column(&mut self) {
        if self.columns.len() > 1 {
            self.active_column = if self.active_column == 0 {
                self.columns.len() - 1
            } else {
                self.active_column - 1
            };
            self.update_status_message(format!("Moved to column {}", self.active_column + 1));
        }
    }

    /// Handle the Select (Enter) key - either navigate or execute
    fn handle_select(&mut self) {
        // Clone the selected command to avoid borrowing issues
        if let Some(selected) = self.get_selected_command().cloned() {
            if selected.has_subcommands {
                // If command has subcommands, navigate into it
                self.navigate_forward();
            } else if selected.is_executable {
                // If command is executable (leaf command), execute it
                self.execute_command();
            } else {
                // Command is neither navigable nor executable
                self.update_status_message(format!("{} cannot be executed or navigated", selected.name));
            }
        }
    }

    /// Execute the currently selected command
    fn execute_command(&mut self) {
        if let Some(selected) = self.get_selected_command().cloned() {
            // Build the full command path including the selected command
            let mut full_path = self.command_path.clone();
            full_path.push(selected.name.clone());
            
            // Start argument input mode
            let prompt = format!("Enter arguments for 'uni {}' (or press Enter for none):", full_path.join(" "));
            
            self.execution_state = ExecutionState::InputtingArguments {
                command_path: full_path,
                prompt,
                input: String::new(),
                cursor_position: 0,
            };
            
            self.update_status_message("Type arguments and press Enter to execute, or Esc to cancel".to_string());
        }
    }

    /// Check if currently showing command output
    pub fn is_showing_output(&self) -> bool {
        matches!(self.execution_state, ExecutionState::ShowingOutput { .. })
    }

    /// Get the current execution output for display
    pub fn get_execution_output(&self) -> Option<(String, String, bool)> {
        match &self.execution_state {
            ExecutionState::ShowingOutput { command, output, is_error } => {
                Some((command.clone(), output.clone(), *is_error))
            }
            ExecutionState::Browsing => None,
            ExecutionState::InputtingArguments { .. } => None,
            ExecutionState::TransitionToCli { .. } => None,
        }
    }

    /// Get the current command path as a breadcrumb string
    pub fn get_command_path_breadcrumb(&self) -> String {
        if self.command_path.is_empty() {
            "uni".to_string()
        } else {
            format!("uni {}", self.command_path.join(" "))
        }
    }

    /// Check if transitioning to CLI mode
    pub fn is_transitioning_to_cli(&self) -> bool {
        matches!(self.execution_state, ExecutionState::TransitionToCli { .. })
    }

    /// Get the command to execute in CLI mode
    pub fn get_cli_command(&self) -> Option<String> {
        match &self.execution_state {
            ExecutionState::TransitionToCli { command } => Some(command.clone()),
            _ => None,
        }
    }

    /// Execute command with the arguments entered by the user
    fn execute_with_arguments(&mut self) {
        if let ExecutionState::InputtingArguments { command_path, input, .. } = &self.execution_state {
            let command_path = command_path.clone();
            let args_input = input.trim();
            
            // Parse arguments (simple space-separated for now)
            let args: Vec<String> = if args_input.is_empty() {
                Vec::new()
            } else {
                args_input.split_whitespace().map(|s| s.to_string()).collect()
            };
            
            // Build the full command string for CLI execution
            let mut command_parts = vec!["uni".to_string()];
            command_parts.extend(command_path.clone());
            command_parts.extend(args.clone());
            let full_command = command_parts.join(" ");
            
            // Transition to CLI mode for real execution
            self.execution_state = ExecutionState::TransitionToCli {
                command: full_command,
            };
            self.should_quit = true; // Exit TUI to transition to CLI
        }
    }

    /// Check if currently inputting arguments
    pub fn is_inputting_arguments(&self) -> bool {
        matches!(self.execution_state, ExecutionState::InputtingArguments { .. })
    }

    /// Get the current argument input state for display
    pub fn get_argument_input(&self) -> Option<(String, String, usize)> {
        match &self.execution_state {
            ExecutionState::InputtingArguments { prompt, input, cursor_position, .. } => {
                Some((prompt.clone(), input.clone(), *cursor_position))
            }
            _ => None,
        }
    }
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new()
    }
}