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

/// Detect monorepo orchestration tools (Nx, Turborepo, Lerna)
/// Priority: 0 (highest - these tools orchestrate other package managers)
pub fn detect(dir: &Path) -> Vec<DetectedRunner> {
    let mut runners = Vec::new();

    // Check for Nx (priority 0)
    let nx_json = dir.join("nx.json");
    if nx_json.exists() {
        runners.push(DetectedRunner::new("nx", "nx.json", Ecosystem::NodeJs, 0));
    }

    // Check for Turborepo (priority 0)
    let turbo_json = dir.join("turbo.json");
    if turbo_json.exists() {
        runners.push(DetectedRunner::new(
            "turbo",
            "turbo.json",
            Ecosystem::NodeJs,
            0,
        ));
    }

    // Check for Lerna (priority 0)
    let lerna_json = dir.join("lerna.json");
    if lerna_json.exists() {
        runners.push(DetectedRunner::new(
            "lerna",
            "lerna.json",
            Ecosystem::NodeJs,
            0,
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
    fn test_detect_nx() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("nx.json")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "nx");
        assert_eq!(runners[0].detected_file, "nx.json");
        assert_eq!(runners[0].priority, 0);
    }

    #[test]
    fn test_detect_turbo() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("turbo.json")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "turbo");
        assert_eq!(runners[0].detected_file, "turbo.json");
        assert_eq!(runners[0].priority, 0);
    }

    #[test]
    fn test_detect_lerna() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("lerna.json")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "lerna");
        assert_eq!(runners[0].detected_file, "lerna.json");
        assert_eq!(runners[0].priority, 0);
    }

    #[test]
    fn test_detect_multiple_monorepo_tools() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("nx.json")).unwrap();
        File::create(dir.path().join("turbo.json")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 2);
        let names: Vec<&str> = runners.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"nx"));
        assert!(names.contains(&"turbo"));
    }

    #[test]
    fn test_detect_no_monorepo() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("package.json")).unwrap();

        let runners = detect(dir.path());
        assert!(runners.is_empty());
    }

    #[test]
    fn test_priority_is_highest() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("turbo.json")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners[0].priority, 0);
        // Priority 0 is higher than Bun (1), PNPM (2), etc.
    }
}
