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

/// Detect Go task runners and Go modules
/// Priority: Taskfile (11) > Go Modules (12)
pub fn detect(dir: &Path) -> Vec<DetectedRunner> {
    let mut runners = Vec::new();

    // Check for Taskfile (priority 11)
    let taskfile_yml = dir.join("Taskfile.yml");
    let taskfile_yaml = dir.join("Taskfile.yaml");
    if taskfile_yml.exists() {
        runners.push(DetectedRunner::new(
            "task",
            "Taskfile.yml",
            Ecosystem::Go,
            11,
        ));
    } else if taskfile_yaml.exists() {
        runners.push(DetectedRunner::new(
            "task",
            "Taskfile.yaml",
            Ecosystem::Go,
            11,
        ));
    }

    // Check for Go Modules (priority 12)
    // go.mod is sufficient for detection (go.sum is optional)
    let go_mod = dir.join("go.mod");
    if go_mod.exists() {
        runners.push(DetectedRunner::new("go", "go.mod", Ecosystem::Go, 12));
    }

    runners
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
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
}
