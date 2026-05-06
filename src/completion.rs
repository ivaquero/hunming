use crate::cli::Cli;
use crate::install::InitShell;
use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{Shell, generate};

pub fn generate_completions(shell: InitShell) -> Result<String> {
    let shell = match shell {
        InitShell::Bash => Shell::Bash,
        InitShell::Zsh => Shell::Zsh,
        InitShell::Powershell => Shell::PowerShell,
    };

    let mut command = Cli::command();
    let mut output = Vec::new();
    generate(shell, &mut command, "hunming", &mut output);

    Ok(String::from_utf8(output)?)
}
