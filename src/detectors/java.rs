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

/// Detect Java/JVM build tools
/// Priority: Gradle (15) > Maven (16)
pub fn detect(dir: &Path) -> Vec<DetectedRunner> {
    let mut runners = Vec::new();

    // Check for Gradle (priority 15)
    let build_gradle = dir.join("build.gradle");
    let build_gradle_kts = dir.join("build.gradle.kts");
    if build_gradle.exists() {
        runners.push(DetectedRunner::new(
            "gradle",
            "build.gradle",
            Ecosystem::Java,
            15,
        ));
    } else if build_gradle_kts.exists() {
        runners.push(DetectedRunner::new(
            "gradle",
            "build.gradle.kts",
            Ecosystem::Java,
            15,
        ));
    }

    // Check for Maven (priority 16)
    let pom_xml = dir.join("pom.xml");
    if pom_xml.exists() {
        runners.push(DetectedRunner::new("maven", "pom.xml", Ecosystem::Java, 16));
    }

    runners
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_detect_gradle() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("build.gradle")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "gradle");
        assert_eq!(runners[0].detected_file, "build.gradle");
    }

    #[test]
    fn test_detect_gradle_kts() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("build.gradle.kts")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "gradle");
        assert_eq!(runners[0].detected_file, "build.gradle.kts");
    }

    #[test]
    fn test_detect_maven() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("pom.xml")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "maven");
    }

    #[test]
    fn test_detect_both_gradle_and_maven() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("build.gradle")).unwrap();
        File::create(dir.path().join("pom.xml")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 2);
        assert!(runners.iter().any(|r| r.name == "gradle"));
        assert!(runners.iter().any(|r| r.name == "maven"));
    }
}
