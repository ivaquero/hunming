use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "hunming", version, about = "Cross-platform alias manager")]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Initialize the configuration file.
    Init,
    /// Add a new alias definition.
    Add(AddArgs),
    /// Remove an alias definition.
    Remove(RemoveArgs),
    /// List known aliases.
    List,
    /// Apply the generated scripts.
    Apply,
}

#[derive(Debug, Args)]
pub struct AddArgs {
    /// Alias name to create.
    pub name: String,

    /// Overwrite an existing alias.
    #[arg(long)]
    pub force: bool,

    /// Command to run for this alias.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 1..)]
    pub command: Vec<String>,
}

#[derive(Debug, Args)]
pub struct RemoveArgs {
    /// Alias name to remove.
    pub name: String,
}
