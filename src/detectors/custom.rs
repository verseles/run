use crate::detectors::{CommandSupport, CommandValidator, DetectedRunner, Ecosystem};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

#[derive(Deserialize)]
struct RunConfig {
    commands: Option<HashMap<String, String>>,
}

pub struct CustomValidator {
    commands: HashMap<String, String>,
}

impl CommandValidator for CustomValidator {
    fn supports_command(&self, _working_dir: &Path, command: &str) -> CommandSupport {
        if self.commands.contains_key(command) {
            CommandSupport::Supported
        } else {
            CommandSupport::NotSupported
        }
    }
}

pub fn detect(dir: &Path) -> Vec<DetectedRunner> {
    let config_path = dir.join("run.toml");
    if !config_path.exists() {
        return vec![];
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let config: RunConfig = match toml::from_str(&content) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    if let Some(commands) = config.commands {
        if commands.is_empty() {
            return vec![];
        }

        // Filter out empty commands
        let valid_commands: HashMap<String, String> = commands
            .into_iter()
            .filter(|(_, cmd)| !cmd.trim().is_empty())
            .collect();

        if valid_commands.is_empty() {
            return vec![];
        }

        // Return a single runner for the custom commands
        // Priority 0 means it overrides everything else
        vec![DetectedRunner::with_custom_commands(
            "custom",
            "run.toml",
            Ecosystem::Custom,
            0,
            Arc::new(CustomValidator {
                commands: valid_commands.clone(),
            }),
            valid_commands,
        )]
    } else {
        vec![]
    }
}
