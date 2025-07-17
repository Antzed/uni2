#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Manifest, SubCmdMeta};
    use crate::tui::{app, commands, events};
    use crate::tui::commands::CommandProvider;

    // Mock the load_manifests function for testing
    fn create_test_manifests() -> Vec<Manifest> {
        vec![
            Manifest {
                name: "git".to_string(),
                description: "Git version control commands".to_string(),
                version: "1.0.0".to_string(),
                commands: vec![
                    SubCmdMeta {
                        name: "status".to_string(),
                        description: "Show git status".to_string(),
                    },
                    SubCmdMeta {
                        name: "push".to_string(),
                        description: "Push changes to remote".to_string(),
                    },
                ],
            },
        ]
    }

    #[test]
    fn test_selection_and_highlighting_functionality() {
        // Create TUI app with test data
        let manifests = create_test_manifests();
        let command_provider = commands::UniCommandProvider::new(manifests);
        
        let root_commands = command_provider.get_root_commands();
        let root_column = commands::CommandColumn {
            title: "Commands".to_string(),
            items: root_commands,
            scroll_offset: 0,
            selected_index: 0,
        };
        
        let mut app = app::TuiApp {
            should_quit: false,
            columns: vec![root_column],
            active_column: 0,
            command_path: vec![],
            command_provider,
            status_message: None,
        };

        // Test initial state
        assert!(!app.columns.is_empty(), "Should have at least one column");
        assert_eq!(app.active_column, 0, "Should start with first column active");
        
        if let Some(first_column) = app.columns.get(0) {
            assert_eq!(first_column.selected_index, 0, "Should start with first item selected");
            assert!(!first_column.items.is_empty(), "First column should have items");
        }
        
        // Test selection info
        let selection_info = app.get_selection_info();
        assert!(selection_info.contains("Column 1/1"), "Selection info should show column position");
        
        // Test help text
        let help_text = app.get_current_help_text();
        assert!(!help_text.is_empty(), "Help text should not be empty");
        assert!(help_text.contains("Command:"), "Help text should contain command info");
        
        // Test selection movement
        let initial_selection = app.columns[0].selected_index;
        let total_items = app.columns[0].items.len();
        
        if total_items > 1 {
            // Test move down
            app.handle_event(events::TuiEvent::MoveDown).unwrap();
            assert_eq!(app.columns[0].selected_index, initial_selection + 1, "Selection should move down");
            
            // Test move up
            app.handle_event(events::TuiEvent::MoveUp).unwrap();
            assert_eq!(app.columns[0].selected_index, initial_selection, "Selection should move back up");
            
            // Test move to bottom
            app.handle_event(events::TuiEvent::End).unwrap();
            assert_eq!(app.columns[0].selected_index, total_items - 1, "Selection should be at bottom");
            
            // Test move to top
            app.handle_event(events::TuiEvent::Home).unwrap();
            assert_eq!(app.columns[0].selected_index, 0, "Selection should be at top");
        }
        
        // Test selected command retrieval
        let selected = app.get_selected_command();
        assert!(selected.is_some(), "Should have a selected command");
        
        // Test help text updates with selection changes
        let help_before = app.get_current_help_text();
        if total_items > 1 {
            app.handle_event(events::TuiEvent::MoveDown).unwrap();
            let help_after = app.get_current_help_text();
            // Help text should potentially change when selection changes
            // (though it might be the same if commands have similar descriptions)
            assert!(!help_after.is_empty(), "Help text should still be available after selection change");
        }
    }

    #[test]
    fn test_selection_edge_cases() {
        // Test with empty column
        let mut app = app::TuiApp {
            should_quit: false,
            columns: vec![commands::CommandColumn {
                title: "Empty".to_string(),
                items: vec![],
                scroll_offset: 0,
                selected_index: 0,
            }],
            active_column: 0,
            command_path: vec![],
            command_provider: commands::UniCommandProvider::new(vec![]),
            status_message: None,
        };

        // Movement should not panic with empty column
        app.handle_event(events::TuiEvent::MoveUp).unwrap();
        app.handle_event(events::TuiEvent::MoveDown).unwrap();
        app.handle_event(events::TuiEvent::Home).unwrap();
        app.handle_event(events::TuiEvent::End).unwrap();
        
        // Should handle no selection gracefully
        let selected = app.get_selected_command();
        assert!(selected.is_none(), "Should have no selected command in empty column");
        
        let help_text = app.get_current_help_text();
        assert!(!help_text.is_empty(), "Should provide fallback help text");
    }

    #[test]
    fn test_status_message_updates() {
        let manifests = create_test_manifests();
        let command_provider = commands::UniCommandProvider::new(manifests);
        
        let root_commands = command_provider.get_root_commands();
        let root_column = commands::CommandColumn {
            title: "Commands".to_string(),
            items: root_commands,
            scroll_offset: 0,
            selected_index: 0,
        };
        
        let mut app = app::TuiApp {
            should_quit: false,
            columns: vec![root_column],
            active_column: 0,
            command_path: vec![],
            command_provider,
            status_message: None,
        };

        // Initial state should have no status message
        assert!(app.status_message.is_none(), "Should start with no status message");
        
        // Moving selection should update status message
        if app.columns[0].items.len() > 1 {
            app.handle_event(events::TuiEvent::MoveDown).unwrap();
            assert!(app.status_message.is_some(), "Should have status message after movement");
            
            if let Some(ref msg) = app.status_message {
                assert!(msg.contains("Selected:"), "Status message should indicate selection");
            }
        }
    }
}