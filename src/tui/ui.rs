// TUI rendering and layout management
use crate::tui::app::TuiApp;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

/// Render the main TUI interface
pub fn render(f: &mut Frame, app: &TuiApp) {
    let size = f.area();

    // Create main layout: breadcrumb + content area
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),    // Breadcrumb area
            Constraint::Min(0),       // Content area
        ])
        .split(size);

    // Render breadcrumb
    render_breadcrumb(f, app, main_chunks[0]);

    // Check current execution state and render appropriate view
    if app.is_showing_output() {
        // Render command output view
        render_command_output(f, app, main_chunks[1]);
    } else if app.is_inputting_arguments() {
        // Render argument input view
        render_argument_input(f, app, main_chunks[1]);
    } else {
        // Render normal navigation view
        render_navigation_view(f, app, main_chunks[1]);
    }
}

/// Render the normal navigation view with columns and help panel
fn render_navigation_view(f: &mut Frame, app: &TuiApp, area: Rect) {
    // Calculate column widths - each column gets equal space, with help panel on the right
    let help_panel_width = 30; // Fixed width for help panel
    let available_width = area.width.saturating_sub(help_panel_width);
    
    // Create content layout: columns area + help panel
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(available_width),
            Constraint::Length(help_panel_width),
        ])
        .split(area);

    // Render command columns
    render_command_columns(f, app, content_chunks[0]);
    
    // Render help panel
    render_help_panel(f, app, content_chunks[1]);
}

/// Render the command output view
fn render_command_output(f: &mut Frame, app: &TuiApp, area: Rect) {
    if let Some((command, output, is_error)) = app.get_execution_output() {
        // Create layout: output area + status area
        let output_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),      // Output content area
                Constraint::Length(3),    // Status area
            ])
            .split(area);

        // Render command output
        let output_title = if is_error {
            format!(" Command Failed: {} ", command)
        } else {
            format!(" Command Output: {} ", command)
        };

        let output_paragraph = Paragraph::new(output.clone())
            .block(
                Block::default()
                    .title(output_title)
                    .borders(Borders::ALL)
                    .border_style(if is_error {
                        Style::default().fg(Color::Red)
                    } else {
                        Style::default().fg(Color::Green)
                    }),
            )
            .style(if is_error {
                Style::default().fg(Color::LightRed)
            } else {
                Style::default().fg(Color::White)
            })
            .wrap(ratatui::widgets::Wrap { trim: true });

        f.render_widget(output_paragraph, output_chunks[0]);

        // Render status information
        render_output_status_panel(f, app, output_chunks[1]);
    }
}

/// Render the command columns area
fn render_command_columns(f: &mut Frame, app: &TuiApp, area: Rect) {
    let num_columns = app.columns.len();
    if num_columns == 0 {
        return;
    }

    // Calculate column width - distribute available space evenly
    let column_width = area.width / num_columns as u16;
    
    // Create constraints for each column
    let mut constraints = Vec::new();
    for i in 0..num_columns {
        if i == num_columns - 1 {
            // Last column gets remaining space
            constraints.push(Constraint::Min(column_width));
        } else {
            constraints.push(Constraint::Length(column_width));
        }
    }

    // Create layout for columns
    let column_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    // Render each column
    for (i, column) in app.columns.iter().enumerate() {
        if let Some(chunk) = column_chunks.get(i) {
            render_command_column(f, app, column, *chunk, i == app.active_column);
        }
    }
}

/// Render a single command column
fn render_command_column(
    f: &mut Frame,
    _app: &TuiApp,
    column: &crate::tui::commands::CommandColumn,
    area: Rect,
    is_active: bool,
) {
    // Create list items from command items
    let list_items: Vec<ListItem> = column
        .items
        .iter()
        .enumerate()
        .map(|(i, cmd)| {
            let is_selected = i == column.selected_index;
            
            // Determine styling based on selection and active state
            let style = if is_selected && is_active {
                // Active column, selected item - bright highlight
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                // Inactive column, selected item - dimmed highlight
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if cmd.is_executable {
                // Executable commands in normal state
                Style::default().fg(Color::Green)
            } else if cmd.has_subcommands {
                // Commands with subcommands in normal state
                Style::default().fg(Color::Yellow)
            } else {
                // Default styling
                Style::default().fg(Color::White)
            };

            // Create display name with appropriate indicators
            let display_name = if cmd.has_subcommands {
                format!("{} →", cmd.name)
            } else if cmd.is_executable {
                format!("▶ {}", cmd.name)
            } else {
                cmd.name.clone()
            };

            ListItem::new(Line::from(Span::styled(display_name, style)))
        })
        .collect();

    // Create the list widget with enhanced styling
    let list = List::new(list_items)
        .block(
            Block::default()
                .title(format!(" {} ", column.title))
                .borders(Borders::ALL)
                .border_style(if is_active {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                }),
        );

    // Create list state for proper rendering
    let mut list_state = ListState::default();
    // Always set the selection to show which item is selected, even in inactive columns
    list_state.select(Some(column.selected_index));

    // Render the list
    f.render_stateful_widget(list, area, &mut list_state);
}

/// Render the help panel on the right side
fn render_help_panel(f: &mut Frame, app: &TuiApp, area: Rect) {
    // Split the help area into help content and status
    let help_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),      // Help content area
            Constraint::Length(3),    // Status area
        ])
        .split(area);

    // Render main help content
    render_help_content(f, app, help_chunks[0]);
    
    // Render status information
    render_status_panel(f, app, help_chunks[1]);
}

/// Render the main help content
fn render_help_content(f: &mut Frame, app: &TuiApp, area: Rect) {
    // Get help text for currently selected command
    let help_text = app.get_current_help_text();
    
    // Create help content with navigation instructions
    let help_content = format!(
        "{}\n\n--- Navigation ---\n↑/↓: Move selection\n←/→: Navigate levels\nTab: Switch columns\nHome/End: Jump to first/last\nq/Esc: Quit",
        help_text
    );

    let help_paragraph = Paragraph::new(help_content)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(help_paragraph, area);
}

/// Render the breadcrumb navigation at the top
fn render_breadcrumb(f: &mut Frame, app: &TuiApp, area: Rect) {
    let breadcrumb_text = app.get_command_path_breadcrumb();
    
    let breadcrumb_paragraph = Paragraph::new(breadcrumb_text)
        .block(
            Block::default()
                .title(" Command Path ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        )
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));

    f.render_widget(breadcrumb_paragraph, area);
}

/// Render the status panel at the bottom of the help area
fn render_status_panel(f: &mut Frame, app: &TuiApp, area: Rect) {
    // Get selection info and status message
    let selection_info = app.get_selection_info();
    let status_text = if let Some(ref msg) = app.status_message {
        format!("{}\n{}", selection_info, msg)
    } else {
        selection_info
    };

    let status_paragraph = Paragraph::new(status_text)
        .block(
            Block::default()
                .title(" Status ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(status_paragraph, area);
}

/// Render the argument input view
fn render_argument_input(f: &mut Frame, app: &TuiApp, area: Rect) {
    if let Some((prompt, input, cursor_position)) = app.get_argument_input() {
        // Create layout: input area + help area + status area
        let input_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),    // Input area
                Constraint::Min(5),       // Help area
                Constraint::Length(3),    // Status area
            ])
            .split(area);

        // Render input field
        let input_display = if cursor_position < input.len() {
            // Show cursor in the middle of text
            format!("{}|{}", &input[..cursor_position], &input[cursor_position..])
        } else {
            // Show cursor at the end
            format!("{}|", input)
        };

        let input_paragraph = Paragraph::new(input_display)
            .block(
                Block::default()
                    .title(" Enter Arguments ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            )
            .style(Style::default().fg(Color::White))
            .wrap(ratatui::widgets::Wrap { trim: false });

        f.render_widget(input_paragraph, input_chunks[0]);

        // Render help text
        let help_text = format!(
            "{}\n\nExamples:\n• For flags: --verbose --output file.txt\n• For values: value1 value2 \"quoted value\"\n• For options: -f --flag=value\n\n--- Input Controls ---\n←/→: Move cursor\nHome/End: Jump to start/end\nBackspace/Delete: Remove characters\nEnter: Execute command\nEsc: Cancel",
            prompt
        );

        let help_paragraph = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title(" Help ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .style(Style::default().fg(Color::White))
            .wrap(ratatui::widgets::Wrap { trim: true });

        f.render_widget(help_paragraph, input_chunks[1]);

        // Render status information
        render_input_status_panel(f, app, input_chunks[2]);
    }
}

/// Render the status panel when inputting arguments
fn render_input_status_panel(f: &mut Frame, app: &TuiApp, area: Rect) {
    let status_text = if let Some(ref msg) = app.status_message {
        msg.clone()
    } else {
        "Type arguments and press Enter to execute, or Esc to cancel".to_string()
    };

    let status_paragraph = Paragraph::new(status_text)
        .block(
            Block::default()
                .title(" Status ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(status_paragraph, area);
}

/// Render the status panel when showing command output
fn render_output_status_panel(f: &mut Frame, app: &TuiApp, area: Rect) {
    let status_text = if let Some(ref msg) = app.status_message {
        msg.clone()
    } else {
        "Press Enter/Esc/← to return to navigation".to_string()
    };

    let status_paragraph = Paragraph::new(status_text)
        .block(
            Block::default()
                .title(" Status ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(status_paragraph, area);
}