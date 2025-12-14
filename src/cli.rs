use clap::{Parser, ArgAction, ValueEnum};
use clap_complete::Shell;

#[derive(Parser, Debug)]
#[command(name = "run")]
#[command(version, about = "Universal task runner for modern development", long_about = None)]
#[command(disable_help_flag = true)]
pub struct Cli {
    /// The command to execute (e.g. "test", "build")
    #[arg(value_name = "COMMAND")]
    pub command: Option<String>,

    /// Arguments to pass to the command
    #[arg(allow_hyphen_values = true)]
    pub args: Vec<String>,

    /// Search N levels up for runners
    #[arg(long, default_value = "3")]
    pub levels: usize,

    /// Ignore specific runners (e.g. --ignore=npm,yarn)
    #[arg(long, value_delimiter = ',', action = ArgAction::Append)]
    pub ignore: Vec<String>,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Quiet mode (suppress all output except command output)
    #[arg(short, long)]
    pub quiet: bool,

    /// Dry run (print command without executing)
    #[arg(long)]
    pub dry_run: bool,

    /// Force update check
    #[arg(long)]
    pub update: bool,

    /// Print help
    #[arg(long, action = ArgAction::Help)]
    pub help: Option<bool>,

    /// Internal: Run auto-update logic (hidden)
    #[arg(long, hide = true)]
    pub internal_auto_update: bool,

    /// Generate shell completions
    #[arg(long, value_enum)]
    pub completion: Option<Shell>,
}

pub fn parse_args() -> Cli {
    Cli::parse()
}
