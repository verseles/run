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

/// Validator for Elixir Mix projects
pub struct MixValidator;

impl CommandValidator for MixValidator {
    fn supports_command(&self, working_dir: &Path, command: &str) -> CommandSupport {
        let mix_exs = working_dir.join("mix.exs");
        if !mix_exs.exists() {
            return CommandSupport::Unknown;
        }

        let content = match fs::read_to_string(&mix_exs) {
            Ok(c) => c,
            Err(_) => return CommandSupport::Unknown,
        };

        // Extract aliases from mix.exs
        let aliases = extract_mix_aliases(&content);

        // Check built-in Mix tasks
        const BUILTINS: &[&str] = &[
            "compile",
            "test",
            "run",
            "deps",
            "deps.get",
            "deps.update",
            "deps.compile",
            "deps.clean",
            "ecto.create",
            "ecto.migrate",
            "ecto.rollback",
            "ecto.drop",
            "ecto.setup",
            "ecto.reset",
            "phx.server",
            "phx.routes",
            "phx.gen.html",
            "phx.gen.json",
            "phx.gen.context",
            "phx.gen.schema",
            "phx.gen.live",
            "phx.new",
            "phx.digest",
            "format",
            "clean",
            "hex.info",
            "hex.publish",
            "help",
            "new",
            "archive.install",
            "local.hex",
            "local.rebar",
            "release",
            "xref",
            "profile.cprof",
            "profile.eprof",
            "profile.fprof",
        ];

        if aliases.contains(command) || BUILTINS.contains(&command) {
            return CommandSupport::Supported;
        }

        // Mix is extensible (custom tasks, deps tasks, etc.)
        // Return Unknown to allow fallback behavior
        CommandSupport::Unknown
    }
}

/// Extract aliases from mix.exs file content
fn extract_mix_aliases(content: &str) -> HashSet<String> {
    let mut aliases = HashSet::new();

    // Find the aliases block - look for patterns like:
    // defp aliases do
    //   [
    //     setup: [...],
    //     "db.reset": [...],
    //     test: [...]
    //   ]
    // end

    // Try to find "aliases" function or inline aliases
    if let Some(start_idx) = content.find("aliases") {
        // Find the opening bracket after aliases
        let after_aliases = &content[start_idx..];
        if let Some(bracket_idx) = after_aliases.find('[') {
            let from_bracket = &after_aliases[bracket_idx..];

            // Find matching closing bracket
            let mut depth = 0;
            let mut end_idx = 0;
            for (i, c) in from_bracket.char_indices() {
                match c {
                    '[' => depth += 1,
                    ']' => {
                        depth -= 1;
                        if depth == 0 {
                            end_idx = i;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            if end_idx > 0 {
                let aliases_block = &from_bracket[1..end_idx];

                // Extract atom keys: `name:` (without quotes)
                for cap in aliases_block.split(',') {
                    let trimmed = cap.trim();

                    // Pattern: atom_key: ... (e.g., "setup:", "test:")
                    if let Some(colon_pos) = trimmed.find(':') {
                        let key = trimmed[..colon_pos].trim();

                        // Skip if it looks like it's inside a list value
                        if key.starts_with('[') || key.starts_with('"') && key.ends_with('"') {
                            // String key like "db.reset": ...
                            if key.starts_with('"') && key.len() > 2 {
                                let unquoted = &key[1..key.len() - 1];
                                if !unquoted.is_empty()
                                    && !unquoted.contains(' ')
                                    && !unquoted.contains('[')
                                {
                                    aliases.insert(unquoted.to_string());
                                }
                            }
                        } else if !key.is_empty()
                            && !key.contains(' ')
                            && !key.contains('[')
                            && !key.contains('"')
                            && !key.starts_with('&')
                        {
                            // Atom key like setup:, test:
                            aliases.insert(key.to_string());
                        }
                    }
                }
            }
        }
    }

    aliases
}

/// Detect Elixir projects (Mix)
/// Priority: 18
pub fn detect(dir: &Path) -> Vec<DetectedRunner> {
    let mut runners = Vec::new();

    let mix_exs = dir.join("mix.exs");

    // mix.exs is sufficient for detection (mix.lock is optional)
    if mix_exs.exists() {
        let validator: Arc<dyn CommandValidator> = Arc::new(MixValidator);
        runners.push(DetectedRunner::with_validator(
            "mix",
            "mix.exs",
            Ecosystem::Elixir,
            18,
            validator,
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
    fn test_detect_mix_with_lock() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("mix.exs")).unwrap();
        File::create(dir.path().join("mix.lock")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "mix");
    }

    #[test]
    fn test_detect_mix_without_lock() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("mix.exs")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "mix");
    }

    #[test]
    fn test_no_mix() {
        let dir = tempdir().unwrap();

        let runners = detect(dir.path());
        assert!(runners.is_empty());
    }

    // Mix validator tests

    #[test]
    fn test_mix_validator_builtins() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("mix.exs")).unwrap();

        let validator = MixValidator;
        assert_eq!(
            validator.supports_command(dir.path(), "compile"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir.path(), "test"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir.path(), "deps.get"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir.path(), "phx.server"),
            CommandSupport::Supported
        );
    }

    #[test]
    fn test_mix_validator_aliases() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("mix.exs")).unwrap();
        writeln!(
            file,
            r#"
defmodule Example.MixProject do
  use Mix.Project

  def project do
    [
      app: :example,
      version: "1.0.0",
      aliases: aliases()
    ]
  end

  defp aliases do
    [
      setup: ["deps.get", "ecto.setup"],
      "db.reset": ["ecto.drop", "ecto.create", "ecto.migrate"],
      seed: ["run priv/repo/seeds.exs"]
    ]
  end
end
"#
        )
        .unwrap();

        let validator = MixValidator;
        assert_eq!(
            validator.supports_command(dir.path(), "setup"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir.path(), "db.reset"),
            CommandSupport::Supported
        );
        assert_eq!(
            validator.supports_command(dir.path(), "seed"),
            CommandSupport::Supported
        );
        // Unknown command returns Unknown (Mix is extensible)
        assert_eq!(
            validator.supports_command(dir.path(), "nonexistent"),
            CommandSupport::Unknown
        );
    }

    #[test]
    fn test_mix_validator_no_aliases() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("mix.exs")).unwrap();
        writeln!(
            file,
            r#"
defmodule Example.MixProject do
  use Mix.Project

  def project do
    [app: :example, version: "1.0.0"]
  end
end
"#
        )
        .unwrap();

        let validator = MixValidator;
        // Built-ins should still work
        assert_eq!(
            validator.supports_command(dir.path(), "compile"),
            CommandSupport::Supported
        );
        // Custom commands return Unknown
        assert_eq!(
            validator.supports_command(dir.path(), "custom"),
            CommandSupport::Unknown
        );
    }

    #[test]
    fn test_mix_validator_no_file() {
        let dir = tempdir().unwrap();

        let validator = MixValidator;
        assert_eq!(
            validator.supports_command(dir.path(), "anything"),
            CommandSupport::Unknown
        );
    }

    #[test]
    fn test_detected_runner_has_working_validator() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("mix.exs")).unwrap();
        writeln!(
            file,
            r#"
defmodule Example.MixProject do
  use Mix.Project

  def project do
    [app: :example, aliases: aliases()]
  end

  defp aliases do
    [setup: ["deps.get"]]
  end
end
"#
        )
        .unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "mix");

        // Verify the detected runner has a working validator
        assert_eq!(
            runners[0].supports_command("setup", dir.path()),
            CommandSupport::Supported
        );
        assert_eq!(
            runners[0].supports_command("compile", dir.path()),
            CommandSupport::Supported
        );
    }

    #[test]
    fn test_extract_mix_aliases() {
        let content = r#"
defmodule Example.MixProject do
  defp aliases do
    [
      setup: ["deps.get", "ecto.setup"],
      "db.reset": ["ecto.drop", "ecto.create"],
      test: ["ecto.create --quiet", "test"],
      seed: &run_seeds/1
    ]
  end
end
"#;
        let aliases = extract_mix_aliases(content);
        assert!(aliases.contains("setup"));
        assert!(aliases.contains("db.reset"));
        assert!(aliases.contains("test"));
        assert!(aliases.contains("seed"));
    }
}
