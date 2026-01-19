// Copyright (C) 2025 Verseles
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, version 3 of the License.

use std::path::Path;

use crate::detectors::{
    dotnet::DotNetValidator, java::JavaValidator, make::MakeValidator, node::NodeValidator,
    php::PhpValidator, ruby::RubyValidator, rust::RustValidator, CommandSupport, CommandValidator,
    DetectedRunner,
};

impl DetectedRunner {
    pub fn supports_command(&self, command: &str, working_dir: &Path) -> CommandSupport {
        match self.name.as_str() {
            "npm" | "yarn" | "pnpm" | "bun" => NodeValidator.supports_command(working_dir, command),
            "cargo" => RustValidator.supports_command(working_dir, command),
            "make" => MakeValidator.supports_command(working_dir, command),
            "composer" => PhpValidator.supports_command(working_dir, command),
            "gradle" => JavaValidator.supports_command(working_dir, command),
            "bundler" | "rake" => RubyValidator.supports_command(working_dir, command),
            "dotnet" => DotNetValidator.supports_command(working_dir, command),

            // Runners not yet implementing advanced validation
            "maven" => CommandSupport::Unknown,
            "poetry" | "pipenv" | "uv" | "pip" => CommandSupport::Unknown,
            "go" | "task" => CommandSupport::Unknown,
            "mix" => CommandSupport::Unknown,
            "swift" => CommandSupport::Unknown,
            "zig" => CommandSupport::Unknown,
            _ => CommandSupport::Unknown,
        }
    }
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
    fn test_gradle_task() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("build.gradle")).unwrap();
        writeln!(file, r#"task customTask {{}}"#).unwrap();

        let runner = DetectedRunner::new("gradle", "build.gradle", Ecosystem::Java, 15);
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
