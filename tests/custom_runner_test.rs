#![allow(deprecated)]

use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[cfg(windows)]
const ECHO_CMD: &str = "cmd /C echo";
#[cfg(not(windows))]
const ECHO_CMD: &str = "echo";

#[test]
fn test_custom_command_in_run_toml() {
    let dir = tempdir().unwrap();
    let run_toml = dir.path().join("run.toml");

    fs::write(
        &run_toml,
        format!(
            r#"
[commands]
hello = "{} hello world"
test = "{} running tests"
"#,
            ECHO_CMD, ECHO_CMD
        ),
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("run").unwrap();
    cmd.current_dir(dir.path())
        .arg("hello")
        .assert()
        .success()
        .stdout(predicates::str::contains("hello world"));

    let mut cmd = Command::cargo_bin("run").unwrap();
    cmd.current_dir(dir.path())
        .arg("test")
        .assert()
        .success()
        .stdout(predicates::str::contains("running tests"));
}

#[test]
fn test_custom_command_override() {
    let dir = tempdir().unwrap();
    let run_toml = dir.path().join("run.toml");
    let package_json = dir.path().join("package.json");

    // Create a package.json that would normally be detected
    fs::write(
        &package_json,
        format!(
            r#"{{ "scripts": {{ "test": "{} npm test" }} }}"#,
            ECHO_CMD
        ),
    )
    .unwrap();

    // Create run.toml that overrides 'test'
    fs::write(
        &run_toml,
        format!(
            r#"
[commands]
test = "{} custom test"
"#,
            ECHO_CMD
        ),
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("run").unwrap();
    cmd.current_dir(dir.path())
        .arg("test")
        .assert()
        .success()
        .stdout(predicates::str::contains("custom test"));
}

#[test]
fn test_complex_command_parsing() {
    let dir = tempdir().unwrap();
    let run_toml = dir.path().join("run.toml");

    // On Windows, single quotes inside double quotes might behave differently with cmd /C echo
    // But for simplicity let's test a simple string first or adjust for platform.

    #[cfg(not(windows))]
    let complex_cmd = "echo 'hello world'";
    #[cfg(windows)]
    let complex_cmd = "cmd /C echo hello world";

    fs::write(
        &run_toml,
        format!(
            r#"
[commands]
complex = "{}"
"#,
            complex_cmd
        ),
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("run").unwrap();
    cmd.current_dir(dir.path())
        .arg("complex")
        .assert()
        .success()
        .stdout(predicates::str::contains("hello world"));
}

#[test]
fn test_empty_custom_command_is_ignored() {
    let dir = tempdir().unwrap();
    let run_toml = dir.path().join("run.toml");
    let package_json = dir.path().join("package.json");

    // Create package.json so detection falls back to npm if custom is ignored
    fs::write(
        &package_json,
        format!(
            r#"{{ "scripts": {{ "test": "{} npm test" }} }}"#,
            ECHO_CMD
        ),
    )
    .unwrap();

    // Custom command is empty, should be filtered out
    fs::write(
        &run_toml,
        r#"
[commands]
test = ""
"#,
    )
    .unwrap();

    // Should detect npm because custom command is empty
    let mut cmd = Command::cargo_bin("run").unwrap();
    cmd.current_dir(dir.path())
        .arg("test")
        .assert()
        .success()
        .stdout(predicates::str::contains("npm test"));
}

#[test]
fn test_custom_overrides_conflict() {
    let dir = tempdir().unwrap();
    let run_toml = dir.path().join("run.toml");

    // Create conflicting lockfiles
    fs::write(dir.path().join("package.json"), "{}").unwrap();
    fs::write(dir.path().join("package-lock.json"), "{}").unwrap();
    fs::write(dir.path().join("yarn.lock"), "").unwrap();

    // Create custom command
    fs::write(
        &run_toml,
        format!(
            r#"
[commands]
test = "{} custom conflict override"
"#,
            ECHO_CMD
        ),
    )
    .unwrap();

    // Should succeed with custom command instead of failing with conflict
    let mut cmd = Command::cargo_bin("run").unwrap();
    cmd.current_dir(dir.path())
        .arg("test")
        .assert()
        .success()
        .stdout(predicates::str::contains("custom conflict override"));
}
