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

/// Detect PHP package manager (Composer)
/// Priority: 10
pub fn detect(dir: &Path) -> Vec<DetectedRunner> {
    let mut runners = Vec::new();

    let composer_json = dir.join("composer.json");
    let composer_lock = dir.join("composer.lock");

    if composer_lock.exists() && composer_json.exists() {
        runners.push(DetectedRunner::new(
            "composer",
            "composer.lock",
            Ecosystem::Php,
            10,
        ));
    } else if composer_json.exists() {
        runners.push(DetectedRunner::new(
            "composer",
            "composer.json",
            Ecosystem::Php,
            10,
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
    fn test_detect_composer_with_lock() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("composer.json")).unwrap();
        File::create(dir.path().join("composer.lock")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "composer");
        assert_eq!(runners[0].detected_file, "composer.lock");
    }

    #[test]
    fn test_detect_composer_without_lock() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("composer.json")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "composer");
        assert_eq!(runners[0].detected_file, "composer.json");
    }

    #[test]
    fn test_no_composer() {
        let dir = tempdir().unwrap();

        let runners = detect(dir.path());
        assert!(runners.is_empty());
    }
}
