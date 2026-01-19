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

//! # run-cli
//!
//! Universal task runner for modern development.
//!
//! Automatically detects the project's package manager or build tool
//! and runs commands through the appropriate tool.

pub mod cli;
pub mod config;
pub mod detectors;
pub mod error;
pub mod output;
pub mod runner;
pub mod update;

pub use cli::Cli;
pub use config::Config;
pub use detectors::DetectedRunner;
pub use error::RunError;
