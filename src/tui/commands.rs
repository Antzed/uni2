// Command discovery and execution integration
use crate::{Manifest, plugin_dir};
use std::process::Command as StdCommand;
use std::io::{self, Write};

/// Data structure representing a command item in the TUI
#[derive(Debug, Clone)]
pub struct CommandItem {
    pub name: String,
    pub description: String,
    pub has_subcommands: bool,
    pub is_executable: bool,
}

/// Data structure representing a column of commands
#[derive(Debug, Clone)]
pub struct CommandColumn {
    pub title: String,
    pub items: Vec<CommandItem>,
    pub scroll_offset: usize,
    pub selected_index: usize,
}

/// Trait for abstracting command sources
pub trait CommandProvider {
    /// Get the list of root-level commands
    fn get_root_commands(&self) -> Vec<CommandItem>;
    
    /// Get subcommands for a given command path
    fn get_subcommands(&self, path: &[String]) -> Vec<CommandItem>;
    
    /// Get help text for a command at the given path
    fn get_help_text(&self, path: &[String]) -> String;
    
    /// Execute a command with the given path and arguments
    fn execute_command(&self, path: &[String], args: &[String]) -> Result<String, String>;
}

/// Implementation of CommandProvider that integrates with existing uni CLI system
pub struct UniCommandProvider {
    pub manifests: Vec<Manifest>,
}

impl UniCommandProvider {
    /// Create a new UniCommandProvider instance
    pub fn new(manifests: Vec<Manifest>) -> Self {
        Self { manifests }
    }

    /// Discover built-in commands from the BuiltIn enum
    fn get_builtin_commands(&self) -> Vec<CommandItem> {
        vec![
            CommandItem {
                name: "add".to_string(),
                description: "Add a new plugin from a script file".to_string(),
                has_subcommands: false,
                is_executable: true,
            },
            CommandItem {
                name: "remove".to_string(),
                description: "Remove an installed plugin by name".to_string(),
                has_subcommands: false,
                is_executable: true,
            },
            CommandItem {
                name: "list".to_string(),
                description: "List all installed plugins".to_string(),
                has_subcommands: false,
                is_executable: true,
            },
            CommandItem {
                name: "create".to_string(),
                description: "Create a new plugin template".to_string(),
                has_subcommands: false,
                is_executable: true,
            },
            CommandItem {
                name: "export".to_string(),
                description: "Export plugins to a zip file".to_string(),
                has_subcommands: false,
                is_executable: true,
            },
            CommandItem {
                name: "import".to_string(),
                description: "Import plugins from a zip file".to_string(),
                has_subcommands: false,
                is_executable: true,
            },
            CommandItem {
                name: "ensure-python".to_string(),
                description: "Ensure Python 3.13.3 and uv are installed".to_string(),
                has_subcommands: false,
                is_executable: true,
            },
        ]
    }

    /// Load plugin commands from manifests
    fn get_plugin_commands(&self) -> Vec<CommandItem> {
        let mut plugin_commands = Vec::new();
        
        for manifest in &self.manifests {
            let has_subcommands = !manifest.commands.is_empty();
            
            plugin_commands.push(CommandItem {
                name: manifest.name.clone(),
                description: manifest.description.clone(),
                has_subcommands,
                is_executable: !has_subcommands, // If no subcommands, it's directly executable
            });
        }
        
        plugin_commands
    }

    /// Find a manifest by plugin name
    fn find_manifest(&self, plugin_name: &str) -> Option<&Manifest> {
        self.manifests.iter().find(|m| m.name == plugin_name)
    }

    /// Get subcommands for a specific plugin
    fn get_plugin_subcommands(&self, plugin_name: &str) -> Vec<CommandItem> {
        if let Some(manifest) = self.find_manifest(plugin_name) {
            manifest.commands.iter().map(|cmd| CommandItem {
                name: cmd.name.clone(),
                description: cmd.description.clone(),
                has_subcommands: false, // Plugin subcommands are leaf nodes
                is_executable: true,
            }).collect()
        } else {
            vec![]
        }
    }

    /// Check if a command is a built-in command
    fn is_builtin_command(&self, command_name: &str) -> bool {
        matches!(command_name, 
            "add" | "remove" | "list" | "create" | "export" | "import" | "ensure-python"
        )
    }

    /// Execute a built-in command using real CLI functionality
    fn execute_builtin_command(&self, path: &[String], _args: &[String]) -> Result<String, String> {
        if path.is_empty() {
            return Err("No command specified".to_string());
        }

        let command_name = &path[0];
        
        match command_name.as_str() {
            "list" => {
                // Capture output from the real list_plugins function
                self.capture_list_output()
            }
            "ensure-python" => {
                // Execute real ensure-python functionality
                self.execute_ensure_python()
            }
            "add" => {
                Err("Interactive add command not supported in TUI mode.\nPlease use the CLI version: uni add <plugin-file>".to_string())
            }
            "remove" => {
                Err("Interactive remove command not supported in TUI mode.\nPlease use the CLI version: uni remove <plugin-name>".to_string())
            }
            "create" => {
                Err("Interactive create command not supported in TUI mode.\nPlease use the CLI version: uni create <plugin-name>".to_string())
            }
            "export" => {
                Err("Interactive export command not supported in TUI mode.\nPlease use the CLI version: uni export <output-file>".to_string())
            }
            "import" => {
                Err("Interactive import command not supported in TUI mode.\nPlease use the CLI version: uni import <zip-file>".to_string())
            }
            _ => {
                Err(format!("Unknown built-in command: '{}'\n\nAvailable built-in commands:\n- add: Add a new plugin\n- remove: Remove a plugin\n- list: List installed plugins\n- create: Create a plugin template\n- export: Export plugins to zip\n- import: Import plugins from zip\n- ensure-python: Check Python setup", command_name))
            }
        }
    }

    /// Capture output from the list_plugins function
    fn capture_list_output(&self) -> Result<String, String> {
        // Try to read plugins directly from the plugin directory
        let plugin_dir = plugin_dir();
        
        if !plugin_dir.exists() {
            return Ok("No plugins installed.\n\nTo add plugins, use the 'add' command or place plugin files in the plugins directory.".to_string());
        }

        let mut output = String::new();
        let mut plugin_count = 0;

        match std::fs::read_dir(&plugin_dir) {
            Ok(entries) => {
                let mut plugins = Vec::new();
                
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.extension().and_then(|e| e.to_str()) == Some("json") {
                            if let Ok(data) = std::fs::read(&path) {
                                if let Ok(manifest) = serde_json::from_slice::<crate::Manifest>(&data) {
                                    plugins.push(manifest);
                                }
                            }
                        }
                    }
                }

                if plugins.is_empty() {
                    output.push_str("No plugins installed.\n\nTo add plugins, use the 'add' command or place plugin files in the plugins directory.");
                } else {
                    output.push_str("Installed plugins:\n\n");
                    for (i, manifest) in plugins.iter().enumerate() {
                        plugin_count += 1;
                        output.push_str(&format!("{}. {} (v{})\n", i + 1, manifest.name, manifest.version));
                        output.push_str(&format!("   Description: {}\n", manifest.description));
                        if !manifest.commands.is_empty() {
                            output.push_str(&format!("   Commands: {}\n", 
                                manifest.commands.iter()
                                    .map(|cmd| cmd.name.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ));
                        }
                        output.push('\n');
                    }
                    output.push_str(&format!("Total: {} plugin(s) installed", plugin_count));
                }
            }
            Err(e) => {
                return Err(format!("Failed to read plugin directory: {}", e));
            }
        }

        Ok(output)
    }

    /// Execute the ensure-python command with real functionality
    fn execute_ensure_python(&self) -> Result<String, String> {
        let mut output = String::new();
        
        // Check current Python version
        let python_status = self.check_python_version();
        output.push_str(&python_status);
        output.push('\n');
        
        // Check current uv version
        let uv_status = self.check_uv_version();
        output.push_str(&uv_status);
        
        Ok(output)
    }

    /// Check Python version
    fn check_python_version(&self) -> String {
        let candidates = ["python3", "python"];
        for exe in &candidates {
            if let Ok(output) = StdCommand::new(exe).arg("--version").output() {
                // stdout on *nix, stderr on Windows; concatenate for safety
                let buf = [output.stdout, output.stderr].concat();
                let text = String::from_utf8_lossy(&buf);
                // expect `Python 3.13.3`
                if text.starts_with("Python") {
                    if let Some(version) = text.split_whitespace().nth(1) {
                        if version == "3.13.3" {
                            return format!("✅ Python 3.13.3 is installed and ready");
                        } else {
                            return format!("ℹ️  Python {} is installed (target: 3.13.3)", version);
                        }
                    }
                }
            }
        }
        "🚫 Python not found - please install Python 3.13.3".to_string()
    }

    /// Check uv version
    fn check_uv_version(&self) -> String {
        if let Ok(output) = StdCommand::new("uv").arg("--version").output() {
            let text = String::from_utf8_lossy(&output.stdout);
            if text.starts_with("uv ") {
                if let Some(version) = text.split_whitespace().nth(1) {
                    return format!("✅ uv {} is installed and ready", version);
                }
            }
        }
        "🚫 uv not found - please install uv package manager".to_string()
    }

    /// Execute a plugin command using real plugin script execution
    fn execute_plugin_command(&self, manifest: &Manifest, path: &[String], args: &[String]) -> Result<String, String> {
        if path.len() == 1 {
            // Direct plugin execution (for plugins without subcommands)
            if manifest.commands.is_empty() {
                // Execute the plugin directly with no subcommand
                self.execute_real_plugin(&manifest.name, &[], args)
            } else {
                Err(format!("Plugin '{}' has subcommands and cannot be executed directly.\n\nAvailable subcommands:\n{}\n\nPlease navigate to a specific subcommand to execute it.", 
                    manifest.name,
                    manifest.commands.iter()
                        .map(|cmd| format!("- {}: {}", cmd.name, cmd.description))
                        .collect::<Vec<_>>()
                        .join("\n")
                ))
            }
        } else if path.len() == 2 {
            // Plugin subcommand execution
            let subcommand_name = &path[1];
            
            if let Some(_subcmd) = manifest.commands.iter().find(|cmd| cmd.name == *subcommand_name) {
                // Execute the plugin with the subcommand
                self.execute_real_plugin(&manifest.name, &[subcommand_name], args)
            } else {
                Err(format!("Subcommand '{}' not found in plugin '{}'.\n\nAvailable subcommands:\n{}", 
                    subcommand_name, 
                    manifest.name,
                    manifest.commands.iter()
                        .map(|cmd| format!("- {}: {}", cmd.name, cmd.description))
                        .collect::<Vec<_>>()
                        .join("\n")
                ))
            }
        } else {
            Err(format!("Invalid command path length: {}\n\nPlugin commands support the following patterns:\n- Direct plugin execution: uni <plugin-name>\n- Plugin subcommand: uni <plugin-name> <subcommand>\n\nDeeper nesting is not supported.", path.len()))
        }
    }

    /// Execute a real plugin script and capture its output
    fn execute_real_plugin(&self, plugin_name: &str, subcommand: &[&str], args: &[String]) -> Result<String, String> {
        let script_path = plugin_dir().join(plugin_name);
        
        if !script_path.exists() {
            return Err(format!("Plugin script not found: {}", script_path.display()));
        }

        // Build the command arguments
        let mut cmd_args = Vec::new();
        
        // Add subcommand if provided
        for subcmd in subcommand {
            cmd_args.push(subcmd.to_string());
        }
        
        // Add additional arguments
        for arg in args {
            cmd_args.push(arg.clone());
        }

        // Determine how to execute the plugin based on its extension
        let output = if script_path.extension().and_then(|e| e.to_str()) == Some("py") {
            // Python script - execute with uv run
            StdCommand::new("uv")
                .args(["run", script_path.to_str().unwrap()])
                .args(&cmd_args)
                .output()
        } else {
            // Executable binary - execute directly
            StdCommand::new(&script_path)
                .args(&cmd_args)
                .output()
        };

        match output {
            Ok(output) => {
                if output.status.success() {
                    // Command succeeded - return stdout
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    
                    let mut result = String::new();
                    if !stdout.is_empty() {
                        result.push_str(&stdout);
                    }
                    if !stderr.is_empty() {
                        if !result.is_empty() {
                            result.push_str("\n--- stderr ---\n");
                        }
                        result.push_str(&stderr);
                    }
                    
                    if result.is_empty() {
                        result = format!("Plugin '{}' executed successfully with no output.", plugin_name);
                    }
                    
                    Ok(result)
                } else {
                    // Command failed - return error with stderr
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    
                    let mut error_msg = format!("Plugin '{}' execution failed (exit code: {})", 
                        plugin_name, 
                        output.status.code().unwrap_or(-1)
                    );
                    
                    if !stderr.is_empty() {
                        error_msg.push_str("\n\nError output:\n");
                        error_msg.push_str(&stderr);
                    }
                    
                    if !stdout.is_empty() {
                        error_msg.push_str("\n\nStandard output:\n");
                        error_msg.push_str(&stdout);
                    }
                    
                    Err(error_msg)
                }
            }
            Err(e) => {
                Err(format!("Failed to execute plugin '{}': {}", plugin_name, e))
            }
        }
    }
}

impl CommandProvider for UniCommandProvider {
    fn get_root_commands(&self) -> Vec<CommandItem> {
        let mut commands = Vec::new();
        
        // Add built-in commands
        commands.extend(self.get_builtin_commands());
        
        // Add plugin commands
        commands.extend(self.get_plugin_commands());
        
        // Sort commands alphabetically for consistent display
        commands.sort_by(|a, b| a.name.cmp(&b.name));
        
        commands
    }

    fn get_subcommands(&self, path: &[String]) -> Vec<CommandItem> {
        if path.is_empty() {
            return self.get_root_commands();
        }
        
        // For now, we only support one level of subcommands (plugin -> subcommand)
        if path.len() == 1 {
            let plugin_name = &path[0];
            
            // Check if this is a plugin with subcommands
            return self.get_plugin_subcommands(plugin_name);
        }
        
        // Deeper nesting not supported yet
        vec![]
    }

    fn get_help_text(&self, path: &[String]) -> String {
        if path.is_empty() {
            return "Select a command to see its description".to_string();
        }
        
        if path.len() == 1 {
            let command_name = &path[0];
            
            // Check built-in commands first
            if let Some(builtin) = self.get_builtin_commands().iter().find(|cmd| cmd.name == *command_name) {
                return builtin.description.clone();
            }
            
            // Check plugin commands
            if let Some(manifest) = self.find_manifest(command_name) {
                return format!("{} (v{})", manifest.description, manifest.version);
            }
        } else if path.len() == 2 {
            let plugin_name = &path[0];
            let subcommand_name = &path[1];
            
            if let Some(manifest) = self.find_manifest(plugin_name) {
                if let Some(subcmd) = manifest.commands.iter().find(|cmd| cmd.name == *subcommand_name) {
                    return subcmd.description.clone();
                }
            }
        }
        
        "No help available for this command".to_string()
    }

    fn execute_command(&self, path: &[String], args: &[String]) -> Result<String, String> {
        if path.is_empty() {
            return Err("No command specified".to_string());
        }

        let command_name = &path[0];

        // Check if this is a built-in command
        if self.is_builtin_command(command_name) {
            return self.execute_builtin_command(path, args);
        }

        // Check if this is a plugin command
        if let Some(manifest) = self.find_manifest(command_name) {
            return self.execute_plugin_command(manifest, path, args);
        }

        Err(format!("Command '{}' not found", command_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SubCmdMeta;

    #[test]
    fn test_builtin_commands_discovery() {
        let provider = UniCommandProvider::new(vec![]);
        let commands = provider.get_root_commands();
        
        // Should have all 7 built-in commands
        assert_eq!(commands.len(), 7);
        
        // Check that specific commands exist
        assert!(commands.iter().any(|cmd| cmd.name == "add"));
        assert!(commands.iter().any(|cmd| cmd.name == "list"));
        assert!(commands.iter().any(|cmd| cmd.name == "create"));
    }

    #[test]
    fn test_plugin_commands_discovery() {
        let manifest = Manifest {
            name: "test-plugin".to_string(),
            description: "A test plugin".to_string(),
            version: "1.0.0".to_string(),
            commands: vec![
                SubCmdMeta {
                    name: "run".to_string(),
                    description: "Run the plugin".to_string(),
                },
            ],
        };
        
        let provider = UniCommandProvider::new(vec![manifest]);
        let commands = provider.get_root_commands();
        
        // Should have built-ins + 1 plugin command
        assert_eq!(commands.len(), 8);
        
        // Check that plugin command exists
        assert!(commands.iter().any(|cmd| cmd.name == "test-plugin"));
        
        // Check that plugin has subcommands
        let plugin_cmd = commands.iter().find(|cmd| cmd.name == "test-plugin").unwrap();
        assert!(plugin_cmd.has_subcommands);
    }

    #[test]
    fn test_plugin_subcommands_discovery() {
        let manifest = Manifest {
            name: "test-plugin".to_string(),
            description: "A test plugin".to_string(),
            version: "1.0.0".to_string(),
            commands: vec![
                SubCmdMeta {
                    name: "run".to_string(),
                    description: "Run the plugin".to_string(),
                },
                SubCmdMeta {
                    name: "status".to_string(),
                    description: "Show plugin status".to_string(),
                },
            ],
        };
        
        let provider = UniCommandProvider::new(vec![manifest]);
        let subcommands = provider.get_subcommands(&["test-plugin".to_string()]);
        
        // Should have 2 subcommands
        assert_eq!(subcommands.len(), 2);
        assert!(subcommands.iter().any(|cmd| cmd.name == "run"));
        assert!(subcommands.iter().any(|cmd| cmd.name == "status"));
    }

    #[test]
    fn test_help_text_generation() {
        let manifest = Manifest {
            name: "test-plugin".to_string(),
            description: "A test plugin".to_string(),
            version: "1.0.0".to_string(),
            commands: vec![
                SubCmdMeta {
                    name: "run".to_string(),
                    description: "Run the plugin".to_string(),
                },
            ],
        };
        
        let provider = UniCommandProvider::new(vec![manifest]);
        
        // Test built-in command help
        let help = provider.get_help_text(&["add".to_string()]);
        assert_eq!(help, "Add a new plugin from a script file");
        
        // Test plugin command help
        let help = provider.get_help_text(&["test-plugin".to_string()]);
        assert_eq!(help, "A test plugin (v1.0.0)");
        
        // Test plugin subcommand help
        let help = provider.get_help_text(&["test-plugin".to_string(), "run".to_string()]);
        assert_eq!(help, "Run the plugin");
    }
}