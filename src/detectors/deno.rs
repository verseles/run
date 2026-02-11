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
use std::sync::Arc;

pub struct DenoValidator;

pub static DENO_BUILTIN: &[&str] = &[
    "run",
    "test",
    "fmt",
    "lint",
    "check",
    "bench",
    "task",
    "compile",
    "install",
    "uninstall",
    "upgrade",
    "repl",
    "info",
    "doc",
    "types",
    "eval",
    "vendor",
    "cache",
    "coverage",
    "help",
];

impl CommandValidator for DenoValidator {
    fn supports_command(&self, working_dir: &Path, command: &str) -> CommandSupport {
        if DENO_BUILTIN.contains(&command) {
            return CommandSupport::Supported;
        }

        // Check tasks in deno.json or deno.jsonc
        if check_deno_task(working_dir, command) {
            return CommandSupport::Supported;
        }

        CommandSupport::Unknown
    }
}

fn check_deno_task(dir: &Path, command: &str) -> bool {
    let check_file = |path: &Path| -> bool {
        if let Ok(content) = fs::read_to_string(path) {
            // Try parsing as standard JSON first
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(tasks) = json.get("tasks").and_then(|t| t.as_object()) {
                    return tasks.contains_key(command);
                }
            } else {
                // Try removing comments for JSONC
                let stripped = strip_jsonc_comments(&content);
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stripped) {
                    if let Some(tasks) = json.get("tasks").and_then(|t| t.as_object()) {
                        return tasks.contains_key(command);
                    }
                }
            }
        }
        false
    };

    let deno_json = dir.join("deno.json");
    if check_file(&deno_json) {
        return true;
    }

    let deno_jsonc = dir.join("deno.jsonc");
    if check_file(&deno_jsonc) {
        return true;
    }

    false
}

/// Simple JSONC comment stripper
fn strip_jsonc_comments(json: &str) -> String {
    let mut result = String::with_capacity(json.len());
    let mut chars = json.chars().peekable();
    let mut in_string = false;
    let mut escape = false;

    while let Some(c) = chars.next() {
        if in_string {
            result.push(c);
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_string = false;
            }
        } else {
            match c {
                '"' => {
                    result.push(c);
                    in_string = true;
                }
                '/' => {
                    if let Some(&next) = chars.peek() {
                        if next == '/' {
                            // Line comment
                            chars.next(); // Consume second slash
                            for c in chars.by_ref() {
                                if c == '\n' {
                                    result.push(c); // Keep newline
                                    break;
                                }
                            }
                        } else if next == '*' {
                            // Block comment
                            chars.next(); // Consume asterisk
                            while let Some(c) = chars.next() {
                                if c == '*' {
                                    if let Some(&next) = chars.peek() {
                                        if next == '/' {
                                            chars.next(); // Consume slash
                                            break;
                                        }
                                    }
                                }
                            }
                        } else {
                            result.push(c);
                        }
                    } else {
                        result.push(c);
                    }
                }
                _ => result.push(c),
            }
        }
    }
    result
}

/// Detect Deno projects
/// Priority: 22 (after generic/Make, but practically Deno is detected via config files so it's specific)
pub fn detect(dir: &Path) -> Vec<DetectedRunner> {
    let mut runners = Vec::new();
    let validator: Arc<dyn CommandValidator> = Arc::new(DenoValidator);

    // Check for deno.json
    let deno_json = dir.join("deno.json");
    if deno_json.exists() {
        runners.push(DetectedRunner::with_validator(
            "deno",
            "deno.json",
            Ecosystem::Deno, // Need to add this variant to Ecosystem
            22,
            Arc::clone(&validator),
        ));
        return runners;
    }

    // Check for deno.jsonc
    let deno_jsonc = dir.join("deno.jsonc");
    if deno_jsonc.exists() {
        runners.push(DetectedRunner::with_validator(
            "deno",
            "deno.jsonc",
            Ecosystem::Deno,
            22,
            Arc::clone(&validator),
        ));
        return runners;
    }

    // Usually lock files (deno.lock) exist too, but deno.json is primary for tasks
    let deno_lock = dir.join("deno.lock");
    if deno_lock.exists() {
        runners.push(DetectedRunner::with_validator(
            "deno",
            "deno.lock",
            Ecosystem::Deno,
            22,
            Arc::clone(&validator),
        ));
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
    fn test_detect_deno_json() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("deno.json")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "deno");
        assert_eq!(runners[0].detected_file, "deno.json");
    }

    #[test]
    fn test_detect_deno_jsonc() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("deno.jsonc")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "deno");
        assert_eq!(runners[0].detected_file, "deno.jsonc");
    }

    #[test]
    fn test_detect_deno_lock() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("deno.lock")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "deno");
        assert_eq!(runners[0].detected_file, "deno.lock");
    }

    #[test]
    fn test_deno_task_in_json() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("deno.json")).unwrap();
        writeln!(file, r#"{{ "tasks": {{ "start": "deno run main.ts" }} }}"#).unwrap();

        let validator = DenoValidator;
        assert_eq!(
            validator.supports_command(dir.path(), "start"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir.path(), "nonexistent"),
            CommandSupport::Unknown
        );
    }

    #[test]
    fn test_deno_task_in_jsonc_with_comments() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("deno.jsonc")).unwrap();
        writeln!(
            file,
            r#"{{
            // This is a comment
            "tasks": {{
                "dev": "deno run --watch main.ts" /* Block comment */
            }}
        }}"#
        )
        .unwrap();

        let validator = DenoValidator;
        assert_eq!(
            validator.supports_command(dir.path(), "dev"),
            CommandSupport::Supported
        );
    }

    #[test]
    fn test_strip_comments() {
        let jsonc = r#"{
            "key": "value", // line comment
            "key2": "value2 /* inside string */",
            /* block comment */
            "key3": 123
        }"#;
        let stripped = strip_jsonc_comments(jsonc);
        assert!(stripped.contains(r#""key": "value""#));
        assert!(!stripped.contains("line comment"));
        assert!(stripped.contains(r#""key2": "value2 /* inside string */""#));
        assert!(!stripped.contains("block comment"));
        assert!(stripped.contains(r#""key3": 123"#));
    }

    #[test]
    fn test_builtin_commands() {
        let dir = tempdir().unwrap();
        let validator = DenoValidator;
        assert_eq!(
            validator.supports_command(dir.path(), "run"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir.path(), "fmt"),
            CommandSupport::Supported
        );
    }
}
