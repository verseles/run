use anyhow::Result;
use std::path::Path;
use crate::detectors::{Detector, Detection};

pub struct SwiftDetector;

impl Detector for SwiftDetector {
    fn detect(&self, path: &Path) -> Result<Option<Detection>> {
        // 19. Swift Package Manager: Package.swift -> swift run
        if path.join("Package.swift").exists() {
             return Ok(Some(Detection {
                runner: "swift".to_string(),
                command: "swift run".to_string(),
                lockfile: None, // Package.resolved?
            }));
        }

        Ok(None)
    }
}
