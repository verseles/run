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

use super::{CommandSupport, CommandValidator, DetectedRunner, Ecosystem};
use std::fs;
use std::path::Path;
use std::sync::Arc;

pub struct PhpValidator;

impl CommandValidator for PhpValidator {
    fn supports_command(&self, working_dir: &Path, command: &str) -> CommandSupport {
        let composer_json = working_dir.join("composer.json");
        if !composer_json.exists() {
            return CommandSupport::Unknown;
        }

        let content = match fs::read_to_string(&composer_json) {
            Ok(c) => c,
            Err(_) => return CommandSupport::Unknown,
        };

        let json: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => return CommandSupport::Unknown,
        };

        if let Some(scripts) = json.get("scripts").and_then(|s| s.as_object()) {
            if scripts.contains_key(command) {
                return CommandSupport::Supported;
            }
            return CommandSupport::NotSupported;
        }

        CommandSupport::Unknown
    }
}

/// Detect PHP package manager (Composer)
/// Priority: 10
pub fn detect(dir: &Path) -> Vec<DetectedRunner> {
    let mut runners = Vec::new();

    let composer_json = dir.join("composer.json");
    let composer_lock = dir.join("composer.lock");
    let validator: Arc<dyn CommandValidator> = Arc::new(PhpValidator);

    if composer_lock.exists() && composer_json.exists() {
        runners.push(DetectedRunner::with_validator(
            "composer",
            "composer.lock",
            Ecosystem::Php,
            10,
            Arc::clone(&validator),
        ));
    } else if composer_json.exists() {
        runners.push(DetectedRunner::with_validator(
            "composer",
            "composer.json",
            Ecosystem::Php,
            10,
            Arc::clone(&validator),
        ));
    }

    runners
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_detect_composer_with_lock() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("composer.json")).unwrap();
        File::create(dir.path().join("composer.lock")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "composer");
        assert_eq!(runners[0].detected_file, "composer.lock");
    }

    #[test]
    fn test_detect_composer_without_lock() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("composer.json")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "composer");
        assert_eq!(runners[0].detected_file, "composer.json");
    }

    #[test]
    fn test_no_composer() {
        let dir = tempdir().unwrap();

        let runners = detect(dir.path());
        assert!(runners.is_empty());
    }
}
