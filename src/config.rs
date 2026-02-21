// Copyright (C) 2025 Verseles
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, version 3 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Default interval between update checks in hours
const DEFAULT_CHECK_INTERVAL_HOURS: u64 = 2;

/// Configuration for the auto-update system
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct UpdateConfig {
    /// Enable auto-update (default: true)
    pub enabled: Option<bool>,
    /// Hours between update checks (default: 2)
    pub check_interval_hours: Option<u64>,
}

impl UpdateConfig {
    /// Get whether updates are enabled (default: true)
    pub fn get_enabled(&self) -> bool {
        self.enabled.unwrap_or(true)
    }

    /// Get the check interval in hours (default: 2)
    pub fn get_check_interval_hours(&self) -> u64 {
        self.check_interval_hours
            .unwrap_or(DEFAULT_CHECK_INTERVAL_HOURS)
    }

    /// Merge two UpdateConfig, with other taking precedence
    pub fn merge(self, other: UpdateConfig) -> Self {
        UpdateConfig {
            enabled: other.enabled.or(self.enabled),
            check_interval_hours: other.check_interval_hours.or(self.check_interval_hours),
        }
    }
}

/// Configuration structure for the run CLI
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// Maximum levels to search above current directory
    pub max_levels: Option<u8>,
    /// Enable auto-update (legacy, use [update] section instead)
    pub auto_update: Option<bool>,
    /// Tools to ignore during detection
    pub ignore_tools: Vec<String>,
    /// Enable verbose output
    pub verbose: Option<bool>,
    /// Enable quiet mode
    pub quiet: Option<bool>,
    /// Update configuration section
    pub update: Option<UpdateConfig>,
    /// Custom commands overrides
    pub commands: Option<HashMap<String, String>>,
}

impl Config {
    /// Load configuration from default locations with precedence:
    /// 1. Defaults (hardcoded)
    /// 2. Global config (~/.config/run/config.toml)
    /// 3. Local config (./run.toml)
    pub fn load() -> Self {
        let mut config = Config::default();

        // Load global config
        if let Some(global_path) = Self::global_config_path() {
            if let Ok(global_config) = Self::load_from_file(&global_path) {
                config = config.merge(global_config);
            }
        }

        // Load local config
        let local_path = PathBuf::from("run.toml");
        if let Ok(local_config) = Self::load_from_file(&local_path) {
            config = config.merge(local_config);
        }

        config
    }

    /// Get the path to the global configuration file
    pub fn global_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("run").join("config.toml"))
    }

    /// Get the path to the update info file
    pub fn update_info_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("run").join("update.json"))
    }

    /// Get the path to the last update check timestamp file
    pub fn last_update_check_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("run").join("last_update_check"))
    }

    /// Load configuration from a specific file
    pub fn load_from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Merge two configs, with other taking precedence
    pub fn merge(self, other: Config) -> Self {
        Config {
            max_levels: other.max_levels.or(self.max_levels),
            auto_update: other.auto_update.or(self.auto_update),
            ignore_tools: if other.ignore_tools.is_empty() {
                self.ignore_tools
            } else {
                other.ignore_tools
            },
            verbose: other.verbose.or(self.verbose),
            quiet: other.quiet.or(self.quiet),
            update: match (self.update, other.update) {
                (Some(base), Some(over)) => Some(base.merge(over)),
                (None, Some(over)) => Some(over),
                (Some(base), None) => Some(base),
                (None, None) => None,
            },
            commands: match (self.commands, other.commands) {
                (Some(mut base), Some(over)) => {
                    base.extend(over);
                    Some(base)
                }
                (None, Some(over)) => Some(over),
                (Some(base), None) => Some(base),
                (None, None) => None,
            },
        }
    }

    /// Get max levels with default fallback
    pub fn get_max_levels(&self) -> u8 {
        self.max_levels.unwrap_or(3)
    }

    /// Get auto update setting with default fallback
    /// This checks both the legacy `auto_update` field and the new `[update]` section
    pub fn get_auto_update(&self) -> bool {
        // New [update] section takes precedence over legacy field
        if let Some(ref update) = self.update {
            return update.get_enabled();
        }
        self.auto_update.unwrap_or(true)
    }

    /// Get the update configuration, creating a default if not set
    pub fn get_update_config(&self) -> UpdateConfig {
        self.update.clone().unwrap_or_default()
    }

    /// Get verbose setting with default fallback
    pub fn get_verbose(&self) -> bool {
        self.verbose.unwrap_or(false)
    }

    /// Get quiet setting with default fallback
    pub fn get_quiet(&self) -> bool {
        self.quiet.unwrap_or(false)
    }

    /// Ensure config directory exists
    pub fn ensure_config_dir() -> std::io::Result<PathBuf> {
        if let Some(config_dir) = dirs::config_dir() {
            let run_dir = config_dir.join("run");
            fs::create_dir_all(&run_dir)?;
            Ok(run_dir)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine config directory",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.get_max_levels(), 3);
        assert!(config.get_auto_update());
        assert!(!config.get_verbose());
        assert!(!config.get_quiet());
    }

    #[test]
    fn test_merge_config() {
        let base = Config {
            max_levels: Some(3),
            auto_update: Some(true),
            ignore_tools: vec!["npm".to_string()],
            verbose: None,
            quiet: None,
            update: None,
            commands: None,
        };

        let override_config = Config {
            max_levels: Some(5),
            auto_update: None,
            ignore_tools: vec!["yarn".to_string()],
            verbose: Some(true),
            quiet: None,
            update: None,
            commands: None,
        };

        let merged = base.merge(override_config);
        assert_eq!(merged.get_max_levels(), 5);
        assert!(merged.get_auto_update());
        assert_eq!(merged.ignore_tools, vec!["yarn".to_string()]);
        assert!(merged.get_verbose());
    }

    #[test]
    fn test_merge_commands() {
        let mut base_cmds = HashMap::new();
        base_cmds.insert("base".to_string(), "echo base".to_string());
        base_cmds.insert("both".to_string(), "echo base_both".to_string());

        let base = Config {
            commands: Some(base_cmds),
            ..Default::default()
        };

        let mut override_cmds = HashMap::new();
        override_cmds.insert("over".to_string(), "echo over".to_string());
        override_cmds.insert("both".to_string(), "echo over_both".to_string());

        let override_config = Config {
            commands: Some(override_cmds),
            ..Default::default()
        };

        let merged = base.merge(override_config);
        let cmds = merged.commands.unwrap();

        assert_eq!(cmds.get("base").unwrap(), "echo base");
        assert_eq!(cmds.get("over").unwrap(), "echo over");
        assert_eq!(cmds.get("both").unwrap(), "echo over_both");
    }

    #[test]
    fn test_load_from_file() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");

        fs::write(
            &config_path,
            r#"
max_levels = 5
auto_update = false
ignore_tools = ["npm", "yarn"]
verbose = true
"#,
        )
        .unwrap();

        let config = Config::load_from_file(&config_path).unwrap();
        assert_eq!(config.get_max_levels(), 5);
        assert!(!config.get_auto_update());
        assert_eq!(config.ignore_tools, vec!["npm", "yarn"]);
        assert!(config.get_verbose());
    }

    #[test]
    fn test_invalid_toml() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "invalid toml {{{").unwrap();

        let result = Config::load_from_file(&config_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_config_defaults() {
        let update_config = UpdateConfig::default();
        assert!(update_config.get_enabled());
        assert_eq!(update_config.get_check_interval_hours(), 2);
    }

    #[test]
    fn test_update_config_custom_values() {
        let update_config = UpdateConfig {
            enabled: Some(false),
            check_interval_hours: Some(24),
        };
        assert!(!update_config.get_enabled());
        assert_eq!(update_config.get_check_interval_hours(), 24);
    }

    #[test]
    fn test_update_config_merge() {
        let base = UpdateConfig {
            enabled: Some(true),
            check_interval_hours: Some(2),
        };
        let over = UpdateConfig {
            enabled: None,
            check_interval_hours: Some(4),
        };
        let merged = base.merge(over);
        assert!(merged.get_enabled());
        assert_eq!(merged.get_check_interval_hours(), 4);
    }

    #[test]
    fn test_load_config_with_update_section() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");

        fs::write(
            &config_path,
            r#"
max_levels = 3

[update]
enabled = true
check_interval_hours = 4
"#,
        )
        .unwrap();

        let config = Config::load_from_file(&config_path).unwrap();
        assert!(config.get_auto_update());
        assert_eq!(config.get_update_config().get_check_interval_hours(), 4);
    }

    #[test]
    fn test_update_section_overrides_legacy_auto_update() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");

        fs::write(
            &config_path,
            r#"
auto_update = true

[update]
enabled = false
"#,
        )
        .unwrap();

        let config = Config::load_from_file(&config_path).unwrap();
        // [update].enabled should override legacy auto_update
        assert!(!config.get_auto_update());
    }
}
