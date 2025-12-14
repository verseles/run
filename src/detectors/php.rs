use anyhow::Result;
use std::path::Path;
use crate::detectors::{Detector, Detection};

pub struct PhpDetector;

impl Detector for PhpDetector {
    fn detect(&self, path: &Path) -> Result<Option<Detection>> {
        // 10. Composer: composer.lock + composer.json -> composer run
        if path.join("composer.lock").exists() && path.join("composer.json").exists() {
             return Ok(Some(Detection {
                runner: "composer".to_string(),
                command: "composer run".to_string(),
                lockfile: Some("composer.lock".to_string()),
            }));
        }

        // Fallback for just composer.json? Plan only mentions with lock.
        // But usually composer is used without lock too.
        // Sticking to plan: "composer.lock + composer.json -> composer run <comando>"

        Ok(None)
    }
}
