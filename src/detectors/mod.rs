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

pub mod custom;
pub mod deno;
pub mod dotnet;
pub mod elixir;
pub mod go;
pub mod java;
pub mod just;
pub mod make;
pub mod monorepo;
pub mod node;
pub mod php;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod swift;
pub mod zig;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Indicates if a command is supported by a runner
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandSupport {
    /// The command is explicitly supported (e.g., found in package.json scripts)
    Supported,
    /// The command is definitely not supported (e.g., not found in package.json scripts)
    NotSupported,
    /// It's unknown if the command is supported (e.g., no manifest parsing implemented)
    Unknown,
}

/// Trait for validating commands against a specific detector.
/// Implementations must be Send + Sync to allow sharing across threads.
pub trait CommandValidator: Send + Sync {
    /// Check if the detected runner supports the given command
    fn supports_command(&self, working_dir: &Path, command: &str) -> CommandSupport;
}

/// Default validator that returns Unknown for all commands.
/// Used for runners that don't have specific validation logic yet.
pub struct UnknownValidator;

impl CommandValidator for UnknownValidator {
    fn supports_command(&self, _working_dir: &Path, _command: &str) -> CommandSupport {
        CommandSupport::Unknown
    }
}

/// Represents a detected runner with its command and configuration
pub struct DetectedRunner {
    /// Name of the runner (e.g., "pnpm", "cargo", "poetry")
    pub name: String,
    /// The file that triggered detection
    pub detected_file: String,
    /// The ecosystem this runner belongs to
    pub ecosystem: Ecosystem,
    /// Priority (lower = higher priority)
    pub priority: u8,
    /// Validator for checking command support
    validator: Arc<dyn CommandValidator>,
    /// Custom commands defined by the user (if any)
    pub custom_commands: Option<HashMap<String, String>>,
}

impl std::fmt::Debug for DetectedRunner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DetectedRunner")
            .field("name", &self.name)
            .field("detected_file", &self.detected_file)
            .field("ecosystem", &self.ecosystem)
            .field("priority", &self.priority)
            .field("validator", &"<dyn CommandValidator>")
            .field("custom_commands", &self.custom_commands)
            .finish()
    }
}

impl Clone for DetectedRunner {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            detected_file: self.detected_file.clone(),
            ecosystem: self.ecosystem,
            priority: self.priority,
            validator: Arc::clone(&self.validator),
            custom_commands: self.custom_commands.clone(),
        }
    }
}

impl PartialEq for DetectedRunner {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.detected_file == other.detected_file
            && self.ecosystem == other.ecosystem
            && self.priority == other.priority
            && self.custom_commands == other.custom_commands
    }
}

impl DetectedRunner {
    /// Create a new DetectedRunner with an UnknownValidator.
    /// Use `with_validator` for runners with specific validation logic.
    pub fn new(name: &str, detected_file: &str, ecosystem: Ecosystem, priority: u8) -> Self {
        Self::with_validator(
            name,
            detected_file,
            ecosystem,
            priority,
            Arc::new(UnknownValidator),
        )
    }

    /// Create a new DetectedRunner with a specific validator.
    pub fn with_validator(
        name: &str,
        detected_file: &str,
        ecosystem: Ecosystem,
        priority: u8,
        validator: Arc<dyn CommandValidator>,
    ) -> Self {
        Self {
            name: name.to_string(),
            detected_file: detected_file.to_string(),
            ecosystem,
            priority,
            validator,
            custom_commands: None,
        }
    }

    /// Create a new DetectedRunner with custom commands
    pub fn with_custom_commands(
        name: &str,
        detected_file: &str,
        ecosystem: Ecosystem,
        priority: u8,
        validator: Arc<dyn CommandValidator>,
        custom_commands: HashMap<String, String>,
    ) -> Self {
        Self {
            name: name.to_string(),
            detected_file: detected_file.to_string(),
            ecosystem,
            priority,
            validator,
            custom_commands: Some(custom_commands),
        }
    }

    /// Check if this runner supports the given command.
    pub fn supports_command(&self, command: &str, working_dir: &Path) -> CommandSupport {
        self.validator.supports_command(working_dir, command)
    }

    /// Build the command to execute
    pub fn build_command(&self, task: &str, extra_args: &[String]) -> Vec<String> {
        // First check if this is a custom command
        if let Some(commands) = &self.custom_commands {
            if let Some(cmd_str) = commands.get(task) {
                let mut parts = match shell_words::split(cmd_str) {
                    Ok(p) => p,
                    Err(_) => {
                        // Fallback to simple whitespace splitting if parsing fails
                        // or should we handle error? For now, fallback seems safe-ish
                        cmd_str.split_whitespace().map(|s| s.to_string()).collect()
                    }
                };
                parts.extend(extra_args.iter().cloned());
                return parts;
            }
        }

        let mut cmd = match self.name.as_str() {
            // Node.js ecosystem
            "bun" => vec!["bun".to_string(), "run".to_string(), task.to_string()],
            "pnpm" => vec!["pnpm".to_string(), "run".to_string(), task.to_string()],
            "yarn" => vec!["yarn".to_string(), "run".to_string(), task.to_string()],
            "npm" => vec!["npm".to_string(), "run".to_string(), task.to_string()],

            // Python ecosystem
            "rye" => vec!["rye".to_string(), "run".to_string(), task.to_string()],
            "uv" => vec!["uv".to_string(), "run".to_string(), task.to_string()],
            "poetry" => vec!["poetry".to_string(), "run".to_string(), task.to_string()],
            "pipenv" => vec!["pipenv".to_string(), "run".to_string(), task.to_string()],
            "pip" => vec!["python".to_string(), "-m".to_string(), task.to_string()],

            // Rust ecosystem
            "cargo" => vec!["cargo".to_string(), task.to_string()],

            // Deno ecosystem
            "deno" => {
                if deno::DENO_BUILTIN.contains(&task) {
                    vec!["deno".to_string(), task.to_string()]
                } else if task.contains('/') || task.ends_with(".ts") || task.ends_with(".js") {
                    vec!["deno".to_string(), "run".to_string(), task.to_string()]
                } else {
                    vec!["deno".to_string(), "task".to_string(), task.to_string()]
                }
            }

            // PHP ecosystem
            "composer" => vec!["composer".to_string(), "run".to_string(), task.to_string()],

            // Go ecosystem
            "task" => vec!["task".to_string(), task.to_string()],
            "go" => {
                // Check if task looks like a path (contains / or ends with .go)
                if task.contains('/') || task.ends_with(".go") {
                    vec!["go".to_string(), "run".to_string(), task.to_string()]
                } else {
                    vec!["go".to_string(), task.to_string()]
                }
            }

            // Ruby ecosystem
            "bundler" => vec!["bundle".to_string(), "exec".to_string(), task.to_string()],
            "rake" => vec!["rake".to_string(), task.to_string()],

            // Java ecosystem
            "gradle" => vec!["gradle".to_string(), task.to_string()],
            "maven" => vec!["mvn".to_string(), task.to_string()],

            // .NET ecosystem
            "dotnet" => vec!["dotnet".to_string(), task.to_string()],

            // Elixir ecosystem
            "mix" => vec!["mix".to_string(), task.to_string()],

            // Swift ecosystem
            "swift" => vec!["swift".to_string(), "run".to_string(), task.to_string()],

            // Zig ecosystem
            "zig" => vec!["zig".to_string(), "build".to_string(), task.to_string()],

            // Just command runner
            "just" => vec!["just".to_string(), task.to_string()],

            // Monorepo orchestration tools
            "nx" => vec!["nx".to_string(), task.to_string()],
            "turbo" => vec!["turbo".to_string(), "run".to_string(), task.to_string()],
            "lerna" => vec!["lerna".to_string(), "run".to_string(), task.to_string()],

            // Generic
            "make" => vec!["make".to_string(), task.to_string()],

            // Fallback
            _ => vec![self.name.clone(), task.to_string()],
        };

        cmd.extend(extra_args.iter().cloned());
        cmd
    }
}

/// Ecosystem categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ecosystem {
    NodeJs,
    Python,
    Rust,
    Deno,
    Php,
    Go,
    Ruby,
    Java,
    DotNet,
    Elixir,
    Swift,
    Zig,
    Generic,
    Custom,
}

impl Ecosystem {
    pub fn as_str(&self) -> &'static str {
        match self {
            Ecosystem::NodeJs => "Node.js",
            Ecosystem::Python => "Python",
            Ecosystem::Rust => "Rust",
            Ecosystem::Deno => "Deno",
            Ecosystem::Php => "PHP",
            Ecosystem::Go => "Go",
            Ecosystem::Ruby => "Ruby",
            Ecosystem::Java => "Java",
            Ecosystem::DotNet => ".NET",
            Ecosystem::Elixir => "Elixir",
            Ecosystem::Swift => "Swift",
            Ecosystem::Zig => "Zig",
            Ecosystem::Generic => "Generic",
            Ecosystem::Custom => "Custom",
        }
    }
}

/// Detect all runners in the given directory
pub fn detect_all(dir: &Path, ignore_list: &[String]) -> Vec<DetectedRunner> {
    let mut runners = Vec::new();

    // Helper to add runners if not ignored
    let mut add_runners = |detected: Vec<DetectedRunner>| {
        for runner in detected {
            if !ignore_list
                .iter()
                .any(|i| i.eq_ignore_ascii_case(&runner.name))
            {
                runners.push(runner);
            }
        }
    };

    // Run all detectors in priority order
    add_runners(custom::detect(dir)); // Custom commands (0) - highest priority
    add_runners(monorepo::detect(dir)); // Monorepo tools (0) - highest priority
    add_runners(node::detect(dir)); // Node.js (1-4)
    add_runners(python::detect(dir)); // Python (5-8)
    add_runners(rust::detect(dir)); // Rust (9)
    add_runners(php::detect(dir)); // PHP (10)
    add_runners(just::detect(dir)); // Just (10)
    add_runners(deno::detect(dir)); // Deno (22)
    add_runners(go::detect(dir)); // Go (11-12)
    add_runners(ruby::detect(dir)); // Ruby (13-14)
    add_runners(java::detect(dir)); // Java (15-16)
    add_runners(dotnet::detect(dir)); // .NET (17)
    add_runners(elixir::detect(dir)); // Elixir (18)
    add_runners(swift::detect(dir)); // Swift (19)
    add_runners(zig::detect(dir)); // Zig (20)
    add_runners(make::detect(dir)); // Make (21)

    // Sort by priority
    runners.sort_by_key(|r| r.priority);
    runners
}

/// Check if a tool is installed on the system
pub fn is_tool_installed(tool: &str) -> bool {
    which::which(tool).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_build_command_npm() {
        let runner = DetectedRunner::new("npm", "package.json", Ecosystem::NodeJs, 4);
        let cmd = runner.build_command("test", &[]);
        assert_eq!(cmd, vec!["npm", "run", "test"]);
    }

    #[test]
    fn test_build_command_with_args() {
        let runner = DetectedRunner::new("npm", "package.json", Ecosystem::NodeJs, 4);
        let cmd = runner.build_command("test", &["--coverage".to_string()]);
        assert_eq!(cmd, vec!["npm", "run", "test", "--coverage"]);
    }

    #[test]
    fn test_build_command_rye() {
        let runner = DetectedRunner::new("rye", "pyproject.toml", Ecosystem::Python, 5);
        let cmd = runner.build_command("test", &[]);
        assert_eq!(cmd, vec!["rye", "run", "test"]);
    }

    #[test]
    fn test_build_command_cargo() {
        let runner = DetectedRunner::new("cargo", "Cargo.toml", Ecosystem::Rust, 9);
        let cmd = runner.build_command("build", &["--release".to_string()]);
        assert_eq!(cmd, vec!["cargo", "build", "--release"]);
    }

    #[test]
    fn test_build_command_go_path() {
        let runner = DetectedRunner::new("go", "go.mod", Ecosystem::Go, 12);
        let cmd = runner.build_command("./cmd/main.go", &[]);
        assert_eq!(cmd, vec!["go", "run", "./cmd/main.go"]);
    }

    #[test]
    fn test_build_command_go_task() {
        let runner = DetectedRunner::new("go", "go.mod", Ecosystem::Go, 12);
        let cmd = runner.build_command("build", &[]);
        assert_eq!(cmd, vec!["go", "build"]);
    }

    // Validator integration tests (moved from validators.rs)

    #[test]
    fn test_node_script_supported() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("package.json")).unwrap();
        writeln!(file, r#"{{"scripts": {{"test": "jest", "build": "tsc"}}}}"#).unwrap();

        let runner = DetectedRunner::with_validator(
            "npm",
            "package.json",
            Ecosystem::NodeJs,
            4,
            Arc::new(node::NodeValidator),
        );
        assert_eq!(
            runner.supports_command("test", dir.path()),
            CommandSupport::Supported
        );
        assert_eq!(
            runner.supports_command("build", dir.path()),
            CommandSupport::Supported
        );
        assert_eq!(
            runner.supports_command("nonexistent", dir.path()),
            CommandSupport::NotSupported
        );
    }

    #[test]
    fn test_cargo_builtin() {
        let dir = tempdir().unwrap();
        let runner = DetectedRunner::with_validator(
            "cargo",
            "Cargo.toml",
            Ecosystem::Rust,
            9,
            Arc::new(rust::RustValidator),
        );

        assert_eq!(
            runner.supports_command("build", dir.path()),
            CommandSupport::Supported
        );
        assert_eq!(
            runner.supports_command("test", dir.path()),
            CommandSupport::Supported
        );
        assert_eq!(
            runner.supports_command("clippy", dir.path()),
            CommandSupport::Supported
        );
        assert_eq!(
            runner.supports_command("precommit", dir.path()),
            CommandSupport::Unknown
        );
    }

    #[test]
    fn test_make_target_supported() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("Makefile")).unwrap();
        writeln!(
            file,
            r#"
.PHONY: build test precommit

build:
	cargo build

test:
	cargo test

precommit: build test
	@echo "Done"
"#
        )
        .unwrap();

        let runner = DetectedRunner::with_validator(
            "make",
            "Makefile",
            Ecosystem::Generic,
            21,
            Arc::new(make::MakeValidator),
        );
        assert_eq!(
            runner.supports_command("build", dir.path()),
            CommandSupport::Supported
        );
        assert_eq!(
            runner.supports_command("precommit", dir.path()),
            CommandSupport::Supported
        );
        assert_eq!(
            runner.supports_command("nonexistent", dir.path()),
            CommandSupport::NotSupported
        );
    }

    #[test]
    fn test_composer_script() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("composer.json")).unwrap();
        writeln!(file, r#"{{"scripts": {{"test": "phpunit"}}}}"#).unwrap();

        let runner = DetectedRunner::with_validator(
            "composer",
            "composer.json",
            Ecosystem::Php,
            10,
            Arc::new(php::PhpValidator),
        );
        assert_eq!(
            runner.supports_command("test", dir.path()),
            CommandSupport::Supported
        );
        assert_eq!(
            runner.supports_command("nonexistent", dir.path()),
            CommandSupport::NotSupported
        );
    }

    #[test]
    fn test_gradle_task() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("build.gradle")).unwrap();
        writeln!(file, r#"task customTask {{}}"#).unwrap();

        let runner = DetectedRunner::with_validator(
            "gradle",
            "build.gradle",
            Ecosystem::Java,
            15,
            Arc::new(java::JavaValidator),
        );
        assert_eq!(
            runner.supports_command("customTask", dir.path()),
            CommandSupport::Supported
        );
        assert_eq!(
            runner.supports_command("build", dir.path()),
            CommandSupport::Supported
        );
    }

    #[test]
    fn test_dotnet_builtin() {
        let dir = tempdir().unwrap();
        let runner = DetectedRunner::with_validator(
            "dotnet",
            "test.csproj",
            Ecosystem::DotNet,
            17,
            Arc::new(dotnet::DotNetValidator),
        );

        assert_eq!(
            runner.supports_command("build", dir.path()),
            CommandSupport::Supported
        );
        assert_eq!(
            runner.supports_command("test", dir.path()),
            CommandSupport::Supported
        );
        assert_eq!(
            runner.supports_command("nonexistent", dir.path()),
            CommandSupport::NotSupported
        );
    }

    #[test]
    fn test_unknown_validator_returns_unknown() {
        let dir = tempdir().unwrap();
        let runner = DetectedRunner::new("unknown", "file", Ecosystem::Generic, 100);
        assert_eq!(
            runner.supports_command("anything", dir.path()),
            CommandSupport::Unknown
        );
    }

    #[test]
    fn test_build_custom_command() {
        let mut commands = HashMap::new();
        commands.insert("hello".to_string(), "echo 'hello world'".to_string());

        let runner = DetectedRunner::with_custom_commands(
            "custom",
            "run.toml",
            Ecosystem::Custom,
            0,
            Arc::new(UnknownValidator),
            commands,
        );

        let cmd = runner.build_command("hello", &[]);
        assert_eq!(cmd, vec!["echo", "hello world"]);
    }
}
