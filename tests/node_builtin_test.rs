use std::fs::File;
use std::io::Write;
use tempfile::tempdir;
use run_cli::detectors::{detect_all, CommandSupport};
use run_cli::runner::{select_runner};

#[test]
fn test_npm_install_builtin_works() {
    let dir = tempdir().unwrap();
    let mut file = File::create(dir.path().join("package.json")).unwrap();
    // Add a script so supports_command would normally return NotSupported for "install"
    writeln!(file, r#"{{"scripts": {{"test": "echo test"}}}}"#).unwrap();
    File::create(dir.path().join("package-lock.json")).unwrap();

    let runners = detect_all(dir.path(), &[]);
    assert_eq!(runners.len(), 1);
    let runner = &runners[0];
    assert_eq!(runner.name, "npm");

    // "install" is a built-in command, so supports_command should return BuiltIn
    assert_eq!(runner.supports_command("install", dir.path()), CommandSupport::BuiltIn);

    // select_runner should succeed
    let result = select_runner(&runners, "install", dir.path(), false);
    assert!(result.is_ok());
    let selected = result.unwrap();

    // Verify command construction
    let cmd = selected.build_command("install", &[], dir.path());
    assert_eq!(cmd, vec!["npm", "install"]);
}

#[test]
fn test_npm_test_script_priority() {
    let dir = tempdir().unwrap();
    let mut file = File::create(dir.path().join("package.json")).unwrap();
    // Add a script named "test" - this should take priority over built-in "test"
    writeln!(file, r#"{{"scripts": {{"test": "echo custom test"}}}}"#).unwrap();
    File::create(dir.path().join("package-lock.json")).unwrap();

    let runners = detect_all(dir.path(), &[]);
    let runner = &runners[0];

    // "test" is in scripts, so supports_command should return Supported
    assert_eq!(runner.supports_command("test", dir.path()), CommandSupport::Supported);

    let result = select_runner(&runners, "test", dir.path(), false);
    assert!(result.is_ok());
    let selected = result.unwrap();

    // Verify command construction uses "run"
    let cmd = selected.build_command("test", &[], dir.path());
    assert_eq!(cmd, vec!["npm", "run", "test"]);
}

#[test]
fn test_npm_test_builtin_fallback() {
    let dir = tempdir().unwrap();
    let mut file = File::create(dir.path().join("package.json")).unwrap();
    // No scripts
    writeln!(file, r#"{{"scripts": {{}}}}"#).unwrap();
    File::create(dir.path().join("package-lock.json")).unwrap();

    let runners = detect_all(dir.path(), &[]);
    let runner = &runners[0];

    // "test" is not in scripts but is built-in -> BuiltIn
    assert_eq!(runner.supports_command("test", dir.path()), CommandSupport::BuiltIn);

    let result = select_runner(&runners, "test", dir.path(), false);
    assert!(result.is_ok());
    let selected = result.unwrap();

    // Verify command construction uses direct execution (npm test)
    let cmd = selected.build_command("test", &[], dir.path());
    assert_eq!(cmd, vec!["npm", "test"]);
}
