use anyhow::Result;
use std::path::Path;
use crate::detectors::{Detector, Detection};

pub struct ZigDetector;

impl Detector for ZigDetector {
    fn detect(&self, path: &Path) -> Result<Option<Detection>> {
        // 20. Zig Build: build.zig -> zig build
        if path.join("build.zig").exists() {
             return Ok(Some(Detection {
                runner: "zig".to_string(),
                command: "zig build".to_string(),
                lockfile: None,
            }));
        }

        Ok(None)
    }
}
