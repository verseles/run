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

use owo_colors::OwoColorize;
use std::env;

/// Check if colors should be disabled
pub fn colors_disabled() -> bool {
    env::var("NO_COLOR").is_ok()
}

/// Print a success message
pub fn success(message: &str) {
    if colors_disabled() {
        eprintln!("✓ {}", message);
    } else {
        eprintln!("{} {}", "✓".green(), message.green());
    }
}

/// Print a warning message
pub fn warning(message: &str) {
    if colors_disabled() {
        eprintln!("⚠ {}", message);
    } else {
        eprintln!("{} {}", "⚠".yellow(), message.yellow());
    }
}

/// Print an error message
pub fn error(message: &str) {
    if colors_disabled() {
        eprintln!("❌ {}", message);
    } else {
        eprintln!("{} {}", "❌".red(), message.red());
    }
}

/// Print an info message (for verbose mode)
pub fn info(message: &str) {
    if colors_disabled() {
        eprintln!("🔍 {}", message);
    } else {
        eprintln!("{} {}", "🔍".cyan(), message.cyan());
    }
}

/// Print a detection message (for verbose mode)
pub fn detected(runner: &str, file: &str) {
    if colors_disabled() {
        eprintln!("📦 Detected: {} ({})", runner, file);
    } else {
        eprintln!(
            "{} Detected: {} ({})",
            "📦".blue(),
            runner.blue().bold(),
            file.blue()
        );
    }
}

/// Print a command execution message
pub fn executing(command: &str) {
    if colors_disabled() {
        eprintln!("✓ Executing: {}", command);
    } else {
        eprintln!("{} Executing: {}", "✓".green(), command.green());
    }
}

/// Print an update notification
pub fn update_notification(from_version: &str, to_version: &str, changelog: Option<&str>) {
    if colors_disabled() {
        eprintln!("⬆ run was updated: {} → {}", from_version, to_version);
    } else {
        eprintln!(
            "{} {} was updated: {} → {}",
            "⬆".green(),
            "run".green().bold(),
            from_version.yellow(),
            to_version.green()
        );
    }

    if let Some(changes) = changelog {
        eprintln!();
        eprintln!("Main changes:");
        for line in changes.lines().take(5) {
            eprintln!("  {}", line);
        }
    }
}
