use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = hunming::cli::Cli::parse();

    match cli.command {
        hunming::cli::Commands::Init => {}
        hunming::cli::Commands::Add { .. } => {}
        hunming::cli::Commands::Remove { .. } => {}
        hunming::cli::Commands::List => {}
        hunming::cli::Commands::Apply => {}
    }

    Ok(())
}
