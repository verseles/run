use anyhow::Result;
use std::path::Path;
use crate::detectors::{Detector, Detection};

pub struct MakeDetector;

impl Detector for MakeDetector {
    fn detect(&self, path: &Path) -> Result<Option<Detection>> {
        // 21. Make: Makefile OR makefile -> make
        if path.join("Makefile").exists() || path.join("makefile").exists() {
             return Ok(Some(Detection {
                runner: "make".to_string(),
                command: "make".to_string(),
                lockfile: None,
            }));
        }

        Ok(None)
    }
}
