use anyhow::Result;
use clap::Parser;
use hunming::completion::generate_completions;
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
                args.tags,
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
        hunming::cli::Commands::Show(args) => {
            let paths = AppPaths::new()?;
            print!("{}", install::show(&paths, args.name)?);
        }
        hunming::cli::Commands::Apply(args) => {
            let paths = AppPaths::new()?;
            let result = install::apply(&paths, args.shell)?;
            match args.shell {
                Some(hunming::install::InitShell::Bash) => {
                    println!("{}", result.bash_script.display());
                }
                Some(hunming::install::InitShell::Zsh) => {
                    println!("{}", result.zsh_script.display());
                }
                Some(hunming::install::InitShell::Powershell) => {
                    println!("{}", result.powershell_script.display());
                }
                None => {
                    println!("{}", result.bash_script.display());
                    println!("{}", result.zsh_script.display());
                    println!("{}", result.powershell_script.display());
                }
            }
        }
        hunming::cli::Commands::Completions(args) => {
            print!("{}", generate_completions(args.shell)?);
        }
        hunming::cli::Commands::Edit => {
            let paths = AppPaths::new()?;
            install::edit(&paths)?;
        }
        hunming::cli::Commands::Doctor(args) => {
            let paths = AppPaths::new()?;
            print!("{}", install::doctor(&paths, args.fix)?);
        }
    }

    Ok(())
}
