// Command discovery and execution integration
use crate::Manifest;

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

    fn execute_command(&self, _path: &[String], _args: &[String]) -> Result<String, String> {
        // Command execution will be implemented in later tasks
        Ok("Command execution will be implemented in later tasks".to_string())
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