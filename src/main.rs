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
        hunming::cli::Commands::Apply(args) => {
            let paths = AppPaths::new()?;
            let result = install::apply(&paths, args.shell)?;
            match args.shell {
                Some(hunming::install::InitShell::Bash) => {
                    println!("{}", result.bash_script.display());
                }
                Some(hunming::install::InitShell::Powershell) => {
                    println!("{}", result.powershell_script.display());
                }
                None => {
                    println!("{}", result.bash_script.display());
                    println!("{}", result.powershell_script.display());
                }
            }
        }
    }

    Ok(())
}
