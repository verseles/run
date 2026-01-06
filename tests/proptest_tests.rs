// Copyright (C) 2025 Verseles
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, version 3 of the License.

//! Property-based tests using proptest
//!
//! These tests verify invariants that should hold for all possible inputs,
//! not just specific examples.

use proptest::prelude::*;

// ============================================================================
// Semver validation tests
// ============================================================================

/// Strategy for generating valid semver versions
fn valid_semver_strategy() -> impl Strategy<Value = String> {
    // Generate major.minor.patch with reasonable bounds
    (0u32..100, 0u32..100, 0u32..100)
        .prop_map(|(major, minor, patch)| format!("{}.{}.{}", major, minor, patch))
}

/// Strategy for generating semver with prerelease
fn semver_with_prerelease_strategy() -> impl Strategy<Value = String> {
    (
        0u32..100,
        0u32..100,
        0u32..100,
        prop::sample::select(vec!["alpha", "beta", "rc"]),
        0u32..20,
    )
        .prop_map(|(major, minor, patch, pre, pre_num)| {
            format!("{}.{}.{}-{}.{}", major, minor, patch, pre, pre_num)
        })
}

proptest! {
    /// All valid semver strings should be parseable
    #[test]
    fn semver_valid_versions_parse(version in valid_semver_strategy()) {
        let parsed = semver::Version::parse(&version);
        prop_assert!(parsed.is_ok(), "Failed to parse: {}", version);
    }

    /// Semver with prerelease should also parse
    #[test]
    fn semver_prerelease_versions_parse(version in semver_with_prerelease_strategy()) {
        let parsed = semver::Version::parse(&version);
        prop_assert!(parsed.is_ok(), "Failed to parse: {}", version);
    }

    /// Version comparison: same version equals itself
    #[test]
    fn semver_version_equals_itself(version in valid_semver_strategy()) {
        let v1 = semver::Version::parse(&version).unwrap();
        let v2 = semver::Version::parse(&version).unwrap();
        prop_assert_eq!(v1, v2);
    }

    /// Version comparison: higher major is always greater
    #[test]
    fn semver_higher_major_is_greater(
        major1 in 0u32..50,
        major2 in 51u32..100,
        minor in 0u32..100,
        patch in 0u32..100
    ) {
        let v1 = semver::Version::new(major1.into(), minor.into(), patch.into());
        let v2 = semver::Version::new(major2.into(), minor.into(), patch.into());
        prop_assert!(v2 > v1);
    }

    /// Version comparison: higher minor is greater when major is same
    #[test]
    fn semver_higher_minor_is_greater(
        major in 0u32..100,
        minor1 in 0u32..50,
        minor2 in 51u32..100,
        patch in 0u32..100
    ) {
        let v1 = semver::Version::new(major.into(), minor1.into(), patch.into());
        let v2 = semver::Version::new(major.into(), minor2.into(), patch.into());
        prop_assert!(v2 > v1);
    }

    /// Version comparison: higher patch is greater when major.minor is same
    #[test]
    fn semver_higher_patch_is_greater(
        major in 0u32..100,
        minor in 0u32..100,
        patch1 in 0u32..50,
        patch2 in 51u32..100
    ) {
        let v1 = semver::Version::new(major.into(), minor.into(), patch1.into());
        let v2 = semver::Version::new(major.into(), minor.into(), patch2.into());
        prop_assert!(v2 > v1);
    }

    /// Version with 'v' prefix should be trimmable and parseable
    #[test]
    fn semver_v_prefix_trimmable(version in valid_semver_strategy()) {
        let with_v = format!("v{}", version);
        let trimmed = with_v.trim_start_matches('v');
        let parsed = semver::Version::parse(trimmed);
        prop_assert!(parsed.is_ok(), "Failed to parse trimmed: {}", trimmed);
    }
}

// ============================================================================
// Go path detection tests
// ============================================================================

/// Strategy for Go paths (should trigger `go run`)
fn go_path_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Paths with slashes
        "[a-z]{1,10}/[a-z]{1,10}\\.go".prop_map(|s| s),
        // Simple .go files
        "[a-z]{1,10}\\.go".prop_map(|s| s),
        // Deep paths
        "[a-z]{1,5}/[a-z]{1,5}/[a-z]{1,5}\\.go".prop_map(|s| s),
        // Paths starting with ./
        "./[a-z]{1,10}\\.go".prop_map(|s| s),
        // Paths starting with ../
        "../[a-z]{1,10}\\.go".prop_map(|s| s),
    ]
}

/// Strategy for Go tasks (should NOT trigger `go run`)
fn go_task_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("build".to_string()),
        Just("test".to_string()),
        Just("run".to_string()),
        Just("fmt".to_string()),
        Just("vet".to_string()),
        Just("mod".to_string()),
        Just("generate".to_string()),
        Just("install".to_string()),
        // Random alphanumeric that doesn't contain / or end in .go
        "[a-z]{3,10}".prop_filter("must not end in .go", |s| !s.ends_with(".go")),
    ]
}

/// Helper function that mirrors the Go runner logic
fn is_go_path(task: &str) -> bool {
    task.contains('/') || task.ends_with(".go")
}

proptest! {
    /// Go paths should be detected as paths
    #[test]
    fn go_paths_are_detected(path in go_path_strategy()) {
        prop_assert!(
            is_go_path(&path),
            "Path not detected: {}",
            path
        );
    }

    /// Go tasks should NOT be detected as paths
    #[test]
    fn go_tasks_are_not_paths(task in go_task_strategy()) {
        prop_assert!(
            !is_go_path(&task),
            "Task incorrectly detected as path: {}",
            task
        );
    }
}

// ============================================================================
// Case-insensitive matching tests (for --ignore flag)
// ============================================================================

/// Strategy for runner names
fn runner_name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("npm"),
        Just("yarn"),
        Just("pnpm"),
        Just("bun"),
        Just("cargo"),
        Just("poetry"),
        Just("uv"),
        Just("composer"),
        Just("gradle"),
        Just("maven"),
        Just("mix"),
        Just("make"),
    ]
    .prop_map(|s| s.to_string())
}

/// Helper function that mirrors CLI ignore logic
fn should_ignore(ignore_list: &[String], runner: &str) -> bool {
    ignore_list.iter().any(|i| i.eq_ignore_ascii_case(runner))
}

proptest! {
    /// Case-insensitive matching: lowercase matches uppercase
    #[test]
    fn ignore_case_insensitive_lower_to_upper(name in runner_name_strategy()) {
        let upper = name.to_uppercase();
        let ignore_list = vec![upper];
        prop_assert!(
            should_ignore(&ignore_list, &name),
            "Failed to match {} with uppercase version",
            name
        );
    }

    /// Case-insensitive matching: uppercase matches lowercase
    #[test]
    fn ignore_case_insensitive_upper_to_lower(name in runner_name_strategy()) {
        let lower = name.to_lowercase();
        let ignore_list = vec![lower.clone()];
        prop_assert!(
            should_ignore(&ignore_list, &name.to_uppercase()),
            "Failed to match {} with lowercase version",
            name
        );
    }

    /// Case-insensitive matching: mixed case
    #[test]
    fn ignore_case_insensitive_mixed(name in runner_name_strategy()) {
        // Create mixed case version
        let mixed: String = name
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i % 2 == 0 {
                    c.to_uppercase().next().unwrap()
                } else {
                    c.to_lowercase().next().unwrap()
                }
            })
            .collect();

        let ignore_list = vec![mixed.clone()];
        prop_assert!(
            should_ignore(&ignore_list, &name),
            "Failed to match {} with mixed case {}",
            name,
            mixed
        );
    }

    /// Non-matching names should not be ignored
    #[test]
    fn non_matching_not_ignored(name in runner_name_strategy()) {
        let other = if name == "npm" { "yarn" } else { "npm" };
        let ignore_list = vec![name.clone()];
        prop_assert!(
            !should_ignore(&ignore_list, other),
            "Incorrectly matched {} with {}",
            name,
            other
        );
    }
}

// ============================================================================
// Priority ordering tests
// ============================================================================

proptest! {
    /// Sorting by priority should always put lower numbers first
    #[test]
    fn priority_sorting_is_stable(
        p1 in 0u8..50,
        p2 in 51u8..100,
        p3 in 101u8..150,
        p4 in 151u8..200
    ) {
        let mut priorities = [p3, p1, p4, p2];
        priorities.sort();

        prop_assert!(priorities[0] == p1);
        prop_assert!(priorities[1] == p2);
        prop_assert!(priorities[2] == p3);
        prop_assert!(priorities[3] == p4);
    }
}

// ============================================================================
// Path normalization tests
// ============================================================================

/// Strategy for directory names
fn dirname_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_-]{0,20}".prop_map(|s| s)
}

proptest! {
    /// Directory paths with trailing slash should work
    #[test]
    fn path_trailing_slash_handling(dir in dirname_strategy()) {
        let with_slash = format!("{}/", dir);
        let without_slash = dir.clone();

        // Both should be valid path components
        let p1 = std::path::Path::new(&with_slash);
        let p2 = std::path::Path::new(&without_slash);

        prop_assert!(p1.components().count() >= 1);
        prop_assert!(p2.components().count() >= 1);
    }

    /// Nested paths should have correct component counts
    #[test]
    fn nested_path_component_count(
        d1 in dirname_strategy(),
        d2 in dirname_strategy(),
        d3 in dirname_strategy()
    ) {
        let path = format!("{}/{}/{}", d1, d2, d3);
        let p = std::path::Path::new(&path);
        prop_assert_eq!(p.components().count(), 3);
    }
}
