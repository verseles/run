use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("run").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Universal task runner"));
}

#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin("run").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("run 0.1.0"));
}

#[test]
fn test_no_args_shows_help() {
    let mut cmd = Command::cargo_bin("run").unwrap();
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Universal task runner"));
}

#[test]
fn test_unknown_arg_does_not_panic() {
    // It should fail or be treated as part of command if configured?
    // Since we handle arguments after command, clap might catch flags before command.
    let mut cmd = Command::cargo_bin("run").unwrap();
    cmd.arg("--unknown")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
}
