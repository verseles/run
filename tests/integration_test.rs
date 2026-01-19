// Copyright (C) 2025 Verseles
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, version 3 of the License.

#![allow(deprecated)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

fn run_cmd() -> Command {
    Command::cargo_bin("run").unwrap()
}

// ============================================================================
// Basic CLI tests
// ============================================================================

#[test]
fn test_help() {
    run_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Universal task runner"));
}

#[test]
fn test_version() {
    run_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_no_command_shows_help() {
    run_cmd()
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn test_no_runner_found() {
    let dir = tempdir().unwrap();

    run_cmd()
        .current_dir(dir.path())
        .arg("test")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No runner found"));
}

#[test]
fn test_no_runner_found_exit_code() {
    let dir = tempdir().unwrap();

    run_cmd()
        .current_dir(dir.path())
        .arg("test")
        .assert()
        .code(2);
}

// ============================================================================
// Node.js ecosystem detection
// ============================================================================

#[test]
fn test_dry_run_npm() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("package.json")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("npm run test"));
}

#[test]
fn test_dry_run_npm_with_lockfile() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("package.json")).unwrap();
    File::create(dir.path().join("package-lock.json")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("npm run test"));
}

#[test]
fn test_dry_run_pnpm() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("package.json")).unwrap();
    File::create(dir.path().join("pnpm-lock.yaml")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("pnpm run test"));
}

#[test]
fn test_dry_run_yarn() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("package.json")).unwrap();
    File::create(dir.path().join("yarn.lock")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("yarn run test"));
}

#[test]
fn test_dry_run_bun_lockb() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("package.json")).unwrap();
    File::create(dir.path().join("bun.lockb")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("bun run test"));
}

#[test]
fn test_dry_run_bun_lock() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("package.json")).unwrap();
    File::create(dir.path().join("bun.lock")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("bun run test"));
}

// ============================================================================
// Python ecosystem detection
// ============================================================================

#[test]
fn test_dry_run_poetry() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("pyproject.toml")).unwrap();
    File::create(dir.path().join("poetry.lock")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("poetry run test"));
}

#[test]
fn test_dry_run_uv() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("pyproject.toml")).unwrap();
    File::create(dir.path().join("uv.lock")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("uv run test"));
}

#[test]
fn test_dry_run_pipenv() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("Pipfile")).unwrap();
    File::create(dir.path().join("Pipfile.lock")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("pipenv run test"));
}

#[test]
fn test_dry_run_pip() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("requirements.txt")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["pytest", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("python -m pytest"));
}

// ============================================================================
// Rust ecosystem detection
// ============================================================================

#[test]
fn test_dry_run_cargo() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("Cargo.toml")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["build", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("cargo build"));
}

#[test]
fn test_dry_run_cargo_test() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("Cargo.toml")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("cargo test"));
}

// ============================================================================
// PHP ecosystem detection
// ============================================================================

#[test]
fn test_dry_run_composer() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("composer.json")).unwrap();
    File::create(dir.path().join("composer.lock")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("composer run test"));
}

// ============================================================================
// Go ecosystem detection
// ============================================================================

#[test]
fn test_dry_run_task() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("Taskfile.yml")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["build", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("task build"));
}

#[test]
fn test_dry_run_task_yaml() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("Taskfile.yaml")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["build", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("task build"));
}

#[test]
fn test_dry_run_go() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("go.mod")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["build", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("go build"));
}

#[test]
fn test_dry_run_go_run_path() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("go.mod")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["./cmd/main.go", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("go run ./cmd/main.go"));
}

// ============================================================================
// Just command runner detection
// ============================================================================

#[test]
fn test_dry_run_just() {
    let dir = tempdir().unwrap();
    let mut file = File::create(dir.path().join("justfile")).unwrap();
    writeln!(file, "build:\n    cargo build").unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["build", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("just build"));
}

#[test]
fn test_dry_run_just_capitalized() {
    let dir = tempdir().unwrap();
    let mut file = File::create(dir.path().join("Justfile")).unwrap();
    writeln!(file, "test:\n    cargo test").unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("just test"));
}

// ============================================================================
// Ruby ecosystem detection
// ============================================================================

#[test]
fn test_dry_run_bundler() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("Gemfile")).unwrap();
    File::create(dir.path().join("Gemfile.lock")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["rspec", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("bundle exec rspec"));
}

#[test]
fn test_dry_run_rake() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("Rakefile")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("rake test"));
}

// ============================================================================
// Java ecosystem detection
// ============================================================================

#[test]
fn test_dry_run_gradle() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("build.gradle")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["build", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("gradle build"));
}

#[test]
fn test_dry_run_gradle_kts() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("build.gradle.kts")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["build", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("gradle build"));
}

#[test]
fn test_dry_run_maven() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("pom.xml")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["compile", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("mvn compile"));
}

// ============================================================================
// .NET ecosystem detection
// ============================================================================

#[test]
fn test_dry_run_dotnet_csproj() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("Test.csproj")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["build", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dotnet build"));
}

#[test]
fn test_dry_run_dotnet_sln() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("Test.sln")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["build", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dotnet build"));
}

// ============================================================================
// Elixir ecosystem detection
// ============================================================================

#[test]
fn test_dry_run_mix() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("mix.exs")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("mix test"));
}

// ============================================================================
// Swift ecosystem detection
// ============================================================================

#[test]
fn test_dry_run_swift() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("Package.swift")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["MyApp", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("swift run MyApp"));
}

// ============================================================================
// Zig ecosystem detection
// ============================================================================

#[test]
fn test_dry_run_zig() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("build.zig")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("zig build test"));
}

// ============================================================================
// Make detection
// ============================================================================

#[test]
fn test_dry_run_makefile() {
    let dir = tempdir().unwrap();
    let mut file = File::create(dir.path().join("Makefile")).unwrap();
    writeln!(file, "build:\n\t@echo building").unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["build", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("make build"));
}

#[test]
fn test_dry_run_makefile_lowercase() {
    let dir = tempdir().unwrap();
    let mut file = File::create(dir.path().join("makefile")).unwrap();
    writeln!(file, "build:\n\t@echo building").unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["build", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("make build"));
}

// ============================================================================
// CLI flag tests
// ============================================================================

#[test]
fn test_ignore_flag() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("package.json")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--ignore=npm", "--dry-run"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No runner found"));
}

#[test]
fn test_ignore_comma_separated() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("package.json")).unwrap();
    File::create(dir.path().join("Cargo.toml")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--ignore=npm,cargo", "--dry-run"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No runner found"));
}

#[test]
fn test_ignore_multiple_flags() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("package.json")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--ignore", "npm", "--ignore", "yarn", "--dry-run"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No runner found"));
}

#[test]
fn test_ignore_case_insensitive() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("package.json")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--ignore=NPM", "--dry-run"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No runner found"));
}

// ============================================================================
// Recursive search tests
// ============================================================================

#[test]
fn test_recursive_search() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("package.json")).unwrap();

    let subdir = dir.path().join("src").join("components");
    fs::create_dir_all(&subdir).unwrap();

    run_cmd()
        .current_dir(&subdir)
        .args(["test", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("npm run test"));
}

#[test]
fn test_recursive_search_3_levels() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("package.json")).unwrap();

    let subdir = dir.path().join("a").join("b").join("c");
    fs::create_dir_all(&subdir).unwrap();

    run_cmd()
        .current_dir(&subdir)
        .args(["test", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("npm run test"));
}

#[test]
fn test_levels_limit_zero() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("package.json")).unwrap();

    let subdir = dir.path().join("src");
    fs::create_dir_all(&subdir).unwrap();

    run_cmd()
        .current_dir(&subdir)
        .args(["test", "--levels=0"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No runner found"));
}

#[test]
fn test_levels_limit_exceeded() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("package.json")).unwrap();

    let deep_subdir = dir.path().join("a").join("b").join("c").join("d");
    fs::create_dir_all(&deep_subdir).unwrap();

    run_cmd()
        .current_dir(&deep_subdir)
        .args(["test", "--levels=3"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No runner found"));
}

#[test]
fn test_levels_limit_sufficient() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("package.json")).unwrap();

    let deep_subdir = dir.path().join("a").join("b").join("c").join("d");
    fs::create_dir_all(&deep_subdir).unwrap();

    run_cmd()
        .current_dir(&deep_subdir)
        .args(["test", "--levels=5", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("npm run test"));
}

// ============================================================================
// Extra arguments tests
// ============================================================================

#[test]
fn test_extra_args() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("package.json")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--dry-run", "--", "--coverage", "--verbose"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "npm run test --coverage --verbose",
        ));
}

#[test]
fn test_extra_args_with_equals() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("package.json")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--dry-run", "--", "--reporter=json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("npm run test --reporter=json"));
}

// ============================================================================
// Verbose and quiet mode tests
// ============================================================================

#[test]
fn test_verbose_mode() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("package.json")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--dry-run", "--verbose"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Detected"));
}

#[test]
fn test_quiet_mode_suppresses_executing() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("package.json")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--dry-run", "--quiet"])
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

// ============================================================================
// Shell completions tests
// ============================================================================

#[test]
fn test_completions_bash() {
    run_cmd()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_run"));
}

#[test]
fn test_completions_zsh() {
    run_cmd()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef run"));
}

#[test]
fn test_completions_fish() {
    run_cmd()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"));
}

#[test]
fn test_completions_powershell() {
    run_cmd()
        .args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Register-ArgumentCompleter"));
}

// ============================================================================
// Cross-ecosystem priority tests
// ============================================================================

#[test]
fn test_priority_nodejs_over_make() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("package.json")).unwrap();
    File::create(dir.path().join("Makefile")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("npm run test"));
}

#[test]
fn test_priority_cargo_over_make() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("Cargo.toml")).unwrap();
    File::create(dir.path().join("Makefile")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["build", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("cargo build"));
}

#[test]
fn test_priority_python_over_make() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("requirements.txt")).unwrap();
    File::create(dir.path().join("Makefile")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["pytest", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("python -m pytest"));
}

#[test]
fn test_priority_mix_over_make() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("mix.exs")).unwrap();
    File::create(dir.path().join("Makefile")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("mix test"));
}

// ============================================================================
// Config file tests
// ============================================================================

#[test]
fn test_local_config_ignore() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("package.json")).unwrap();

    let mut config = File::create(dir.path().join("run.toml")).unwrap();
    writeln!(config, "ignore_tools = [\"npm\"]").unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["test", "--dry-run"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No runner found"));
}

// ============================================================================
// Command validation / fallback tests
// ============================================================================

#[test]
fn test_fallback_cargo_to_make() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("Cargo.toml")).unwrap();
    let mut makefile = File::create(dir.path().join("Makefile")).unwrap();
    writeln!(makefile, "precommit:\n\t@echo precommit").unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["precommit", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("make precommit"));
}

#[test]
fn test_fallback_npm_to_make() {
    let dir = tempdir().unwrap();
    let mut pkg = File::create(dir.path().join("package.json")).unwrap();
    writeln!(pkg, r#"{{"name": "test"}}"#).unwrap();
    let mut makefile = File::create(dir.path().join("Makefile")).unwrap();
    writeln!(makefile, "deploy:\n\t@echo deploying").unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["deploy", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("make deploy"));
}

#[test]
fn test_npm_script_takes_priority() {
    let dir = tempdir().unwrap();
    let mut pkg = File::create(dir.path().join("package.json")).unwrap();
    writeln!(pkg, r#"{{"scripts": {{"build": "tsc"}}}}"#).unwrap();
    let mut makefile = File::create(dir.path().join("Makefile")).unwrap();
    writeln!(makefile, "build:\n\t@echo make build").unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["build", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("npm run build"));
}

#[test]
fn test_command_not_supported_error() {
    let dir = tempdir().unwrap();
    File::create(dir.path().join("Cargo.toml")).unwrap();

    run_cmd()
        .current_dir(dir.path())
        .args(["nonexistent-command", "--dry-run"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not supported"));
}
