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

impl CommandValidator for DenoValidator {
    fn supports_command(&self, working_dir: &Path, command: &str) -> CommandSupport {
        // Try deno.json first
        let deno_json = working_dir.join("deno.json");
        if deno_json.exists() {
            if let Ok(content) = fs::read_to_string(&deno_json) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(tasks) = json.get("tasks").and_then(|t| t.as_object()) {
                        if tasks.contains_key(command) {
                            return CommandSupport::Supported;
                        }
                    }
                }
            }
        }

        // Try deno.jsonc
        let deno_jsonc = working_dir.join("deno.jsonc");
        if deno_jsonc.exists() {
            // Check if file exists, but for now we might fail parsing comments with standard serde_json
            // If we can't parse, we can't be sure, but existence suggests it might be supported
            // However, to be safe, we return Unknown if we can't confirm.
            // A simple heuristic: check if the file contains "tasks" and the command name
            if let Ok(content) = fs::read_to_string(&deno_jsonc) {
                // Very basic heuristic to avoid full JSONC parsing complexity
                if content.contains("\"tasks\"") && content.contains(&format!("\"{}\"", command)) {
                    // This is a weak check, but better than nothing for JSONC without a parser
                    // It might return false positives if the command name is mentioned elsewhere
                    // but Deno runner will fail if task doesn't exist, which is acceptable
                    return CommandSupport::Supported;
                }
            }
        }

        // Also support running files directly if they exist
        if (command.ends_with(".ts")
            || command.ends_with(".js")
            || command.ends_with(".tsx")
            || command.ends_with(".jsx"))
            && working_dir.join(command).exists()
        {
            return CommandSupport::Supported;
        }

        CommandSupport::NotSupported
    }
}

/// Detect Deno runner
/// Priority: 5 (shared with Python?) or unique. Let's use 5.
pub fn detect(dir: &Path) -> Vec<DetectedRunner> {
    let mut runners = Vec::new();
    let validator: Arc<dyn CommandValidator> = Arc::new(DenoValidator);

    // Check for deno.json, deno.jsonc, or deno.lock
    let deno_json = dir.join("deno.json");
    let deno_jsonc = dir.join("deno.jsonc");
    let deno_lock = dir.join("deno.lock");

    if deno_json.exists() {
        runners.push(DetectedRunner::with_validator(
            "deno",
            "deno.json",
            Ecosystem::Deno,
            5,
            Arc::clone(&validator),
        ));
    } else if deno_jsonc.exists() {
        runners.push(DetectedRunner::with_validator(
            "deno",
            "deno.jsonc",
            Ecosystem::Deno,
            5,
            Arc::clone(&validator),
        ));
    } else if deno_lock.exists() {
        runners.push(DetectedRunner::with_validator(
            "deno",
            "deno.lock",
            Ecosystem::Deno,
            5,
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
    fn test_validator_supports_task() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("deno.json")).unwrap();
        writeln!(file, r#"{{"tasks": {{"start": "deno run main.ts"}}}}"#).unwrap();

        let validator = DenoValidator;
        assert_eq!(
            validator.supports_command(dir.path(), "start"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir.path(), "test"),
            CommandSupport::NotSupported
        );
    }

    #[test]
    fn test_validator_supports_file() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("main.ts")).unwrap();

        let validator = DenoValidator;
        assert_eq!(
            validator.supports_command(dir.path(), "main.ts"),
            CommandSupport::Supported
        );
    }
}
