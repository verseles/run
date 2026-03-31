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

pub const DART_BUILTIN: &[&str] = &[
    "analyze", "compile", "create", "doc", "fix", "format", "info", "pub", "run", "test",
];

pub const FLUTTER_BUILTIN: &[&str] = &[
    "analyze",
    "assemble",
    "attach",
    "bash-completion",
    "build",
    "channel",
    "clean",
    "config",
    "create",
    "custom-devices",
    "devices",
    "doctor",
    "downgrade",
    "drive",
    "emulators",
    "gen-l10n",
    "install",
    "logs",
    "pub",
    "run",
    "screenshot",
    "symbolize",
    "test",
    "upgrade",
];

pub struct DartValidator {
    pub is_flutter: bool,
}

impl CommandValidator for DartValidator {
    fn supports_command(&self, _working_dir: &Path, command: &str) -> CommandSupport {
        if self.is_flutter {
            if FLUTTER_BUILTIN.contains(&command) {
                return CommandSupport::Supported;
            }
        } else if DART_BUILTIN.contains(&command) {
            return CommandSupport::Supported;
        }

        // Custom scripts or anything else might be supported via `dart run` or `flutter run`
        CommandSupport::Unknown
    }
}

fn check_is_flutter(pubspec_path: &Path) -> bool {
    let content = match fs::read_to_string(pubspec_path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    let yaml: serde_yaml::Value = match serde_yaml::from_str(&content) {
        Ok(v) => v,
        Err(_) => return false,
    };

    // Check if dependencies -> flutter exists
    if let Some(deps) = yaml.get("dependencies") {
        if deps.get("flutter").is_some() {
            return true;
        }
    }

    false
}

pub fn detect(dir: &Path) -> Vec<DetectedRunner> {
    let mut runners = Vec::new();
    let pubspec = dir.join("pubspec.yaml");

    if pubspec.exists() {
        let is_flutter = check_is_flutter(&pubspec);
        let validator: Arc<dyn CommandValidator> = Arc::new(DartValidator { is_flutter });

        if is_flutter {
            runners.push(DetectedRunner::with_validator(
                "flutter",
                "pubspec.yaml",
                Ecosystem::Dart,
                11,
                validator,
            ));
        } else {
            runners.push(DetectedRunner::with_validator(
                "dart",
                "pubspec.yaml",
                Ecosystem::Dart,
                11,
                validator,
            ));
        }
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
        let mut file = File::create(dir.path().join("pubspec.yaml")).unwrap();
        writeln!(
            file,
            r#"
name: my_dart_project
dependencies:
  path: ^1.8.0
"#
        )
        .unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "dart");
    }

    #[test]
    fn test_detect_flutter() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("pubspec.yaml")).unwrap();
        writeln!(
            file,
            r#"
name: my_flutter_project
dependencies:
  flutter:
    sdk: flutter
"#
        )
        .unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "flutter");
    }

    #[test]
    fn test_dart_validator() {
        let validator = DartValidator { is_flutter: false };
        let dir = Path::new("");
        assert_eq!(
            validator.supports_command(dir, "run"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir, "test"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir, "unknown_command"),
            CommandSupport::Unknown
        );
    }

    #[test]
    fn test_flutter_validator() {
        let validator = DartValidator { is_flutter: true };
        let dir = Path::new("");
        assert_eq!(
            validator.supports_command(dir, "run"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir, "build"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir, "unknown_command"),
            CommandSupport::Unknown
        );
    }
}
