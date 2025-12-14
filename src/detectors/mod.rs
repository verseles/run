pub mod node;
pub mod rust;
pub mod python;
pub mod go;
pub mod ruby;
pub mod java;
pub mod php;
pub mod dotnet;
pub mod elixir;
pub mod swift;
pub mod zig;
pub mod make;

use anyhow::Result;
use std::path::{Path, PathBuf};

pub trait Detector {
    fn detect(&self, path: &Path) -> Result<Option<Detection>>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Detection {
    pub runner: String,
    pub command: String,
    pub lockfile: Option<String>,
}

pub fn find_runner(
    start_path: &Path,
    max_levels: usize,
    ignore: &[String],
) -> Result<Option<(Detection, PathBuf)>> {
    let mut current_path = start_path.to_path_buf();
    let mut levels_checked = 0;

    let detectors: Vec<Box<dyn Detector>> = vec![
        Box::new(node::NodeDetector),
        Box::new(python::PythonDetector),
        Box::new(rust::RustDetector),
        Box::new(php::PhpDetector),
        Box::new(go::GoDetector),
        Box::new(ruby::RubyDetector),
        Box::new(java::JavaDetector),
        Box::new(dotnet::DotNetDetector),
        Box::new(elixir::ElixirDetector),
        Box::new(swift::SwiftDetector),
        Box::new(zig::ZigDetector),
        Box::new(make::MakeDetector),
    ];

    loop {
        // Run detectors
        for detector in &detectors {
             if let Some(detection) = detector.detect(&current_path)? {
                 // Check ignore list
                 if ignore.contains(&detection.runner) {
                     continue;
                 }
                 // Found!
                 // In case of conflict (Step 7), we need to check ALL detectors and resolve.
                 // The plan says: "Quando múltiplos lockfiles do **mesmo ecossistema** forem encontrados"
                 // My implementation in `node.rs` and `python.rs` checks precedence internally (order of if statements).
                 // `node.rs` checks Bun > PNPM > Yarn > NPM.
                 // `python.rs` checks UV > Poetry > Pipenv > Pip.
                 // So "Same ecosystem" conflict is handled by the detector itself returning the first match.

                 // BUT, the plan says: "Verificar quais ferramentas correspondentes estão instaladas globalmente usando which"
                 // And "Se apenas uma ferramenta estiver instalada: usar essa... Se ambas... parar com erro".

                 // So I need to modify `node.rs` (and others) to return MULTIPLE candidates if multiple lockfiles are present?
                 // Or handle it inside `node.rs`.

                 // The best place is inside `node.rs`.
                 // I will return to `node.rs` in Step 7 to implement this conflict logic.
                 // For now (Step 6), simple precedence is enough or I can refactor now.

                 // Step 7 is specifically "Lockfile Conflict Resolution". I will do it there.

                 return Ok(Some((detection, current_path)));
             }
        }

        if levels_checked >= max_levels {
            break;
        }

        if !current_path.pop() {
            break; // Reached root
        }
        levels_checked += 1;
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_detect_node_npm() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        File::create(path.join("package.json")).unwrap();

        let (detection, detected_path) = find_runner(path, 0, &[]).unwrap().unwrap();
        assert_eq!(detection.runner, "npm");
        assert_eq!(detected_path, path);
    }

    #[test]
    fn test_detect_node_yarn() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        File::create(path.join("package.json")).unwrap();
        File::create(path.join("yarn.lock")).unwrap();

        let (detection, _detected_path) = find_runner(path, 0, &[]).unwrap().unwrap();
        assert_eq!(detection.runner, "yarn");
    }

    #[test]
    fn test_detect_rust_cargo() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        File::create(path.join("Cargo.toml")).unwrap();

        let (detection, _detected_path) = find_runner(path, 0, &[]).unwrap().unwrap();
        assert_eq!(detection.runner, "cargo");
    }

    #[test]
    fn test_recursive_search() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        File::create(root.join("package.json")).unwrap();

        let subdir = root.join("src/nested");
        std::fs::create_dir_all(&subdir).unwrap();

        let (detection, detected_path) = find_runner(&subdir, 3, &[]).unwrap().unwrap();
        assert_eq!(detection.runner, "npm");
        assert_eq!(detected_path, root);
    }

    #[test]
    fn test_ignore() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        File::create(path.join("package.json")).unwrap();

        let result = find_runner(path, 0, &["npm".to_string()]).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_python_poetry() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        File::create(path.join("pyproject.toml")).unwrap();
        File::create(path.join("poetry.lock")).unwrap();

        let (detection, _) = find_runner(path, 0, &[]).unwrap().unwrap();
        assert_eq!(detection.runner, "poetry");
    }

    #[test]
    fn test_make() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        File::create(path.join("Makefile")).unwrap();

        let (detection, _) = find_runner(path, 0, &[]).unwrap().unwrap();
        assert_eq!(detection.runner, "make");
    }
}
mod conflict_tests;
