#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::Command;
    use tempfile::tempdir;
    use std::fs::File;
    use predicates::prelude::*;

    #[test]
    fn test_dry_run_npm() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        File::create(path.join("package.json")).unwrap();

        let mut cmd = Command::cargo_bin("run").unwrap();
        cmd.current_dir(path)
            .arg("test")
            .arg("--dry-run")
            .assert()
            .success()
            .stdout(predicate::str::contains("npm run test"));
    }

    #[test]
    fn test_runner_not_found() {
         let dir = tempdir().unwrap();
         // Empty dir

         let mut cmd = Command::cargo_bin("run").unwrap();
         cmd.current_dir(dir.path())
             .arg("test")
             .arg("--levels=0")
             .assert()
             .failure()
             .code(2)
             .stderr(predicate::str::contains("Nenhum runner encontrado"));
    }
}
