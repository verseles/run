use anyhow::Result;
use std::path::Path;
use crate::detectors::{Detector, Detection};

pub struct RubyDetector;

impl Detector for RubyDetector {
    fn detect(&self, path: &Path) -> Result<Option<Detection>> {
        // 13. Bundler: Gemfile.lock + Gemfile -> bundle exec
        if path.join("Gemfile.lock").exists() && path.join("Gemfile").exists() {
             return Ok(Some(Detection {
                runner: "bundler".to_string(),
                command: "bundle exec".to_string(),
                lockfile: Some("Gemfile.lock".to_string()),
            }));
        }

        // 14. Rake: Rakefile -> rake
        if path.join("Rakefile").exists() {
             return Ok(Some(Detection {
                runner: "rake".to_string(),
                command: "rake".to_string(),
                lockfile: None,
            }));
        }

        Ok(None)
    }
}
