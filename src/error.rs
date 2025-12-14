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

use thiserror::Error;

/// Exit codes for the CLI
pub mod exit_codes {
    pub const SUCCESS: i32 = 0;
    pub const GENERIC_ERROR: i32 = 1;
    pub const RUNNER_NOT_FOUND: i32 = 2;
    pub const LOCKFILE_CONFLICT: i32 = 3;
    pub const TOOL_NOT_INSTALLED: i32 = 127;
}

#[derive(Error, Debug)]
pub enum RunError {
    #[error("No runner found in {0} levels above the current directory")]
    RunnerNotFound(u8),

    #[error("Lockfile conflict detected: {0}")]
    LockfileConflict(String),

    #[error("Tool not installed: {0}")]
    ToolNotInstalled(String),

    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
}

impl RunError {
    pub fn exit_code(&self) -> i32 {
        match self {
            RunError::RunnerNotFound(_) => exit_codes::RUNNER_NOT_FOUND,
            RunError::LockfileConflict(_) => exit_codes::LOCKFILE_CONFLICT,
            RunError::ToolNotInstalled(_) => exit_codes::TOOL_NOT_INSTALLED,
            _ => exit_codes::GENERIC_ERROR,
        }
    }
}
