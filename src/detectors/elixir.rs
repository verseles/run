// Copyright (C) 2025 Verseles
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, version 3 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

use super::{DetectedRunner, Ecosystem};
use std::path::Path;

/// Detect Elixir projects (Mix)
/// Priority: 18
pub fn detect(dir: &Path) -> Vec<DetectedRunner> {
    let mut runners = Vec::new();

    let mix_exs = dir.join("mix.exs");

    // mix.exs is sufficient for detection (mix.lock is optional)
    if mix_exs.exists() {
        runners.push(DetectedRunner::new("mix", "mix.exs", Ecosystem::Elixir, 18));
    }

    runners
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_detect_mix_with_lock() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("mix.exs")).unwrap();
        File::create(dir.path().join("mix.lock")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "mix");
    }

    #[test]
    fn test_detect_mix_without_lock() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("mix.exs")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "mix");
    }

    #[test]
    fn test_no_mix() {
        let dir = tempdir().unwrap();

        let runners = detect(dir.path());
        assert!(runners.is_empty());
    }
}
