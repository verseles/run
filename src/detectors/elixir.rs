use anyhow::Result;
use std::path::Path;
use crate::detectors::{Detector, Detection};

pub struct ElixirDetector;

impl Detector for ElixirDetector {
    fn detect(&self, path: &Path) -> Result<Option<Detection>> {
        // 18. Mix: mix.exs + mix.lock -> mix
        if path.join("mix.exs").exists() {
             let lockfile = if path.join("mix.lock").exists() {
                 Some("mix.lock".to_string())
             } else {
                 None
             };

             return Ok(Some(Detection {
                runner: "mix".to_string(),
                command: "mix".to_string(),
                lockfile,
            }));
        }

        Ok(None)
    }
}
