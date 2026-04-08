//! Command-line interface for hashtime.
//!
//! This binary provides commands for generating, checking, comparing,
//! and restoring file hashes and timestamps.

mod cmd;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hashtime")]
#[command(about = "Generate, check, and restore file hashes and timestamps", long_about = None)]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Gen(cmd::generate::GenArgs),
    Check(cmd::check::CheckArgs),
    Compare(cmd::compare::CompareArgs),
    Restore(cmd::restore::RestoreArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Gen(args) => cmd::generate::run_gen(args),
        Commands::Check(args) => cmd::check::run_check(args),
        Commands::Compare(args) => cmd::compare::run_compare(args),
        Commands::Restore(args) => cmd::restore::run_restore(args),
    }
}
