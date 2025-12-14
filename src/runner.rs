use anyhow::{Context, Result};
use std::process::Command;
use crate::detectors::Detection;

pub fn execute_command(detection: &Detection, args: &[String]) -> Result<i32> {
    let parts: Vec<&str> = detection.command.split_whitespace().collect();
    let (program, cmd_args) = parts.split_first().context("Invalid command")?;

    let mut command = Command::new(program);
    command.args(cmd_args);
    command.args(args);

    // Pass through stdio
    // In real implementation we might want to capture if needed, but for now we inherit
    // to let the user interact with the process (e.g. interactive prompts)
    // and see output in real time.
    // The plan says: "stdout/stderr/exit code conectados ao terminal"

    // However, if we need to do something AFTER, we still just wait.
    // Spawn creates a child.

    // Handle signals? Rust's std::process handles SIGINT by default (terminating),
    // but if we want to forward signals we need something like `ctrlc` crate or `tokio::signal`.
    // The plan didn't explicitly ask for advanced signal forwarding but implied "Delegate execution".
    // Usually `Command::status()` is enough as it waits for the child.
    // If the child receives SIGINT (Ctrl+C), it usually terminates, and the parent (us) also gets it.
    // But we want to ensure we return the exit code correctly.

    let status = command.status().context("Failed to execute command")?;

    Ok(status.code().unwrap_or(1)) // 1 if terminated by signal
}
