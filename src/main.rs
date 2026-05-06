use anyhow::Result;
use clap::Parser;
use hunming::install;
use hunming::paths::AppPaths;

fn main() -> Result<()> {
    let cli = hunming::cli::Cli::parse();

    match cli.command {
        hunming::cli::Commands::Init(args) => {
            let paths = AppPaths::new()?;
            install::init(&paths, args.shell)?;
        }
        hunming::cli::Commands::Add(args) => {
            let paths = AppPaths::new()?;
            install::add(
                &paths,
                args.name,
                args.bash,
                args.powershell,
                args.command,
                args.force,
            )?;
        }
        hunming::cli::Commands::Remove(args) => {
            let paths = AppPaths::new()?;
            install::remove(&paths, args.name)?;
        }
        hunming::cli::Commands::List => {
            let paths = AppPaths::new()?;
            print!("{}", install::list(&paths)?);
        }
        hunming::cli::Commands::Apply => {
            let paths = AppPaths::new()?;
            let result = install::apply(&paths)?;
            println!("{}", result.bash_script.display());
            println!("{}", result.powershell_script.display());
        }
    }

    Ok(())
}
