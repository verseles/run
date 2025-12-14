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

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use run_cli::cli::{Cli, Commands};
use run_cli::config::Config;
use run_cli::error::exit_codes;
use run_cli::output;
use run_cli::runner::{check_conflicts, execute, search_runners};
use run_cli::update;
use std::env;
use std::io;
use std::process;

fn main() {
    // Check for internal update flag (used by background updater)
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--internal-update-check" {
        // Run update check in background
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let _ = rt.block_on(update::perform_update_check());
        return;
    }

    // Parse CLI arguments
    let cli = Cli::parse();

    // Load configuration
    let config = Config::load();

    // Merge config with CLI arguments
    let verbose = cli.verbose || config.get_verbose();
    let quiet = cli.quiet || config.get_quiet();
    let max_levels = cli.levels;
    let mut ignore_list = config.ignore_tools.clone();
    ignore_list.extend(cli.ignore.clone());

    // Check for update notification
    update::check_update_notification(quiet);

    // Handle subcommands
    if let Some(Commands::Completions { shell }) = cli.subcommand {
        let mut cmd = Cli::command();
        let name = cmd.get_name().to_string();
        generate(shell, &mut cmd, name, &mut io::stdout());
        return;
    }

    // Handle --update flag
    if cli.update {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        match rt.block_on(update::perform_blocking_update(quiet)) {
            Ok(_) => process::exit(exit_codes::SUCCESS),
            Err(e) => {
                output::error(&format!("Update failed: {}", e));
                process::exit(exit_codes::GENERIC_ERROR);
            }
        }
    }

    // Require a command
    let command = match &cli.command {
        Some(cmd) => cmd.clone(),
        None => {
            // If no command, just show help
            Cli::command().print_help().unwrap();
            println!();
            process::exit(exit_codes::SUCCESS);
        }
    };

    // Get current directory
    let current_dir = match env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            output::error(&format!("Failed to get current directory: {}", e));
            process::exit(exit_codes::GENERIC_ERROR);
        }
    };

    // Search for runners
    let (runners, working_dir) = match search_runners(
        &current_dir,
        max_levels,
        &ignore_list,
        verbose,
    ) {
        Ok(result) => result,
        Err(e) => {
            output::error(&e.to_string());
            eprintln!("Hint: Use --levels=N to increase search depth or check if you're in the right directory.");
            process::exit(e.exit_code());
        }
    };

    // Check for conflicts and select runner
    let runner = match check_conflicts(&runners, verbose) {
        Ok(r) => r,
        Err(e) => {
            output::error(&e.to_string());
            process::exit(e.exit_code());
        }
    };

    // Execute the command
    let result = match execute(
        &runner,
        &command,
        &cli.args,
        &working_dir,
        cli.dry_run,
        verbose,
        quiet,
    ) {
        Ok(r) => r,
        Err(e) => {
            output::error(&e.to_string());
            process::exit(e.exit_code());
        }
    };

    // For dry run, always exit successfully
    if cli.dry_run {
        process::exit(exit_codes::SUCCESS);
    }

    // Spawn background update check (after command completes)
    if config.get_auto_update() && !update::is_update_disabled() {
        update::spawn_background_update();
    }

    // Exit with the same code as the executed command
    let exit_code = result
        .exit_status
        .code()
        .unwrap_or(exit_codes::GENERIC_ERROR);
    process::exit(exit_code);
}
