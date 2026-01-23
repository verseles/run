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

/// Validator for Just command runner
pub struct JustValidator;

impl CommandValidator for JustValidator {
    fn supports_command(&self, working_dir: &Path, command: &str) -> CommandSupport {
        // Try multiple justfile naming conventions
        let justfile_names = ["justfile", "Justfile", ".justfile"];

        let mut justfile_path = None;
        for name in &justfile_names {
            let path = working_dir.join(name);
            if path.exists() {
                justfile_path = Some(path);
                break;
            }
        }

        let path = match justfile_path {
            Some(p) => p,
            None => return CommandSupport::Unknown,
        };

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return CommandSupport::Unknown,
        };

        let recipes = extract_just_recipes(&content);

        if recipes.contains(command) {
            return CommandSupport::Supported;
        }

        CommandSupport::NotSupported
    }
}

/// Extract recipe names from justfile content
fn extract_just_recipes(content: &str) -> HashSet<String> {
    let mut recipes = HashSet::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines, comments, and lines starting with whitespace (recipe body)
        if trimmed.is_empty() || trimmed.starts_with('#') || line.starts_with(char::is_whitespace) {
            continue;
        }

        // Skip variable assignments (contain :=)
        if trimmed.contains(":=") {
            continue;
        }

        // Skip set/alias/export directives
        if trimmed.starts_with("set ")
            || trimmed.starts_with("alias ")
            || trimmed.starts_with("export ")
            || trimmed.starts_with("import ")
            || trimmed.starts_with("mod ")
        {
            continue;
        }

        // Recipe pattern: name args? ':'
        // Examples:
        //   build:
        //   test *args:
        //   deploy target='prod':
        //   @hidden:
        if let Some(colon_pos) = trimmed.find(':') {
            let before_colon = &trimmed[..colon_pos];

            // Handle @ prefix for quiet recipes
            let name_part = before_colon.trim_start_matches('@');

            // Extract just the recipe name (before any parameters)
            let recipe_name = name_part
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_start_matches('['); // Handle attributes like [private]

            // Skip if it looks like a dependency reference or attribute
            if !recipe_name.is_empty()
                && !recipe_name.starts_with('[')
                && !recipe_name.contains('=')
            {
                recipes.insert(recipe_name.to_string());
            }
        }
    }

    recipes
}

/// Detect Just command runner
/// Priority: 10 (between PHP and Go, as it's a generic task runner)
pub fn detect(dir: &Path) -> Vec<DetectedRunner> {
    let mut runners = Vec::new();

    let justfile_priority = ["justfile", "Justfile", ".justfile"];

    // Use read_dir to get exact filenames (case-sensitive on all platforms)
    if let Ok(entries) = std::fs::read_dir(dir) {
        let files: HashSet<String> = entries
            .flatten()
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();

        for &target in &justfile_priority {
            if files.contains(target) {
                let validator: Arc<dyn CommandValidator> = Arc::new(JustValidator);
                runners.push(DetectedRunner::with_validator(
                    "just",
                    target,
                    Ecosystem::Generic,
                    10, // Priority 10 - between PHP (10) and Go (11)
                    validator,
                ));
                break; // Only detect one justfile
            }
        }
    }

    runners
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_detect_justfile_lowercase() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("justfile")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "just");
        assert_eq!(runners[0].detected_file, "justfile");
    }

    #[test]
    fn test_detect_justfile_capitalized() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("Justfile")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "just");
        assert_eq!(runners[0].detected_file, "Justfile");
    }

    #[test]
    fn test_detect_justfile_hidden() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join(".justfile")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "just");
        assert_eq!(runners[0].detected_file, ".justfile");
    }

    #[test]
    fn test_no_justfile() {
        let dir = tempdir().unwrap();

        let runners = detect(dir.path());
        assert!(runners.is_empty());
    }

    // Validator tests

    #[test]
    fn test_just_validator_simple_recipes() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("justfile")).unwrap();
        writeln!(
            file,
            r#"
# Build the project
build:
    cargo build --release

# Run tests
test:
    cargo test

# Deploy to production
deploy target='prod':
    ./deploy.sh {{{{target}}}}
"#
        )
        .unwrap();

        let validator = JustValidator;
        assert_eq!(
            validator.supports_command(dir.path(), "build"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir.path(), "test"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir.path(), "deploy"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir.path(), "nonexistent"),
            CommandSupport::NotSupported
        );
    }

    #[test]
    fn test_just_validator_quiet_recipes() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("justfile")).unwrap();
        writeln!(
            file,
            r#"
@quiet-recipe:
    echo "This is quiet"

@another:
    echo "Also quiet"
"#
        )
        .unwrap();

        let validator = JustValidator;
        assert_eq!(
            validator.supports_command(dir.path(), "quiet-recipe"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir.path(), "another"),
            CommandSupport::Supported
        );
    }

    #[test]
    fn test_just_validator_with_variables() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("justfile")).unwrap();
        writeln!(
            file,
            r#"
# Variables should not be treated as recipes
version := "1.0.0"
name := "myapp"

set shell := ["bash", "-c"]

build:
    cargo build
"#
        )
        .unwrap();

        let validator = JustValidator;
        assert_eq!(
            validator.supports_command(dir.path(), "build"),
            CommandSupport::Supported
        );
        // Variables should not be detected as recipes
        assert_eq!(
            validator.supports_command(dir.path(), "version"),
            CommandSupport::NotSupported
        );
        assert_eq!(
            validator.supports_command(dir.path(), "name"),
            CommandSupport::NotSupported
        );
    }

    #[test]
    fn test_just_validator_with_parameters() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("justfile")).unwrap();
        writeln!(
            file,
            r#"
# Recipe with positional args
greet name:
    echo "Hello, {{{{name}}}}!"

# Recipe with variadic args
test *args:
    cargo test {{{{args}}}}

# Recipe with default value
deploy env='staging':
    ./deploy.sh {{{{env}}}}
"#
        )
        .unwrap();

        let validator = JustValidator;
        assert_eq!(
            validator.supports_command(dir.path(), "greet"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir.path(), "test"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir.path(), "deploy"),
            CommandSupport::Supported
        );
    }

    #[test]
    fn test_just_validator_no_file() {
        let dir = tempdir().unwrap();

        let validator = JustValidator;
        assert_eq!(
            validator.supports_command(dir.path(), "anything"),
            CommandSupport::Unknown
        );
    }

    #[test]
    fn test_detected_runner_has_working_validator() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("justfile")).unwrap();
        writeln!(
            file,
            r#"
build:
    cargo build

test:
    cargo test
"#
        )
        .unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "just");

        // Verify the detected runner has a working validator
        assert_eq!(
            runners[0].supports_command("build", dir.path()),
            CommandSupport::Supported
        );
        assert_eq!(
            runners[0].supports_command("test", dir.path()),
            CommandSupport::Supported
        );
        assert_eq!(
            runners[0].supports_command("nonexistent", dir.path()),
            CommandSupport::NotSupported
        );
    }

    #[test]
    fn test_extract_just_recipes() {
        let content = r#"
# Comment
version := "1.0.0"

set shell := ["bash", "-c"]

build:
    cargo build

@quiet:
    echo quiet

test *args:
    cargo test

deploy target='prod':
    ./deploy.sh
"#;
        let recipes = extract_just_recipes(content);
        assert!(recipes.contains("build"));
        assert!(recipes.contains("quiet"));
        assert!(recipes.contains("test"));
        assert!(recipes.contains("deploy"));
        assert!(!recipes.contains("version"));
        assert!(!recipes.contains("set"));
    }
}
