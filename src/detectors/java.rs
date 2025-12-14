use anyhow::Result;
use std::path::Path;
use crate::detectors::{Detector, Detection};

pub struct JavaDetector;

impl Detector for JavaDetector {
    fn detect(&self, path: &Path) -> Result<Option<Detection>> {
        // 15. Gradle: build.gradle OR build.gradle.kts -> gradle
        if path.join("build.gradle").exists() || path.join("build.gradle.kts").exists() {
             let lockfile = if path.join("gradle.lockfile").exists() {
                 Some("gradle.lockfile".to_string())
             } else {
                 None
             };

             // Check for gradle wrapper
             let command = if path.join("gradlew").exists() {
                 if cfg!(windows) {
                     ".\\gradlew"
                 } else {
                     "./gradlew"
                 }
             } else {
                 "gradle"
             };

             return Ok(Some(Detection {
                runner: "gradle".to_string(),
                command: command.to_string(),
                lockfile,
            }));
        }

        // 16. Maven: pom.xml -> mvn
        if path.join("pom.xml").exists() {
             // Check for maven wrapper
             let command = if path.join("mvnw").exists() {
                 if cfg!(windows) {
                     ".\\mvnw"
                 } else {
                     "./mvnw"
                 }
             } else {
                 "mvn"
             };

             return Ok(Some(Detection {
                runner: "maven".to_string(),
                command: command.to_string(),
                lockfile: None,
            }));
        }

        Ok(None)
    }
}
