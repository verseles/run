use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Result;
use directories::ProjectDirs;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Config {
    pub max_levels: Option<usize>,
    pub auto_update: Option<bool>,
    pub ignore_tools: Option<Vec<String>>,
    pub verbose: Option<bool>,
    pub quiet: Option<bool>,
}

pub fn load_config() -> Config {
    let mut config = Config::default();

    // 1. Global config: ~/.config/run/config.toml
    if let Some(proj_dirs) = ProjectDirs::from("", "", "run") {
        let global_config_path = proj_dirs.config_dir().join("config.toml");
        if let Ok(c) = load_from_file(&global_config_path) {
             config = merge(config, c);
        }
    }

    // 2. Local config: ./run.toml
    let local_config_path = Path::new("run.toml");
    if let Ok(c) = load_from_file(local_config_path) {
        config = merge(config, c);
    }

    config
}

fn load_from_file(path: &Path) -> Result<Config> {
    if path.exists() {
        let content = fs::read_to_string(path)?;
        let c: Config = toml::from_str(&content)?;
        Ok(c)
    } else {
        Ok(Config::default())
    }
}

fn merge(base: Config, other: Config) -> Config {
    Config {
        max_levels: other.max_levels.or(base.max_levels),
        auto_update: other.auto_update.or(base.auto_update),
        ignore_tools: other.ignore_tools.or(base.ignore_tools),
        verbose: other.verbose.or(base.verbose),
        quiet: other.quiet.or(base.quiet),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_load_config_local() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("run.toml");
        let mut file = File::create(&config_path).unwrap();
        writeln!(file, "max_levels = 10\nverbose = true").unwrap();

        // Change current directory to temp dir to test local config load
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let config = load_config();

        std::env::set_current_dir(original_dir).unwrap();

        assert_eq!(config.max_levels, Some(10));
        assert_eq!(config.verbose, Some(true));
    }

    #[test]
    fn test_merge_config() {
        let base = Config {
            max_levels: Some(3),
            verbose: Some(false),
            ..Default::default()
        };
        let other = Config {
            max_levels: Some(5),
            verbose: None,
            ..Default::default()
        };

        let merged = merge(base, other);
        assert_eq!(merged.max_levels, Some(5));
        assert_eq!(merged.verbose, Some(false));
    }
}
