use crate::detectors::{Detection, Detector};
use anyhow::Result;
use std::path::Path;

pub struct RustDetector;

impl Detector for RustDetector {
    fn detect(&self, path: &Path) -> Result<Option<Detection>> {
        // Cargo.toml + Cargo.lock -> cargo
        if path.join("Cargo.toml").exists() {
            let lockfile = if path.join("Cargo.lock").exists() {
                Some("Cargo.lock".to_string())
            } else {
                None
            };
            return Ok(Some(Detection {
                runner: "cargo".to_string(),
                command: "cargo".to_string(),
                lockfile,
            }));
        }
        Ok(None)
    }
}
