use anyhow::Result;
use std::path::Path;
use crate::detectors::{Detector, Detection};

pub struct GoDetector;

impl Detector for GoDetector {
    fn detect(&self, path: &Path) -> Result<Option<Detection>> {
        // 11. Taskfile: Taskfile.yml OR Taskfile.yaml -> task
        if path.join("Taskfile.yml").exists() || path.join("Taskfile.yaml").exists() {
             return Ok(Some(Detection {
                runner: "task".to_string(),
                command: "task".to_string(),
                lockfile: None, // Taskfile is config, not exactly lockfile but can be treated as presence
            }));
        }

        // 12. Go Modules: go.mod + go.sum -> go run OR go
        if path.join("go.mod").exists() {
            let lockfile = if path.join("go.sum").exists() {
                Some("go.sum".to_string())
            } else {
                None
            };

            // Logic: "go run <comando>" (if command looks like path) OR "go <comando>"
            // This logic depends on the command passed by user. But Detection struct has a static command prefix.
            // The plan says: "go run <comando> (se comando parecer caminho) OU go <comando>"
            // We can't decide here easily without knowing the command.
            // But usually `go run` takes a file. `go test` takes arguments.
            // If the user types `run test`, and we return `go`, it executes `go test`. Correct.
            // If user types `run main.go`, we return `go`, it executes `go main.go`. Incorrect if it needs `run`.
            // But `go run main.go` works.
            // If the user types `run build`, `go build`.

            // Wait, if I return `go`, and user types `main.go`, `go main.go` is invalid.
            // If I return `go run`, and user types `test`, `go run test` is invalid.

            // The plan implies intelligence or just `go`.
            // "go run <comando> (se comando parecer caminho) OU go <comando>"

            // If the user wants to run a script, usually `go run` is for running a file.
            // Most tasks in Go are `go test`, `go build`, `go fmt`.
            // So `go` is the safer default prefix.
            // If user wants `go run`, they might type `run run main.go` -> `go run main.go`.

            return Ok(Some(Detection {
                runner: "go".to_string(),
                command: "go".to_string(),
                lockfile,
            }));
        }

        Ok(None)
    }
}
