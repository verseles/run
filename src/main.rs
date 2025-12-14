mod cli;
mod config;
mod detectors;
mod runner;
mod update;

use clap::{Parser, CommandFactory};
use owo_colors::OwoColorize;
use std::env;
use anyhow::{Result, Context};
use clap_complete::generate;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Check for internal auto update flag
    let cli_basic = cli::Cli::parse();
    let mut cli = cli_basic;

    if cli.internal_auto_update {
        if let Err(_e) = update::perform_update().await {
            // Silently fail
        }
        return Ok(());
    }

    // Handle completions
    if let Some(shell) = cli.completion {
        let mut cmd = cli::Cli::command();
        let bin_name = cmd.get_name().to_string();
        generate(shell, &mut cmd, bin_name, &mut std::io::stdout());
        return Ok(());
    }

    // Load config
    let config = config::load_config();

    // Check for update notification from previous run
    if !cli.quiet {
         update::check_for_update_notification();
    }

    // Apply config defaults to CLI
    if cli.levels == 3 {
        if let Some(l) = config.max_levels {
            cli.levels = l;
        }
    }

    if !cli.verbose {
         if let Some(v) = config.verbose {
             if v { cli.verbose = true; }
         }
    }

    if !cli.quiet {
        if let Some(q) = config.quiet {
            if q { cli.quiet = true; }
        }
    }

    if cli.ignore.is_empty() {
        if let Some(ignored) = &config.ignore_tools {
            cli.ignore = ignored.clone();
        }
    }


    if cli.command.is_none() && !cli.update {
        // Show help if no command and not updating
        use clap::CommandFactory;
        let mut cmd = cli::Cli::command();
        cmd.print_help().unwrap();
        return Ok(());
    }

    // Force update if requested
    if cli.update {
         println!("{} Checking for updates...", "🔍".blue());
         if let Err(e) = update::perform_update().await {
             eprintln!("{} Failed to update: {}", "❌".red(), e);
             std::process::exit(1);
         } else {
             println!("{} Update check complete.", "✓".green());
         }
    }

    if cli.verbose {
        println!("{} {}", "🔍".blue(), "Run CLI started".blue());
        println!("Config loaded: {:?}", config);
    }

    // Handle command execution
    if let Some(cmd) = cli.command {
        // Detect runner
        let current_dir = env::current_dir().context("Failed to get current directory")?;

        let detection_result = detectors::find_runner(
            &current_dir,
            cli.levels,
            &cli.ignore
        )?;

        if let Some((detection, detected_path)) = detection_result {
            if cli.verbose {
                println!("{} Detected {} in {}", "📦".green(), detection.runner.yellow(), detected_path.display());
            }

            if !cli.quiet {
                 println!("{} Executing: {} {}", "✓".green(), detection.command.cyan(), cmd);
            }

            let mut final_args = vec![cmd];
            final_args.extend(cli.args);

            if cli.dry_run {
                println!("Dry run: {} {}", detection.command, final_args.join(" "));
                return Ok(());
            }

            let exit_code = runner::execute_command(&detection, &final_args)?;

            // Trigger auto-update in background if configured
            let auto_update = config.auto_update.unwrap_or(true);
            if auto_update && !cli.update {
                 let _ = update::spawn_auto_update();
            }

            std::process::exit(exit_code);

        } else {
             eprintln!("{}", format!("Erro: Nenhum runner encontrado em {} níveis acima do diretório atual.", cli.levels).red());
             eprintln!("Dica: Use --levels=N para aumentar a busca ou --ignore=<tool> para ajustar detecção.");
             std::process::exit(2);
        }
    }

    Ok(())
}
