# Design Document

## Overview

The TUI interface will be implemented using the Ratatui library, which provides a robust framework for building terminal user interfaces in Rust. The design follows a column-based navigation pattern where each level of the command hierarchy is represented as a separate vertical column, similar to file managers like Miller columns or macOS Finder.

The architecture separates concerns between the TUI rendering, state management, command discovery, and execution. This ensures the TUI integrates cleanly with the existing uni CLI infrastructure while providing a responsive and intuitive user experience.

## Architecture

### High-Level Components

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   TUI Module    │    │  State Manager  │    │ Command System  │
│                 │    │                 │    │                 │
│ - Event Loop    │◄──►│ - Navigation    │◄──►│ - Discovery     │
│ - Rendering     │    │ - Selection     │    │ - Execution     │
│ - Input Handler │    │ - History       │    │ - Manifests     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Module Structure

- **`tui/mod.rs`** - Main TUI module and public interface
- **`tui/app.rs`** - Application state and event handling
- **`tui/ui.rs`** - Rendering logic and layout management  
- **`tui/events.rs`** - Input event processing
- **`tui/commands.rs`** - Command discovery and execution integration

## Components and Interfaces

### TUI Application State

```rust
pub struct TuiApp {
    // Navigation state
    pub command_path: Vec<String>,           // Current command path (e.g., ["git", "s"])
    pub columns: Vec<CommandColumn>,         // Column data for each level
    pub active_column: usize,                // Currently focused column
    pub selected_indices: Vec<usize>,        // Selected item in each column
    
    // Command system integration
    pub manifests: Vec<Manifest>,            // Loaded plugin manifests
    pub built_ins: Vec<CommandInfo>,         // Built-in commands
    
    // UI state
    pub help_text: String,                   // Current help display
    pub status_message: Option<String>,      // Status/error messages
    pub execution_output: Option<String>,    // Command output display
    pub should_quit: bool,                   // Exit flag
}

pub struct CommandColumn {
    pub title: String,                       // Column header
    pub items: Vec<CommandItem>,             // Available commands
    pub scroll_offset: usize,                // For scrolling long lists
}

pub struct CommandItem {
    pub name: String,                        // Command name
    pub description: String,                 // Help text
    pub has_subcommands: bool,               // Whether it has children
    pub is_executable: bool,                 // Whether it's a leaf command
}
```

### Event System

```rust
pub enum TuiEvent {
    // Navigation events
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    
    // Action events
    Select,                                  // Enter key
    Execute,                                 // Execute current command
    Back,                                    // Go back one level
    
    // UI events
    NextColumn,                              // Tab key
    PrevColumn,                              // Shift+Tab
    ScrollUp,
    ScrollDown,
    
    // System events
    Quit,
    Resize(u16, u16),                        // Terminal resize
}
```

### Command Integration Interface

```rust
pub trait CommandProvider {
    fn get_root_commands(&self) -> Vec<CommandItem>;
    fn get_subcommands(&self, path: &[String]) -> Vec<CommandItem>;
    fn get_help_text(&self, path: &[String]) -> String;
    fn execute_command(&self, path: &[String], args: &[String]) -> Result<String, String>;
}

pub struct UniCommandProvider {
    manifests: Vec<Manifest>,
    built_ins: Vec<BuiltInCommand>,
}
```

## Data Models

### Command Hierarchy Representation

The TUI maintains a tree-like structure of commands that mirrors the existing CLI structure:

```rust
// Root level: built-ins + plugins
Root
├── add (built-in)
├── remove (built-in)  
├── list (built-in)
├── git (plugin)
│   ├── s (subcommand)
│   ├── status (subcommand)
│   └── push (subcommand)
└── docker (plugin)
    ├── build (subcommand)
    └── run (subcommand)
```

### State Transitions

```rust
// Navigation state machine
NavigationState {
    Browsing {
        column: usize,
        selection: usize,
    },
    Executing {
        command_path: Vec<String>,
        output: String,
    },
    ShowingHelp {
        command_path: Vec<String>,
        help_text: String,
    },
}
```

## Error Handling

### Error Types

```rust
#[derive(Debug)]
pub enum TuiError {
    // Terminal/rendering errors
    TerminalError(std::io::Error),
    RatatuiError(ratatui::Error),
    
    // Command system errors
    CommandNotFound(String),
    ExecutionError(String),
    ManifestError(String),
    
    // Navigation errors
    InvalidNavigation(String),
    ColumnOutOfBounds(usize),
}
```

### Error Recovery

- **Terminal errors**: Gracefully exit TUI and restore terminal state
- **Command errors**: Display error message in status area, allow continued navigation
- **Navigation errors**: Reset to safe state (root level)
- **Plugin errors**: Skip problematic plugins, log warnings

## Testing Strategy

### Unit Tests

1. **State Management Tests**
   - Navigation state transitions
   - Column management
   - Selection tracking
   - Command path building

2. **Command Integration Tests**
   - Plugin discovery
   - Command execution
   - Help text generation
   - Error handling

3. **Event Processing Tests**
   - Input event mapping
   - State updates
   - Edge cases (empty columns, invalid selections)

### Integration Tests

1. **TUI Workflow Tests**
   - Complete navigation flows
   - Command execution cycles
   - Error recovery scenarios

2. **Plugin Compatibility Tests**
   - Dynamic plugin loading
   - Manifest parsing
   - Subcommand discovery

### Manual Testing Scenarios

1. **Navigation Testing**
   - Multi-level command navigation
   - Column scrolling with many items
   - Terminal resize handling
   - Keyboard shortcut combinations

2. **Command Execution Testing**
   - Built-in command execution
   - Plugin command execution
   - Error command handling
   - Output display formatting

## Implementation Phases

### Phase 1: Core TUI Infrastructure
- Basic Ratatui setup and terminal management
- Column-based layout rendering
- Basic keyboard input handling
- Exit/quit functionality

### Phase 2: Command Integration
- Integration with existing command discovery
- Static command display (built-ins + plugins)
- Help text display
- Navigation between columns

### Phase 3: Interactive Navigation
- Full keyboard navigation
- Selection tracking
- Column scrolling
- Dynamic column management

### Phase 4: Command Execution
- Command execution integration
- Output display
- Error handling and display
- Status messaging

### Phase 5: Polish and Optimization
- Performance optimization
- Visual enhancements
- Edge case handling
- Documentation and testing