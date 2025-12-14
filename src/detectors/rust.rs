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

use super::{DetectedRunner, Ecosystem};
use std::path::Path;

/// Detect Rust package manager (Cargo)
/// Priority: 9
pub fn detect(dir: &Path) -> Vec<DetectedRunner> {
    let mut runners = Vec::new();

    let cargo_toml = dir.join("Cargo.toml");
    let cargo_lock = dir.join("Cargo.lock");

    if cargo_toml.exists() && cargo_lock.exists() {
        runners.push(DetectedRunner::new(
            "cargo",
            "Cargo.toml",
            Ecosystem::Rust,
            9,
        ));
    } else if cargo_toml.exists() {
        // Even without lock file, Cargo.toml is sufficient
        runners.push(DetectedRunner::new(
            "cargo",
            "Cargo.toml",
            Ecosystem::Rust,
            9,
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
