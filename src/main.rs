mod cli;
mod config;
mod fs;
mod install;
mod model;
mod paths;
mod render;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Init => {}
        cli::Commands::Add { .. } => {}
        cli::Commands::Remove { .. } => {}
        cli::Commands::List => {}
        cli::Commands::Apply => {}
    }

    Ok(())
}
