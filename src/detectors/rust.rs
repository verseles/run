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

pub struct RustValidator;

impl CommandValidator for RustValidator {
    fn supports_command(&self, working_dir: &Path, command: &str) -> CommandSupport {
        static CARGO_BUILTIN: &[&str] = &[
            "build",
            "b",
            "check",
            "c",
            "clean",
            "doc",
            "d",
            "new",
            "init",
            "add",
            "remove",
            "run",
            "r",
            "test",
            "t",
            "bench",
            "update",
            "search",
            "publish",
            "install",
            "uninstall",
            "clippy",
            "fmt",
            "fix",
            "tree",
            "vendor",
            "verify-project",
            "version",
            "yank",
            "help",
            "generate-lockfile",
            "locate-project",
            "metadata",
            "pkgid",
            "fetch",
            "login",
            "logout",
            "owner",
            "package",
            "report",
            "rustc",
            "rustdoc",
        ];

        if CARGO_BUILTIN.contains(&command) {
            return CommandSupport::Supported;
        }

        // Check for aliases in .cargo/config.toml or .cargo/config
        if check_cargo_alias(working_dir, command) {
            return CommandSupport::Supported;
        }

        CommandSupport::NotSupported
    }
}

fn check_cargo_alias(dir: &Path, command: &str) -> bool {
    let check_file = |path: std::path::PathBuf| -> bool {
        if !path.exists() {
            return false;
        }
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(config) = content.parse::<toml::Value>() {
                if let Some(alias) = config.get("alias").and_then(|v| v.as_table()) {
                    return alias.contains_key(command);
                }
            }
        }
        false
    };

    // Check in .cargo/ directory inside the working directory
    let dot_cargo = dir.join(".cargo");
    check_file(dot_cargo.join("config.toml")) || check_file(dot_cargo.join("config"))
}

/// Detect Rust package manager (Cargo)
/// Priority: 9
pub fn detect(dir: &Path) -> Vec<DetectedRunner> {
    let mut runners = Vec::new();

    let cargo_toml = dir.join("Cargo.toml");
    let cargo_lock = dir.join("Cargo.lock");
    let validator: Arc<dyn CommandValidator> = Arc::new(RustValidator);

    if cargo_toml.exists() && cargo_lock.exists() {
        runners.push(DetectedRunner::with_validator(
            "cargo",
            "Cargo.toml",
            Ecosystem::Rust,
            9,
            Arc::clone(&validator),
        ));
    } else if cargo_toml.exists() {
        // Even without lock file, Cargo.toml is sufficient
        runners.push(DetectedRunner::with_validator(
            "cargo",
            "Cargo.toml",
            Ecosystem::Rust,
            9,
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
    fn test_detect_cargo_with_lock() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("Cargo.toml")).unwrap();
        File::create(dir.path().join("Cargo.lock")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "cargo");
    }

    #[test]
    fn test_detect_cargo_without_lock() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("Cargo.toml")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "cargo");
    }

    #[test]
    fn test_no_cargo_toml() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("Cargo.lock")).unwrap();

        let runners = detect(dir.path());
        assert!(runners.is_empty());
    }

    #[test]
    fn test_detected_runner_has_working_validator() {
        use super::CommandSupport;

        let dir = tempdir().unwrap();
        File::create(dir.path().join("Cargo.toml")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "cargo");

        // Verify the detected runner has a working validator (not UnknownValidator)
        assert_eq!(
            runners[0].supports_command("build", dir.path()),
            CommandSupport::Supported
        );
        assert_eq!(
            runners[0].supports_command("test", dir.path()),
            CommandSupport::Supported
        );
        assert_eq!(
            runners[0].supports_command("invalid_command", dir.path()),
            CommandSupport::NotSupported
        );
    }

    #[test]
    fn test_cargo_alias_support() {
        use std::io::Write;
        let dir = tempdir().unwrap();
        File::create(dir.path().join("Cargo.toml")).unwrap();

        let dot_cargo = dir.path().join(".cargo");
        fs::create_dir(&dot_cargo).unwrap();

        let mut config = File::create(dot_cargo.join("config.toml")).unwrap();
        writeln!(
            config,
            r#"
[alias]
my-alias = "run"
        "#
        )
        .unwrap();

        let runner = DetectedRunner::with_validator(
            "cargo",
            "Cargo.toml",
            Ecosystem::Rust,
            9,
            Arc::new(RustValidator),
        );

        assert_eq!(
            runner.supports_command("my-alias", dir.path()),
            CommandSupport::Supported
        );
        assert_eq!(
            runner.supports_command("unknown-alias", dir.path()),
            CommandSupport::NotSupported
        );
    }

    #[test]
    fn test_cargo_legacy_config_alias_support() {
        use std::io::Write;
        let dir = tempdir().unwrap();
        File::create(dir.path().join("Cargo.toml")).unwrap();

        let dot_cargo = dir.path().join(".cargo");
        fs::create_dir(&dot_cargo).unwrap();

        let mut config = File::create(dot_cargo.join("config")).unwrap();
        writeln!(
            config,
            r#"
[alias]
legacy-alias = "test"
        "#
        )
        .unwrap();

        let runner = DetectedRunner::with_validator(
            "cargo",
            "Cargo.toml",
            Ecosystem::Rust,
            9,
            Arc::new(RustValidator),
        );

        assert_eq!(
            runner.supports_command("legacy-alias", dir.path()),
            CommandSupport::Supported
        );
    }
}
