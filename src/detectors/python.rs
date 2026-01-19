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

pub struct PythonValidator;

impl CommandValidator for PythonValidator {
    fn supports_command(&self, working_dir: &Path, command: &str) -> CommandSupport {
        let pyproject = working_dir.join("pyproject.toml");
        if !pyproject.exists() {
            return CommandSupport::Unknown;
        }

        let content = match fs::read_to_string(&pyproject) {
            Ok(c) => c,
            Err(_) => return CommandSupport::Unknown,
        };

        let toml_value: toml::Value = match toml::from_str(&content) {
            Ok(v) => v,
            Err(_) => return CommandSupport::Unknown,
        };

        // Check [project.scripts] (PEP 621 - modern style, Poetry 2.0+ and UV)
        if let Some(scripts) = toml_value
            .get("project")
            .and_then(|p| p.get("scripts"))
            .and_then(|s| s.as_table())
        {
            if scripts.contains_key(command) {
                return CommandSupport::Supported;
            }
        }

        // Check [tool.poetry.scripts] (Poetry legacy style)
        if let Some(scripts) = toml_value
            .get("tool")
            .and_then(|t| t.get("poetry"))
            .and_then(|p| p.get("scripts"))
            .and_then(|s| s.as_table())
        {
            if scripts.contains_key(command) {
                return CommandSupport::Supported;
            }
        }

        // Python is extensible - uv run / poetry run can also execute
        // commands from the virtual environment (pytest, mypy, etc.)
        // So we return Unknown to allow fallback behavior
        CommandSupport::Unknown
    }
}

/// Detect Python package managers
/// Priority: UV (5) > Poetry (6) > Pipenv (7) > Pip (8)
pub fn detect(dir: &Path) -> Vec<DetectedRunner> {
    let mut runners = Vec::new();

    let has_pyproject = dir.join("pyproject.toml").exists();
    let validator: Arc<dyn CommandValidator> = Arc::new(PythonValidator);

    // Check for UV (priority 5)
    let uv_lock = dir.join("uv.lock");
    if uv_lock.exists() && has_pyproject {
        runners.push(DetectedRunner::with_validator(
            "uv",
            "uv.lock",
            Ecosystem::Python,
            5,
            Arc::clone(&validator),
        ));
    }

    // Check for Poetry (priority 6)
    let poetry_lock = dir.join("poetry.lock");
    if poetry_lock.exists() && has_pyproject {
        runners.push(DetectedRunner::with_validator(
            "poetry",
            "poetry.lock",
            Ecosystem::Python,
            6,
            Arc::clone(&validator),
        ));
    }

    // Check for Pipenv (priority 7)
    let pipfile = dir.join("Pipfile");
    let pipfile_lock = dir.join("Pipfile.lock");
    if pipfile_lock.exists() && pipfile.exists() {
        runners.push(DetectedRunner::with_validator(
            "pipenv",
            "Pipfile.lock",
            Ecosystem::Python,
            7,
            Arc::clone(&validator),
        ));
    }

    // Check for Pip (priority 8) - fallback
    let requirements = dir.join("requirements.txt");
    if requirements.exists() {
        runners.push(DetectedRunner::with_validator(
            "pip",
            "requirements.txt",
            Ecosystem::Python,
            8,
            Arc::clone(&validator),
        ));
    } else if has_pyproject && runners.is_empty() {
        // Only use pip with pyproject.toml if no other Python runner is detected
        runners.push(DetectedRunner::with_validator(
            "pip",
            "pyproject.toml",
            Ecosystem::Python,
            8,
            Arc::clone(&validator),
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
    fn test_detect_uv() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("pyproject.toml")).unwrap();
        File::create(dir.path().join("uv.lock")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "uv");
    }

    #[test]
    fn test_detect_poetry() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("pyproject.toml")).unwrap();
        File::create(dir.path().join("poetry.lock")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "poetry");
    }

    #[test]
    fn test_detect_pipenv() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("Pipfile")).unwrap();
        File::create(dir.path().join("Pipfile.lock")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "pipenv");
    }

    #[test]
    fn test_detect_pip_requirements() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("requirements.txt")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "pip");
        assert_eq!(runners[0].detected_file, "requirements.txt");
    }

    #[test]
    fn test_detect_pip_pyproject_fallback() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("pyproject.toml")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "pip");
        assert_eq!(runners[0].detected_file, "pyproject.toml");
    }

    #[test]
    fn test_no_pyproject_for_uv() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("uv.lock")).unwrap();

        let runners = detect(dir.path());
        assert!(runners.is_empty());
    }

    // Validator tests

    #[test]
    fn test_python_validator_pep621_scripts() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("pyproject.toml")).unwrap();
        writeln!(
            file,
            r#"
[project]
name = "example"
version = "1.0.0"

[project.scripts]
myapp = "example:main"
serve = "example.server:run"
"#
        )
        .unwrap();

        let validator = PythonValidator;
        assert_eq!(
            validator.supports_command(dir.path(), "myapp"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir.path(), "serve"),
            CommandSupport::Supported
        );
        // Unknown commands return Unknown (Python is extensible)
        assert_eq!(
            validator.supports_command(dir.path(), "nonexistent"),
            CommandSupport::Unknown
        );
    }

    #[test]
    fn test_python_validator_poetry_scripts() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("pyproject.toml")).unwrap();
        writeln!(
            file,
            r#"
[tool.poetry]
name = "example"
version = "1.0.0"

[tool.poetry.scripts]
cli = "example.cli:main"
worker = "example.worker:start"
"#
        )
        .unwrap();

        let validator = PythonValidator;
        assert_eq!(
            validator.supports_command(dir.path(), "cli"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir.path(), "worker"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir.path(), "unknown"),
            CommandSupport::Unknown
        );
    }

    #[test]
    fn test_python_validator_no_scripts_section() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("pyproject.toml")).unwrap();
        writeln!(
            file,
            r#"
[project]
name = "example"
version = "1.0.0"
"#
        )
        .unwrap();

        let validator = PythonValidator;
        // No scripts section, so return Unknown to allow fallback
        assert_eq!(
            validator.supports_command(dir.path(), "test"),
            CommandSupport::Unknown
        );
    }

    #[test]
    fn test_python_validator_no_pyproject() {
        let dir = tempdir().unwrap();

        let validator = PythonValidator;
        assert_eq!(
            validator.supports_command(dir.path(), "anything"),
            CommandSupport::Unknown
        );
    }

    #[test]
    fn test_detected_runner_has_working_validator() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("pyproject.toml")).unwrap();
        writeln!(
            file,
            r#"
[project]
name = "example"
version = "1.0.0"

[project.scripts]
myapp = "example:main"
"#
        )
        .unwrap();
        File::create(dir.path().join("uv.lock")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "uv");

        // Verify the detected runner has a working validator
        assert_eq!(
            runners[0].supports_command("myapp", dir.path()),
            CommandSupport::Supported
        );
        assert_eq!(
            runners[0].supports_command("nonexistent", dir.path()),
            CommandSupport::Unknown
        );
    }
}
