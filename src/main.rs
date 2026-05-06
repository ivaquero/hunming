use anyhow::Result;
use clap::Parser;
use hunming::completion::generate_completions;
use hunming::config::render_template;
use hunming::fs::atomic_write;
use hunming::install;
use hunming::paths::AppPaths;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<()> {
    let cli = hunming::cli::Cli::parse();
    let paths = resolve_paths(cli.config)?;
    let profile = cli.profile;

    match cli.command {
        hunming::cli::Commands::Init(args) => {
            install::init_with_profile(&paths, args.shell, profile)?;
        }
        hunming::cli::Commands::Add(args) => {
            install::add_with_profile(
                &paths,
                args.name,
                args.bash,
                args.powershell,
                args.profile,
                args.tags,
                args.command,
                args.force,
                profile,
            )?;
        }
        hunming::cli::Commands::Remove(args) => {
            install::remove_with_profile(&paths, args.name, profile)?;
        }
        hunming::cli::Commands::List => {
            print!("{}", install::list(&paths)?);
        }
        hunming::cli::Commands::Show(args) => {
            print!("{}", install::show(&paths, args.name)?);
        }
        hunming::cli::Commands::Apply(args) => {
            let result = install::apply_with_profile(&paths, args.shell, profile)?;
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
        hunming::cli::Commands::Backup(args) => {
            let result = install::backup(&paths, args.shell)?;
            for path in result.profile_paths {
                println!("{}", path.display());
            }
        }
        hunming::cli::Commands::Restore(args) => {
            let result = install::restore(&paths, args.shell)?;
            for path in result.profile_paths {
                println!("{}", path.display());
            }
        }
        hunming::cli::Commands::Completions(args) => {
            print!("{}", generate_completions(args.shell)?);
        }
        hunming::cli::Commands::Template(args) => {
            let content = render_template()?;
            if let Some(output) = args.output {
                if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
                    fs::create_dir_all(parent)?;
                }
                atomic_write(&output, &content)?;
                println!("{}", output.display());
            } else {
                print!("{}", content);
            }
        }
        hunming::cli::Commands::Edit => {
            install::edit_with_profile(&paths, profile)?;
        }
        hunming::cli::Commands::Doctor(args) => {
            print!("{}", install::doctor_with_profile(&paths, args.fix, profile)?);
        }
    }

    Ok(())
}

fn resolve_paths(config: Option<PathBuf>) -> Result<AppPaths> {
    match config {
        Some(config_file) => Ok(AppPaths::from_config_file(config_file)),
        None => AppPaths::new(),
    }
}
