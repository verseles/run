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

use clap::{Parser, Subcommand};

/// Universal task runner - automatically detects and runs project commands
#[derive(Parser, Debug, Clone)]
#[command(name = "run")]
#[command(author = "Verseles")]
#[command(version)]
#[command(about = "Universal task runner for modern development", long_about = None)]
#[command(after_help = "SUPPORTED RUNNERS:
  Node.js:  bun, pnpm, yarn, npm
  Python:   uv, poetry, pipenv, pip
  Rust:     cargo
  PHP:      composer
  Go:       task, go
  Ruby:     bundler, rake
  Java:     gradle, maven
  .NET:     dotnet
  Elixir:   mix
  Swift:    swift
  Zig:      zig
  Generic:  make

EXAMPLES:
  run test                      # Run test command using detected runner
  run build -- --verbose        # Pass extra arguments after --
  run lint --levels=5           # Search up to 5 levels above current dir
  run start --ignore=npm,yarn   # Skip specific runners
  run deploy --dry-run          # Show command without executing")]
pub struct Cli {
    /// Command to run (e.g., test, build, start)
    #[arg(value_name = "COMMAND")]
    pub command: Option<String>,

    /// Arguments to pass to the command
    #[arg(value_name = "ARGS", trailing_var_arg = true)]
    pub args: Vec<String>,

    /// How many directory levels to search above current dir
    #[arg(short, long, default_value = "3", value_parser = clap::value_parser!(u8).range(0..=10))]
    pub levels: u8,

    /// Runners to ignore (comma-separated or multiple flags)
    #[arg(short, long = "ignore", value_delimiter = ',')]
    pub ignore: Vec<String>,

    /// Show detailed detection information
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppress all output except errors and command output
    #[arg(short, long)]
    pub quiet: bool,

    /// Show command without executing
    #[arg(long)]
    pub dry_run: bool,

    /// Force immediate update check
    #[arg(long)]
    pub update: bool,

    #[command(subcommand)]
    pub subcommand: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

impl Cli {
    /// Check if a runner should be ignored
    pub fn should_ignore(&self, runner: &str) -> bool {
        self.ignore.iter().any(|i| i.eq_ignore_ascii_case(runner))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }

    #[test]
    fn test_basic_command() {
        let cli = Cli::parse_from(["run", "test"]);
        assert_eq!(cli.command, Some("test".to_string()));
        assert!(cli.args.is_empty());
    }

    #[test]
    fn test_command_with_args() {
        let cli = Cli::parse_from(["run", "test", "--", "--coverage", "--verbose"]);
        assert_eq!(cli.command, Some("test".to_string()));
        assert_eq!(cli.args, vec!["--coverage", "--verbose"]);
    }

    #[test]
    fn test_ignore_single() {
        let cli = Cli::parse_from(["run", "test", "--ignore", "npm"]);
        assert!(cli.should_ignore("npm"));
        assert!(!cli.should_ignore("yarn"));
    }

    #[test]
    fn test_ignore_comma_separated() {
        let cli = Cli::parse_from(["run", "test", "--ignore=npm,yarn"]);
        assert!(cli.should_ignore("npm"));
        assert!(cli.should_ignore("yarn"));
        assert!(!cli.should_ignore("pnpm"));
    }

    #[test]
    fn test_ignore_multiple_flags() {
        let cli = Cli::parse_from(["run", "test", "--ignore", "npm", "--ignore", "yarn"]);
        assert!(cli.should_ignore("npm"));
        assert!(cli.should_ignore("yarn"));
    }

    #[test]
    fn test_levels() {
        let cli = Cli::parse_from(["run", "test", "--levels=5"]);
        assert_eq!(cli.levels, 5);
    }

    #[test]
    fn test_default_levels() {
        let cli = Cli::parse_from(["run", "test"]);
        assert_eq!(cli.levels, 3);
    }

    #[test]
    fn test_verbose_and_quiet() {
        let cli = Cli::parse_from(["run", "test", "-v"]);
        assert!(cli.verbose);
        assert!(!cli.quiet);

        let cli = Cli::parse_from(["run", "test", "-q"]);
        assert!(!cli.verbose);
        assert!(cli.quiet);
    }

    #[test]
    fn test_dry_run() {
        let cli = Cli::parse_from(["run", "test", "--dry-run"]);
        assert!(cli.dry_run);
    }
}
