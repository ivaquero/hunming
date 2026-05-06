use crate::install::InitShell;
use crate::model::Profile;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "hunming", version, about = "Cross-platform alias manager")]
#[command(arg_required_else_help = true)]
pub struct Cli {
    /// Path to aliases.toml.
    #[arg(long, global = true, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Initialize the configuration file.
    Init(InitArgs),
    /// Add a new alias definition.
    Add(AddArgs),
    /// Remove an alias definition.
    Remove(RemoveArgs),
    /// List known aliases.
    List,
    /// Show an alias definition.
    Show(ShowArgs),
    /// Apply the generated scripts.
    Apply(ApplyArgs),
    /// Back up shell profiles before changes.
    Backup(BackupArgs),
    /// Restore shell profiles from the last backup.
    Restore(RestoreArgs),
    /// Generate shell completions.
    Completions(CompletionsArgs),
    /// Export an aliases.toml template.
    Template(TemplateArgs),
    /// Edit the configuration file.
    Edit,
    /// Check the current installation.
    Doctor(DoctorArgs),
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Initialize only one shell profile.
    #[arg(long, value_enum)]
    pub shell: Option<InitShell>,
}

#[derive(Debug, Args)]
pub struct AddArgs {
    /// Alias name to create.
    pub name: String,

    /// Overwrite an existing alias.
    #[arg(long)]
    pub force: bool,

    /// Bash command to run for this alias.
    #[arg(long)]
    pub bash: Option<String>,

    /// PowerShell command to run for this alias.
    #[arg(long)]
    pub powershell: Option<String>,

    /// Work or personal profile to assign to this alias.
    #[arg(long, value_enum)]
    pub profile: Option<Profile>,

    /// Tag to assign to this alias. Repeat to add more tags.
    #[arg(long = "tag")]
    pub tags: Vec<String>,

    /// Command to run for this alias.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0..)]
    pub command: Vec<String>,
}

#[derive(Debug, Args)]
pub struct ApplyArgs {
    /// Generate only one shell script.
    #[arg(long, value_enum)]
    pub shell: Option<InitShell>,
}

#[derive(Debug, Args)]
pub struct BackupArgs {
    /// Back up only one shell profile.
    #[arg(long, value_enum)]
    pub shell: Option<InitShell>,
}

#[derive(Debug, Args)]
pub struct RestoreArgs {
    /// Restore only one shell profile.
    #[arg(long, value_enum)]
    pub shell: Option<InitShell>,
}

#[derive(Debug, Args)]
pub struct CompletionsArgs {
    /// Shell to generate completions for.
    #[arg(value_enum)]
    pub shell: InitShell,
}

#[derive(Debug, Args)]
pub struct TemplateArgs {
    /// Write the template to a file instead of stdout.
    #[arg(long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct ShowArgs {
    /// Alias name to display.
    pub name: String,
}

#[derive(Debug, Args)]
pub struct DoctorArgs {
    /// Attempt safe repairs.
    #[arg(long)]
    pub fix: bool,
}

#[derive(Debug, Args)]
pub struct RemoveArgs {
    /// Alias name to remove.
    pub name: String,
}
