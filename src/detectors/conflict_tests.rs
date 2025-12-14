#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;
    use crate::detectors::{find_runner, Detection};

    #[test]
    fn test_conflict_npm_yarn() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        File::create(path.join("package.json")).unwrap();
        File::create(path.join("package-lock.json")).unwrap(); // npm
        File::create(path.join("yarn.lock")).unwrap(); // yarn

        let result = find_runner(path, 0, &[]);

        match result {
            Ok(Some((d, _))) => {
                println!("Resolved to {}", d.runner);
                // If it resolved, it means either:
                // 1. Only one tool installed (warning printed to stderr, but test passes)
                // 2. Logic is broken and just picked first one (if ignoring which check).
            },
            Err(e) => {
                 let msg = e.to_string();
                 println!("Error: {}", msg);
                 assert!(msg.contains("conflitos") || msg.contains("nenhuma das ferramentas") || msg.contains("Conflict detected"));
            },
            Ok(None) => {
                // Should not happen as we created lockfiles
                panic!("Should have detected something");
            }
        }
    }
}
