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

/// Validator for Taskfile (go-task)
pub struct TaskfileValidator;

impl CommandValidator for TaskfileValidator {
    fn supports_command(&self, working_dir: &Path, command: &str) -> CommandSupport {
        // Try both extensions
        let taskfile_yml = working_dir.join("Taskfile.yml");
        let taskfile_yaml = working_dir.join("Taskfile.yaml");

        let path = if taskfile_yml.exists() {
            taskfile_yml
        } else if taskfile_yaml.exists() {
            taskfile_yaml
        } else {
            return CommandSupport::Unknown;
        };

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return CommandSupport::Unknown,
        };

        let yaml: serde_yaml::Value = match serde_yaml::from_str(&content) {
            Ok(v) => v,
            Err(_) => return CommandSupport::Unknown,
        };

        if let Some(tasks) = yaml.get("tasks").and_then(|t| t.as_mapping()) {
            // Task names can include colons (docker:build)
            for key in tasks.keys() {
                if let Some(task_name) = key.as_str() {
                    if task_name == command {
                        return CommandSupport::Supported;
                    }
                }
            }
            return CommandSupport::NotSupported;
        }

        CommandSupport::Unknown
    }
}

/// Validator for Go modules (built-in go commands)
pub struct GoValidator;

impl CommandValidator for GoValidator {
    fn supports_command(&self, _working_dir: &Path, command: &str) -> CommandSupport {
        // Go has built-in commands
        const BUILTINS: &[&str] = &[
            "build", "clean", "doc", "env", "fix", "fmt", "generate", "get", "install", "list",
            "mod", "work", "run", "test", "tool", "version", "vet", "help",
        ];

        if BUILTINS.contains(&command) {
            return CommandSupport::Supported;
        }

        // Go is extensible (go run, go generate, etc.), so return Unknown
        CommandSupport::Unknown
    }
}

/// Detect Go task runners and Go modules
/// Priority: Taskfile (11) > Go Modules (12)
pub fn detect(dir: &Path) -> Vec<DetectedRunner> {
    let mut runners = Vec::new();

    // Check for Taskfile (priority 11)
    let taskfile_yml = dir.join("Taskfile.yml");
    let taskfile_yaml = dir.join("Taskfile.yaml");
    let taskfile_validator: Arc<dyn CommandValidator> = Arc::new(TaskfileValidator);

    if taskfile_yml.exists() {
        runners.push(DetectedRunner::with_validator(
            "task",
            "Taskfile.yml",
            Ecosystem::Go,
            11,
            Arc::clone(&taskfile_validator),
        ));
    } else if taskfile_yaml.exists() {
        runners.push(DetectedRunner::with_validator(
            "task",
            "Taskfile.yaml",
            Ecosystem::Go,
            11,
            Arc::clone(&taskfile_validator),
        ));
    }

    // Check for Go Modules (priority 12)
    // go.mod is sufficient for detection (go.sum is optional)
    let go_mod = dir.join("go.mod");
    if go_mod.exists() {
        let go_validator: Arc<dyn CommandValidator> = Arc::new(GoValidator);
        runners.push(DetectedRunner::with_validator(
            "go",
            "go.mod",
            Ecosystem::Go,
            12,
            go_validator,
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
    fn test_detect_taskfile_yml() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("Taskfile.yml")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "task");
        assert_eq!(runners[0].detected_file, "Taskfile.yml");
    }

    #[test]
    fn test_detect_taskfile_yaml() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("Taskfile.yaml")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "task");
        assert_eq!(runners[0].detected_file, "Taskfile.yaml");
    }

    #[test]
    fn test_detect_go_mod_with_sum() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("go.mod")).unwrap();
        File::create(dir.path().join("go.sum")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "go");
    }

    #[test]
    fn test_detect_go_mod_without_sum() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("go.mod")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "go");
    }

    #[test]
    fn test_detect_both_taskfile_and_go() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("Taskfile.yml")).unwrap();
        File::create(dir.path().join("go.mod")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 2);
        assert!(runners.iter().any(|r| r.name == "task"));
        assert!(runners.iter().any(|r| r.name == "go"));
    }

    // Taskfile validator tests

    #[test]
    fn test_taskfile_validator_supported() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("Taskfile.yml")).unwrap();
        writeln!(
            file,
            r#"
version: '3'

tasks:
  build:
    cmds:
      - go build .
  test:
    cmds:
      - go test ./...
  docker:build:
    cmds:
      - docker build .
"#
        )
        .unwrap();

        let validator = TaskfileValidator;
        assert_eq!(
            validator.supports_command(dir.path(), "build"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir.path(), "test"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir.path(), "docker:build"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir.path(), "nonexistent"),
            CommandSupport::NotSupported
        );
    }

    #[test]
    fn test_taskfile_validator_no_file() {
        let dir = tempdir().unwrap();

        let validator = TaskfileValidator;
        assert_eq!(
            validator.supports_command(dir.path(), "anything"),
            CommandSupport::Unknown
        );
    }

    #[test]
    fn test_taskfile_validator_yaml_extension() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("Taskfile.yaml")).unwrap();
        writeln!(
            file,
            r#"
version: '3'

tasks:
  lint:
    cmds:
      - golangci-lint run
"#
        )
        .unwrap();

        let validator = TaskfileValidator;
        assert_eq!(
            validator.supports_command(dir.path(), "lint"),
            CommandSupport::Supported
        );
    }

    // Go validator tests

    #[test]
    fn test_go_validator_builtins() {
        let dir = tempdir().unwrap();

        let validator = GoValidator;
        assert_eq!(
            validator.supports_command(dir.path(), "build"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir.path(), "test"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir.path(), "run"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir.path(), "mod"),
            CommandSupport::Supported
        );
        // Unknown command returns Unknown (Go is extensible)
        assert_eq!(
            validator.supports_command(dir.path(), "custom"),
            CommandSupport::Unknown
        );
    }

    #[test]
    fn test_detected_runner_has_working_validator() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("Taskfile.yml")).unwrap();
        writeln!(
            file,
            r#"
version: '3'

tasks:
  build:
    cmds:
      - go build .
"#
        )
        .unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "task");

        // Verify the detected runner has a working validator
        assert_eq!(
            runners[0].supports_command("build", dir.path()),
            CommandSupport::Supported
        );
        assert_eq!(
            runners[0].supports_command("nonexistent", dir.path()),
            CommandSupport::NotSupported
        );
    }
}
