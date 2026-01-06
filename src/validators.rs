// Copyright (C) 2025 Verseles
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, version 3 of the License.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::detectors::DetectedRunner;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandSupport {
    Supported,
    NotSupported,
    Unknown,
}

impl DetectedRunner {
    pub fn supports_command(&self, command: &str, working_dir: &Path) -> CommandSupport {
        match self.name.as_str() {
            "npm" | "yarn" | "pnpm" | "bun" => check_node_script(command, working_dir),
            "cargo" => check_cargo_command(command),
            "make" => check_make_target(command, working_dir),
            "composer" => check_composer_script(command, working_dir),
            "gradle" => check_gradle_task(command, working_dir),
            "maven" => CommandSupport::Unknown,
            "poetry" | "pipenv" | "uv" | "pip" => CommandSupport::Unknown,
            "go" | "task" => CommandSupport::Unknown,
            "bundler" | "rake" => check_rake_task(command, working_dir),
            "dotnet" => check_dotnet_command(command),
            "mix" => CommandSupport::Unknown,
            "swift" => CommandSupport::Unknown,
            "zig" => CommandSupport::Unknown,
            _ => CommandSupport::Unknown,
        }
    }
}

fn check_node_script(command: &str, working_dir: &Path) -> CommandSupport {
    let package_json = working_dir.join("package.json");
    if !package_json.exists() {
        return CommandSupport::Unknown;
    }

    let content = match fs::read_to_string(&package_json) {
        Ok(c) => c,
        Err(_) => return CommandSupport::Unknown,
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return CommandSupport::Unknown,
    };

    if let Some(scripts) = json.get("scripts").and_then(|s| s.as_object()) {
        if scripts.contains_key(command) {
            return CommandSupport::Supported;
        }
        return CommandSupport::NotSupported;
    }

    CommandSupport::Unknown
}

fn check_cargo_command(command: &str) -> CommandSupport {
    static CARGO_BUILTIN: &[&str] = &[
        "build",
        "b",
        "check",
        "c",
        "clean",
        "doc",
        "d",
        "new",
        "init",
        "add",
        "remove",
        "run",
        "r",
        "test",
        "t",
        "bench",
        "update",
        "search",
        "publish",
        "install",
        "uninstall",
        "clippy",
        "fmt",
        "fix",
        "tree",
        "vendor",
        "verify-project",
        "version",
        "yank",
        "help",
        "generate-lockfile",
        "locate-project",
        "metadata",
        "pkgid",
        "fetch",
        "login",
        "logout",
        "owner",
        "package",
        "report",
        "rustc",
        "rustdoc",
    ];

    if CARGO_BUILTIN.contains(&command) {
        return CommandSupport::Supported;
    }

    CommandSupport::NotSupported
}

fn check_make_target(command: &str, working_dir: &Path) -> CommandSupport {
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

fn check_composer_script(command: &str, working_dir: &Path) -> CommandSupport {
    let composer_json = working_dir.join("composer.json");
    if !composer_json.exists() {
        return CommandSupport::Unknown;
    }

    let content = match fs::read_to_string(&composer_json) {
        Ok(c) => c,
        Err(_) => return CommandSupport::Unknown,
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return CommandSupport::Unknown,
    };

    if let Some(scripts) = json.get("scripts").and_then(|s| s.as_object()) {
        if scripts.contains_key(command) {
            return CommandSupport::Supported;
        }
        return CommandSupport::NotSupported;
    }

    CommandSupport::Unknown
}

fn check_gradle_task(command: &str, working_dir: &Path) -> CommandSupport {
    static GRADLE_BUILTIN: &[&str] = &[
        "build",
        "clean",
        "test",
        "check",
        "assemble",
        "jar",
        "classes",
        "testClasses",
        "javadoc",
        "dependencies",
        "projects",
        "properties",
        "tasks",
        "help",
        "wrapper",
        "init",
        "buildEnvironment",
        "components",
        "model",
        "dependencyInsight",
    ];

    if GRADLE_BUILTIN.contains(&command) {
        return CommandSupport::Supported;
    }

    let build_gradle = working_dir.join("build.gradle");
    let build_gradle_kts = working_dir.join("build.gradle.kts");

    let content = if build_gradle.exists() {
        fs::read_to_string(&build_gradle).ok()
    } else if build_gradle_kts.exists() {
        fs::read_to_string(&build_gradle_kts).ok()
    } else {
        None
    };

    if let Some(content) = content {
        let task_pattern = format!("task {}", command);
        let task_pattern_paren = format!("task(\"{}\"", command);
        let task_pattern_single = format!("task('{}'", command);

        if content.contains(&task_pattern)
            || content.contains(&task_pattern_paren)
            || content.contains(&task_pattern_single)
        {
            return CommandSupport::Supported;
        }
    }

    CommandSupport::Unknown
}

fn check_rake_task(command: &str, working_dir: &Path) -> CommandSupport {
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

fn check_dotnet_command(command: &str) -> CommandSupport {
    static DOTNET_BUILTIN: &[&str] = &[
        "build",
        "clean",
        "test",
        "run",
        "publish",
        "pack",
        "restore",
        "new",
        "add",
        "remove",
        "list",
        "nuget",
        "tool",
        "workload",
        "watch",
        "format",
        "help",
        "sln",
        "store",
        "msbuild",
        "vstest",
        "dev-certs",
        "fsi",
        "user-secrets",
        "ef",
    ];

    if DOTNET_BUILTIN.contains(&command) {
        return CommandSupport::Supported;
    }

    CommandSupport::NotSupported
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detectors::Ecosystem;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_node_script_supported() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("package.json")).unwrap();
        writeln!(file, r#"{{"scripts": {{"test": "jest", "build": "tsc"}}}}"#).unwrap();

        let runner = DetectedRunner::new("npm", "package.json", Ecosystem::NodeJs, 4);
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
        let runner = DetectedRunner::new("cargo", "Cargo.toml", Ecosystem::Rust, 9);

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
            CommandSupport::NotSupported
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

        let runner = DetectedRunner::new("make", "Makefile", Ecosystem::Generic, 21);
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

        let runner = DetectedRunner::new("composer", "composer.json", Ecosystem::Php, 10);
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
    fn test_dotnet_builtin() {
        let dir = tempdir().unwrap();
        let runner = DetectedRunner::new("dotnet", "test.csproj", Ecosystem::DotNet, 17);

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
}
