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
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration structure for the run CLI
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// Maximum levels to search above current directory
    pub max_levels: Option<u8>,
    /// Enable auto-update
    pub auto_update: Option<bool>,
    /// Tools to ignore during detection
    pub ignore_tools: Vec<String>,
    /// Enable verbose output
    pub verbose: Option<bool>,
    /// Enable quiet mode
    pub quiet: Option<bool>,
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
        }
    }

    /// Get max levels with default fallback
    pub fn get_max_levels(&self) -> u8 {
        self.max_levels.unwrap_or(3)
    }

    /// Get auto update setting with default fallback
    pub fn get_auto_update(&self) -> bool {
        self.auto_update.unwrap_or(true)
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
        };

        let override_config = Config {
            max_levels: Some(5),
            auto_update: None,
            ignore_tools: vec!["yarn".to_string()],
            verbose: Some(true),
            quiet: None,
        };

        let merged = base.merge(override_config);
        assert_eq!(merged.get_max_levels(), 5);
        assert!(merged.get_auto_update());
        assert_eq!(merged.ignore_tools, vec!["yarn".to_string()]);
        assert!(merged.get_verbose());
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
}
