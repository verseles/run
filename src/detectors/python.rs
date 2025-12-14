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

/// Detect Python package managers
/// Priority: UV (5) > Poetry (6) > Pipenv (7) > Pip (8)
pub fn detect(dir: &Path) -> Vec<DetectedRunner> {
    let mut runners = Vec::new();

    let has_pyproject = dir.join("pyproject.toml").exists();

    // Check for UV (priority 5)
    let uv_lock = dir.join("uv.lock");
    if uv_lock.exists() && has_pyproject {
        runners.push(DetectedRunner::new("uv", "uv.lock", Ecosystem::Python, 5));
    }

    // Check for Poetry (priority 6)
    let poetry_lock = dir.join("poetry.lock");
    if poetry_lock.exists() && has_pyproject {
        runners.push(DetectedRunner::new(
            "poetry",
            "poetry.lock",
            Ecosystem::Python,
            6,
        ));
    }

    // Check for Pipenv (priority 7)
    let pipfile = dir.join("Pipfile");
    let pipfile_lock = dir.join("Pipfile.lock");
    if pipfile_lock.exists() && pipfile.exists() {
        runners.push(DetectedRunner::new(
            "pipenv",
            "Pipfile.lock",
            Ecosystem::Python,
            7,
        ));
    }

    // Check for Pip (priority 8) - fallback
    let requirements = dir.join("requirements.txt");
    if requirements.exists() {
        runners.push(DetectedRunner::new(
            "pip",
            "requirements.txt",
            Ecosystem::Python,
            8,
        ));
    } else if has_pyproject && runners.is_empty() {
        // Only use pip with pyproject.toml if no other Python runner is detected
        runners.push(DetectedRunner::new(
            "pip",
            "pyproject.toml",
            Ecosystem::Python,
            8,
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
}
