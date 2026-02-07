#![allow(deprecated)]

use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_custom_command_in_run_toml() {
    let dir = tempdir().unwrap();
    let run_toml = dir.path().join("run.toml");

    fs::write(
        &run_toml,
        r#"
[commands]
hello = "echo hello world"
test = "echo running tests"
"#,
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
        r#"{ "scripts": { "test": "echo npm test" } }"#,
    )
    .unwrap();

    // Create run.toml that overrides 'test'
    fs::write(
        &run_toml,
        r#"
[commands]
test = "echo custom test"
"#,
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

    fs::write(
        &run_toml,
        r#"
[commands]
complex = "echo 'hello world'"
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("run").unwrap();
    cmd.current_dir(dir.path())
        .arg("complex")
        .assert()
        .success()
        .stdout(predicates::str::contains("hello world"));
}
