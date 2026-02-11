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

/// Get the package manager specified by Corepack in package.json
/// Returns the package manager name (e.g., "pnpm", "yarn", "npm") if found
/// Format: "packageManager": "pnpm@9.0.0" or "packageManager": "yarn@4.0.0+sha256.abc123"
pub fn get_corepack_manager(dir: &Path) -> Option<String> {
    let package_json = dir.join("package.json");
    let content = fs::read_to_string(&package_json).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;

    let package_manager = json.get("packageManager")?.as_str()?;

    // Parse format: "pnpm@9.0.0" or "yarn@4.0.0+sha256.abc123"
    // Extract just the package manager name (before @)
    let name = package_manager.split('@').next()?;

    if name.is_empty() {
        return None;
    }

    Some(name.to_string())
}

/// Detect Node.js package managers
/// Priority: Bun (1) > PNPM (2) > Yarn (3) > NPM (4)
pub fn detect(dir: &Path) -> Vec<DetectedRunner> {
    let mut runners = Vec::new();

    let has_package_json = dir.join("package.json").exists();
    let validator: Arc<dyn CommandValidator> = Arc::new(NodeValidator);

    // Check for Corepack (packageManager field)
    if has_package_json {
        if let Some(manager) = get_corepack_manager(dir) {
            let (priority, name) = match manager.as_str() {
                "bun" => (1, "bun"),
                "pnpm" => (2, "pnpm"),
                "yarn" => (3, "yarn"),
                "npm" => (4, "npm"),
                _ => (4, manager.as_str()),
            };

            runners.push(DetectedRunner::with_validator(
                name,
                "package.json",
                Ecosystem::NodeJs,
                priority,
                Arc::clone(&validator),
            ));
        }
    }

    // Check for Bun (priority 1)
    let bun_lockb = dir.join("bun.lockb");
    let bun_lock = dir.join("bun.lock");
    if bun_lockb.exists() && has_package_json {
        runners.push(DetectedRunner::with_validator(
            "bun",
            "bun.lockb",
            Ecosystem::NodeJs,
            1,
            Arc::clone(&validator),
        ));
    } else if bun_lock.exists() && has_package_json {
        runners.push(DetectedRunner::with_validator(
            "bun",
            "bun.lock",
            Ecosystem::NodeJs,
            1,
            Arc::clone(&validator),
        ));
    }

    // Check for PNPM (priority 2)
    let pnpm_lock = dir.join("pnpm-lock.yaml");
    if pnpm_lock.exists() && has_package_json {
        runners.push(DetectedRunner::with_validator(
            "pnpm",
            "pnpm-lock.yaml",
            Ecosystem::NodeJs,
            2,
            Arc::clone(&validator),
        ));
    }

    // Check for Yarn (priority 3)
    let yarn_lock = dir.join("yarn.lock");
    if yarn_lock.exists() && has_package_json {
        runners.push(DetectedRunner::with_validator(
            "yarn",
            "yarn.lock",
            Ecosystem::NodeJs,
            3,
            Arc::clone(&validator),
        ));
    }

    // Check for NPM (priority 4)
    let npm_lock = dir.join("package-lock.json");
    if npm_lock.exists() && has_package_json {
        runners.push(DetectedRunner::with_validator(
            "npm",
            "package-lock.json",
            Ecosystem::NodeJs,
            4,
            Arc::clone(&validator),
        ));
    } else if has_package_json && runners.is_empty() {
        // Fallback to npm if only package.json exists and no other Node runner detected
        runners.push(DetectedRunner::with_validator(
            "npm",
            "package.json",
            Ecosystem::NodeJs,
            4,
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

    #[test]
    fn test_detected_runner_has_working_validator() {
        use super::CommandSupport;
        use std::io::Write;

        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("package.json")).unwrap();
        writeln!(file, r#"{{"scripts": {{"test": "jest", "build": "tsc"}}}}"#).unwrap();
        File::create(dir.path().join("package-lock.json")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "npm");

        // Verify the detected runner has a working validator (not UnknownValidator)
        assert_eq!(
            runners[0].supports_command("test", dir.path()),
            CommandSupport::Supported
        );
        assert_eq!(
            runners[0].supports_command("build", dir.path()),
            CommandSupport::Supported
        );
        assert_eq!(
            runners[0].supports_command("nonexistent", dir.path()),
            CommandSupport::NotSupported
        );
    }

    #[test]
    fn test_corepack_pnpm() {
        use std::io::Write;

        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("package.json")).unwrap();
        writeln!(file, r#"{{"packageManager": "pnpm@9.0.0"}}"#).unwrap();

        let result = get_corepack_manager(dir.path());
        assert_eq!(result, Some("pnpm".to_string()));
    }

    #[test]
    fn test_corepack_yarn_with_hash() {
        use std::io::Write;

        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("package.json")).unwrap();
        writeln!(
            file,
            r#"{{"packageManager": "yarn@4.0.0+sha256.abc123def456"}}"#
        )
        .unwrap();

        let result = get_corepack_manager(dir.path());
        assert_eq!(result, Some("yarn".to_string()));
    }

    #[test]
    fn test_corepack_npm() {
        use std::io::Write;

        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("package.json")).unwrap();
        writeln!(file, r#"{{"packageManager": "npm@10.2.0"}}"#).unwrap();

        let result = get_corepack_manager(dir.path());
        assert_eq!(result, Some("npm".to_string()));
    }

    #[test]
    fn test_corepack_missing() {
        use std::io::Write;

        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("package.json")).unwrap();
        writeln!(file, r#"{{"scripts": {{"test": "jest"}}}}"#).unwrap();

        let result = get_corepack_manager(dir.path());
        assert_eq!(result, None);
    }

    #[test]
    fn test_corepack_no_package_json() {
        let dir = tempdir().unwrap();

        let result = get_corepack_manager(dir.path());
        assert_eq!(result, None);
    }

    #[test]
    fn test_detect_respects_package_manager() {
        use std::io::Write;
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("package.json")).unwrap();
        writeln!(file, r#"{{"packageManager": "pnpm@9.0.0"}}"#).unwrap();

        // No lockfiles

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1, "Expected 1 runner, found {:?}", runners);
        assert_eq!(runners[0].name, "pnpm");
    }

    #[test]
    fn test_detect_conflicting_package_manager() {
        use std::io::Write;
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("package.json")).unwrap();
        writeln!(file, r#"{{"packageManager": "pnpm@9.0.0"}}"#).unwrap();

        // yarn.lock exists
        File::create(dir.path().join("yarn.lock")).unwrap();

        let runners = detect(dir.path());
        // Should detect BOTH yarn (file) and pnpm (packageManager)
        let names: Vec<&str> = runners.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"yarn"), "Should contain yarn");
        assert!(names.contains(&"pnpm"), "Should contain pnpm");
    }
}
