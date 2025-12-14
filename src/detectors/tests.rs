#[cfg(test)]
mod tests {

    use crate::detectors::find_runner;
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

        let (detection, detected_path) = find_runner(path, 0, &[]).unwrap().unwrap();
        assert_eq!(detection.runner, "yarn");
    }

    #[test]
    fn test_detect_rust_cargo() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        File::create(path.join("Cargo.toml")).unwrap();

        let (detection, detected_path) = find_runner(path, 0, &[]).unwrap().unwrap();
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
}
