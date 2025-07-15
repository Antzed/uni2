# Requirements Document

## Introduction

This feature adds an interactive Terminal User Interface (TUI) to the uni CLI tool using the Ratatui library. The TUI will be activated with a `-i` flag and provide a visual, column-based navigation system for exploring and executing commands. Users can navigate through command hierarchies using a breadcrumb-style column layout, view help information, and execute commands interactively.

## Requirements

### Requirement 1

**User Story:** As a uni CLI user, I want to launch an interactive TUI mode with a `-i` flag, so that I can visually explore and execute commands without memorizing command syntax.

#### Acceptance Criteria

1. WHEN the user runs `uni -i` THEN the system SHALL launch a full-screen TUI interface
2. WHEN the TUI launches THEN the system SHALL display available top-level commands in the first column
3. WHEN the TUI is active THEN the system SHALL capture keyboard input for navigation
4. WHEN the user presses 'q' or 'Esc' THEN the system SHALL exit the TUI and return to the terminal

### Requirement 2

**User Story:** As a user navigating the TUI, I want to see commands organized in vertical columns, so that I can understand the command hierarchy and my current navigation path.

#### Acceptance Criteria

1. WHEN the TUI displays commands THEN the system SHALL show each command level in a separate vertical column
2. WHEN a user selects a command with subcommands THEN the system SHALL add a new column showing the subcommands
3. WHEN navigating deeper into command hierarchy THEN the system SHALL maintain all previous columns visible
4. WHEN the command path is `uni git s` THEN the system SHALL display columns as `| git | s |`
5. WHEN the user navigates back THEN the system SHALL remove the rightmost column

### Requirement 3

**User Story:** As a user exploring commands, I want to see help information for the currently selected command, so that I can understand what each command does before executing it.

#### Acceptance Criteria

1. WHEN a command is highlighted THEN the system SHALL display its help text in the rightmost area
2. WHEN no command is selected THEN the system SHALL show general navigation help
3. WHEN a command has a description THEN the system SHALL display the description from the manifest
4. WHEN a command has usage information THEN the system SHALL display usage examples

### Requirement 4

**User Story:** As a user in the TUI, I want to navigate using keyboard controls, so that I can efficiently move through the command hierarchy.

#### Acceptance Criteria

1. WHEN the user presses Up/Down arrows THEN the system SHALL move selection within the current column
2. WHEN the user presses Right arrow or Enter on a command with subcommands THEN the system SHALL navigate into that command
3. WHEN the user presses Left arrow or Backspace THEN the system SHALL navigate back to the previous level
4. WHEN the user presses Tab THEN the system SHALL move focus between columns
5. WHEN the user presses Home/End THEN the system SHALL jump to first/last item in current column

### Requirement 5

**User Story:** As a user ready to execute a command, I want to press Enter on a final command to execute it, so that I can run the selected command without leaving the TUI.

#### Acceptance Criteria

1. WHEN the user presses Enter on a leaf command (no subcommands) THEN the system SHALL execute that command
2. WHEN a command is executed THEN the system SHALL show the command output within the TUI
3. WHEN command execution completes THEN the system SHALL allow the user to return to navigation or exit
4. WHEN a command fails THEN the system SHALL display the error message clearly
5. WHEN executing a command THEN the system SHALL pass any required arguments or flags

### Requirement 6

**User Story:** As a user of the TUI, I want the interface to be responsive and visually clear, so that I can efficiently work with the command structure.

#### Acceptance Criteria

1. WHEN the terminal is resized THEN the system SHALL adjust the TUI layout accordingly
2. WHEN there are many commands THEN the system SHALL provide scrolling within columns
3. WHEN columns exceed terminal width THEN the system SHALL provide horizontal scrolling
4. WHEN displaying text THEN the system SHALL use clear visual indicators for selection and focus
5. WHEN showing command status THEN the system SHALL use appropriate colors and symbols

### Requirement 7

**User Story:** As a developer integrating the TUI, I want the interface to work with the existing plugin system, so that all dynamically loaded commands are available in the TUI.

#### Acceptance Criteria

1. WHEN the TUI loads THEN the system SHALL discover all installed plugins using the existing manifest system
2. WHEN displaying commands THEN the system SHALL include both built-in and plugin commands
3. WHEN a plugin has subcommands THEN the system SHALL display them in the column hierarchy
4. WHEN plugin manifests change THEN the system SHALL reflect updates without requiring TUI restart
5. WHEN executing plugin commands THEN the system SHALL use the same execution path as the CLI mode