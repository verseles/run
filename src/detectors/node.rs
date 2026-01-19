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

pub struct NodeValidator;

impl CommandValidator for NodeValidator {
    fn supports_command(&self, working_dir: &Path, command: &str) -> CommandSupport {
        let package_json = working_dir.join("package.json");
        if !package_json.exists() {
            return CommandSupport::Unknown;
        }

        let content = match fs::read_to_string(&package_json) {
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

/// Detect Node.js package managers
/// Priority: Bun (1) > PNPM (2) > Yarn (3) > NPM (4)
pub fn detect(dir: &Path) -> Vec<DetectedRunner> {
    let mut runners = Vec::new();

    let has_package_json = dir.join("package.json").exists();

    // Check for Bun (priority 1)
    let bun_lockb = dir.join("bun.lockb");
    let bun_lock = dir.join("bun.lock");
    if bun_lockb.exists() && has_package_json {
        runners.push(DetectedRunner::new(
            "bun",
            "bun.lockb",
            Ecosystem::NodeJs,
            1,
        ));
    } else if bun_lock.exists() && has_package_json {
        runners.push(DetectedRunner::new("bun", "bun.lock", Ecosystem::NodeJs, 1));
    }

    // Check for PNPM (priority 2)
    let pnpm_lock = dir.join("pnpm-lock.yaml");
    if pnpm_lock.exists() && has_package_json {
        runners.push(DetectedRunner::new(
            "pnpm",
            "pnpm-lock.yaml",
            Ecosystem::NodeJs,
            2,
        ));
    }

    // Check for Yarn (priority 3)
    let yarn_lock = dir.join("yarn.lock");
    if yarn_lock.exists() && has_package_json {
        runners.push(DetectedRunner::new(
            "yarn",
            "yarn.lock",
            Ecosystem::NodeJs,
            3,
        ));
    }

    // Check for NPM (priority 4)
    let npm_lock = dir.join("package-lock.json");
    if npm_lock.exists() && has_package_json {
        runners.push(DetectedRunner::new(
            "npm",
            "package-lock.json",
            Ecosystem::NodeJs,
            4,
        ));
    } else if has_package_json && runners.is_empty() {
        // Fallback to npm if only package.json exists and no other Node runner detected
        runners.push(DetectedRunner::new(
            "npm",
            "package.json",
            Ecosystem::NodeJs,
            4,
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
    fn test_detect_bun_lockb() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("package.json")).unwrap();
        File::create(dir.path().join("bun.lockb")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "bun");
        assert_eq!(runners[0].detected_file, "bun.lockb");
    }

    #[test]
    fn test_detect_bun_lock() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("package.json")).unwrap();
        File::create(dir.path().join("bun.lock")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "bun");
        assert_eq!(runners[0].detected_file, "bun.lock");
    }

    #[test]
    fn test_detect_pnpm() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("package.json")).unwrap();
        File::create(dir.path().join("pnpm-lock.yaml")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "pnpm");
    }

    #[test]
    fn test_detect_yarn() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("package.json")).unwrap();
        File::create(dir.path().join("yarn.lock")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "yarn");
    }

    #[test]
    fn test_detect_npm() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("package.json")).unwrap();
        File::create(dir.path().join("package-lock.json")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "npm");
    }

    #[test]
    fn test_detect_npm_fallback() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("package.json")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "npm");
        assert_eq!(runners[0].detected_file, "package.json");
    }

    #[test]
    fn test_detect_multiple_lockfiles() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("package.json")).unwrap();
        File::create(dir.path().join("package-lock.json")).unwrap();
        File::create(dir.path().join("yarn.lock")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 2);
        // Should have both yarn and npm
        let names: Vec<&str> = runners.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"yarn"));
        assert!(names.contains(&"npm"));
    }

    #[test]
    fn test_no_package_json() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("yarn.lock")).unwrap();

        let runners = detect(dir.path());
        assert!(runners.is_empty());
    }
}
