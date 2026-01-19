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
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::Arc;

pub struct MakeValidator;

impl CommandValidator for MakeValidator {
    fn supports_command(&self, working_dir: &Path, command: &str) -> CommandSupport {
        let makefile_paths = ["Makefile", "makefile", "GNUmakefile"];

        for makefile_name in makefile_paths {
            let makefile = working_dir.join(makefile_name);
            if !makefile.exists() {
                continue;
            }

            let content = match fs::read_to_string(&makefile) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let mut targets: HashSet<&str> = HashSet::new();

            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('#') || trimmed.starts_with('\t') || trimmed.is_empty() {
                    continue;
                }

                if let Some(colon_pos) = trimmed.find(':') {
                    let target_part = &trimmed[..colon_pos];
                    if !target_part.contains('$') && !target_part.contains('%') {
                        for target in target_part.split_whitespace() {
                            if !target.starts_with('.') {
                                targets.insert(target);
                            }
                        }
                    }
                }
            }

            if targets.contains(command) {
                return CommandSupport::Supported;
            }

            return CommandSupport::NotSupported;
        }

        CommandSupport::Unknown
    }
}

/// Detect Makefile projects
/// Priority: 21 (last, as it's the most generic)
pub fn detect(dir: &Path) -> Vec<DetectedRunner> {
    let mut runners = Vec::new();
    let validator: Arc<dyn CommandValidator> = Arc::new(MakeValidator);

    // Use read_dir to get exact filename (case-sensitive on all platforms)
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name == "Makefile" || name == "makefile" {
                    runners.push(DetectedRunner::with_validator(
                        "make",
                        name,
                        Ecosystem::Generic,
                        21,
                        Arc::clone(&validator),
                    ));
                    break;
                }
            }
        }
    }

    runners
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_detect_makefile() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("Makefile")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "make");
        assert_eq!(runners[0].detected_file, "Makefile");
    }

    #[test]
    fn test_detect_makefile_lowercase() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("makefile")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "make");
        assert_eq!(runners[0].detected_file, "makefile");
    }

    #[test]
    fn test_no_makefile() {
        let dir = tempdir().unwrap();

        let runners = detect(dir.path());
        assert!(runners.is_empty());
    }
}
