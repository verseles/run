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
use std::path::Path;
use std::sync::Arc;

pub struct RustValidator;

impl CommandValidator for RustValidator {
    fn supports_command(&self, _working_dir: &Path, command: &str) -> CommandSupport {
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

        CommandSupport::NotSupported
    }
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
}
