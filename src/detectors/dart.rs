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
use serde::Deserialize;

#[derive(Deserialize)]
struct Pubspec {
    dependencies: Option<serde_yaml::Value>,
    dev_dependencies: Option<serde_yaml::Value>,
    environment: Option<serde_yaml::Value>,
}

pub struct DartValidator;

impl CommandValidator for DartValidator {
    fn supports_command(&self, _working_dir: &Path, command: &str) -> CommandSupport {
        static DART_BUILTIN: &[&str] = &[
            "analyze", "build", "compile", "create", "doc", "fix", "format", "info", "install",
            "pub", "run", "test",
        ];

        static FLUTTER_BUILTIN: &[&str] = &[
            "analyze", "build", "channel", "clean", "config", "create", "devices", "doctor",
            "downgrade", "drive", "emulators", "format", "gen-l10n", "install", "logs", "pub",
            "run", "screenshot", "symbolize", "test", "upgrade",
        ];

        if DART_BUILTIN.contains(&command) || FLUTTER_BUILTIN.contains(&command) {
            return CommandSupport::Supported;
        }

        CommandSupport::Unknown
    }
}

pub fn detect(dir: &Path) -> Vec<DetectedRunner> {
    let mut runners = Vec::new();
    let pubspec_path = dir.join("pubspec.yaml");

    if pubspec_path.exists() {
        let is_flutter = if let Ok(content) = fs::read_to_string(&pubspec_path) {
            if let Ok(pubspec) = serde_yaml::from_str::<Pubspec>(&content) {
                let has_flutter_dep = pubspec.dependencies
                    .as_ref()
                    .map(|d| d.get("flutter").is_some())
                    .unwrap_or(false);

                let has_flutter_dev_dep = pubspec.dev_dependencies
                    .as_ref()
                    .map(|d| d.get("flutter").is_some())
                    .unwrap_or(false);

                let has_flutter_env = pubspec.environment
                    .as_ref()
                    .map(|e| e.get("flutter").is_some())
                    .unwrap_or(false);

                has_flutter_dep || has_flutter_dev_dep || has_flutter_env
            } else {
                false
            }
        } else {
            false
        };

        let (name, priority) = if is_flutter {
            ("flutter", 19)
        } else {
            ("dart", 19)
        };

        runners.push(DetectedRunner::with_validator(
            name,
            "pubspec.yaml",
            Ecosystem::Dart,
            priority,
            Arc::new(DartValidator),
        ));
    }

    runners
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_detect_dart() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("pubspec.yaml")).unwrap();
        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "dart");
    }

    #[test]
    fn test_detect_flutter() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("pubspec.yaml")).unwrap();
        writeln!(file, "dependencies:\n  flutter:\n    sdk: flutter").unwrap();
        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "flutter");
    }
}
