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

mod dotnet;
mod elixir;
mod go;
mod java;
mod make;
mod node;
mod php;
mod python;
mod ruby;
mod rust;
mod swift;
mod zig;

use std::path::Path;

/// Represents a detected runner with its command and configuration
#[derive(Debug, Clone, PartialEq)]
pub struct DetectedRunner {
    /// Name of the runner (e.g., "pnpm", "cargo", "poetry")
    pub name: String,
    /// The file that triggered detection
    pub detected_file: String,
    /// The ecosystem this runner belongs to
    pub ecosystem: Ecosystem,
    /// Priority (lower = higher priority)
    pub priority: u8,
}

impl DetectedRunner {
    pub fn new(name: &str, detected_file: &str, ecosystem: Ecosystem, priority: u8) -> Self {
        Self {
            name: name.to_string(),
            detected_file: detected_file.to_string(),
            ecosystem,
            priority,
        }
    }

    /// Build the command to execute
    pub fn build_command(&self, task: &str, extra_args: &[String]) -> Vec<String> {
        let mut cmd = match self.name.as_str() {
            // Node.js ecosystem
            "bun" => vec!["bun".to_string(), "run".to_string(), task.to_string()],
            "pnpm" => vec!["pnpm".to_string(), "run".to_string(), task.to_string()],
            "yarn" => vec!["yarn".to_string(), "run".to_string(), task.to_string()],
            "npm" => vec!["npm".to_string(), "run".to_string(), task.to_string()],

            // Python ecosystem
            "uv" => vec!["uv".to_string(), "run".to_string(), task.to_string()],
            "poetry" => vec!["poetry".to_string(), "run".to_string(), task.to_string()],
            "pipenv" => vec!["pipenv".to_string(), "run".to_string(), task.to_string()],
            "pip" => vec!["python".to_string(), "-m".to_string(), task.to_string()],

            // Rust ecosystem
            "cargo" => vec!["cargo".to_string(), task.to_string()],

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
    Php,
    Go,
    Ruby,
    Java,
    DotNet,
    Elixir,
    Swift,
    Zig,
    Generic,
}

impl Ecosystem {
    pub fn as_str(&self) -> &'static str {
        match self {
            Ecosystem::NodeJs => "Node.js",
            Ecosystem::Python => "Python",
            Ecosystem::Rust => "Rust",
            Ecosystem::Php => "PHP",
            Ecosystem::Go => "Go",
            Ecosystem::Ruby => "Ruby",
            Ecosystem::Java => "Java",
            Ecosystem::DotNet => ".NET",
            Ecosystem::Elixir => "Elixir",
            Ecosystem::Swift => "Swift",
            Ecosystem::Zig => "Zig",
            Ecosystem::Generic => "Generic",
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
    add_runners(node::detect(dir)); // Node.js (1-4)
    add_runners(python::detect(dir)); // Python (5-8)
    add_runners(rust::detect(dir)); // Rust (9)
    add_runners(php::detect(dir)); // PHP (10)
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
}
