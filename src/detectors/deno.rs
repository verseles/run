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
        // Try deno.json first, then deno.jsonc
        let mut config_file = working_dir.join("deno.json");
        if !config_file.exists() {
            config_file = working_dir.join("deno.jsonc");
            if !config_file.exists() {
                return CommandSupport::Unknown;
            }
        }

        let content = match fs::read_to_string(&config_file) {
            Ok(c) => c,
            Err(_) => return CommandSupport::Unknown,
        };

        // Try to parse directly. If it fails (due to comments or invalid JSON), we return Unknown.
        // This means for commented config files, we won't know if the command is supported,
        // but the runner will still try to execute it.
        let json: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => return CommandSupport::Unknown,
        };

        if let Some(tasks) = json.get("tasks").and_then(|t| t.as_object()) {
            if tasks.contains_key(command) {
                return CommandSupport::Supported;
            }
            return CommandSupport::NotSupported;
        }

        CommandSupport::Unknown
    }
}

/// Detect Deno runner
/// Priority: 5 (between Node and Python)
pub fn detect(dir: &Path) -> Vec<DetectedRunner> {
    let mut runners = Vec::new();
    let validator: Arc<dyn CommandValidator> = Arc::new(DenoValidator);

    // Check for deno.json or deno.jsonc
    let deno_json = dir.join("deno.json");
    if deno_json.exists() {
        runners.push(DetectedRunner::with_validator(
            "deno",
            "deno.json",
            Ecosystem::Deno,
            5,
            Arc::clone(&validator),
        ));
        return runners;
    }

    let deno_jsonc = dir.join("deno.jsonc");
    if deno_jsonc.exists() {
        runners.push(DetectedRunner::with_validator(
            "deno",
            "deno.jsonc",
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
    fn test_deno_validator_supported() {
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
    fn test_deno_validator_unknown_on_comments() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("deno.jsonc")).unwrap();
        writeln!(
            file,
            r#"
{{
  "tasks": {{
    "start": "deno run main.ts"
  }}
  // This is a comment which might break serde_json
}}
"#
        )
        .unwrap();

        let validator = DenoValidator;
        // Since serde_json doesn't support comments, it should fail parsing and return Unknown
        assert_eq!(
            validator.supports_command(dir.path(), "start"),
            CommandSupport::Unknown
        );
    }
}
