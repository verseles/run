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

use super::{CommandSupport, CommandValidator, DetectedRunner, Ecosystem};
use std::fs;
use std::path::Path;

pub struct RubyValidator;

impl CommandValidator for RubyValidator {
    fn supports_command(&self, working_dir: &Path, command: &str) -> CommandSupport {
        let rakefile = working_dir.join("Rakefile");
        if !rakefile.exists() {
            return CommandSupport::Unknown;
        }

        let content = match fs::read_to_string(&rakefile) {
            Ok(c) => c,
            Err(_) => return CommandSupport::Unknown,
        };

        let task_pattern = format!("task :{}", command);
        let task_pattern_string = format!("task \"{}\"", command);
        let task_pattern_single = format!("task '{}'", command);

        if content.contains(&task_pattern)
            || content.contains(&task_pattern_string)
            || content.contains(&task_pattern_single)
        {
            return CommandSupport::Supported;
        }

        CommandSupport::Unknown
    }
}

/// Detect Ruby package managers
/// Priority: Bundler (13) > Rake (14)
pub fn detect(dir: &Path) -> Vec<DetectedRunner> {
    let mut runners = Vec::new();

    // Check for Bundler (priority 13)
    let gemfile = dir.join("Gemfile");
    let gemfile_lock = dir.join("Gemfile.lock");
    if gemfile_lock.exists() && gemfile.exists() {
        runners.push(DetectedRunner::new(
            "bundler",
            "Gemfile.lock",
            Ecosystem::Ruby,
            13,
        ));
    } else if gemfile.exists() {
        runners.push(DetectedRunner::new(
            "bundler",
            "Gemfile",
            Ecosystem::Ruby,
            13,
        ));
    }

    // Check for Rake (priority 14)
    let rakefile = dir.join("Rakefile");
    if rakefile.exists() {
        runners.push(DetectedRunner::new("rake", "Rakefile", Ecosystem::Ruby, 14));
    }

    runners
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_detect_bundler_with_lock() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("Gemfile")).unwrap();
        File::create(dir.path().join("Gemfile.lock")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "bundler");
        assert_eq!(runners[0].detected_file, "Gemfile.lock");
    }

    #[test]
    fn test_detect_bundler_without_lock() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("Gemfile")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "bundler");
        assert_eq!(runners[0].detected_file, "Gemfile");
    }

    #[test]
    fn test_detect_rake() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("Rakefile")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "rake");
    }

    #[test]
    fn test_detect_bundler_and_rake() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("Gemfile")).unwrap();
        File::create(dir.path().join("Rakefile")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 2);
        assert!(runners.iter().any(|r| r.name == "bundler"));
        assert!(runners.iter().any(|r| r.name == "rake"));
    }
}
