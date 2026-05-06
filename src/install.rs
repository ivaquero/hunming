use crate::config::load_config;
use crate::config::save_config;
use crate::config::default_config;
use crate::fs::atomic_write;
use crate::model::Alias;
use crate::paths::AppPaths;
use crate::render::{render_bash, render_powershell};
use anyhow::{bail, Context, Result};
use directories::BaseDirs;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;

pub const MANAGED_BLOCK_START: &str = "# >>> hunming init >>>";
pub const MANAGED_BLOCK_END: &str = "# <<< hunming init <<<";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyResult {
    pub bash_script: PathBuf,
    pub powershell_script: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitResult {
    pub config_file: PathBuf,
    pub bash_profile: PathBuf,
    pub powershell_profile: PathBuf,
    pub bash_script: PathBuf,
    pub powershell_script: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitTargets {
    pub bash_profile: PathBuf,
    pub powershell_profile: PathBuf,
}

pub fn apply(paths: &AppPaths) -> Result<ApplyResult> {
    let config = load_config(paths)?;
    paths.ensure_generated_dir()?;

    let bash_script = render_bash(&config);
    let powershell_script = render_powershell(&config);

    atomic_write(&paths.bash_script, &bash_script)?;
    atomic_write(&paths.powershell_script, &powershell_script)?;

    Ok(ApplyResult {
        bash_script: paths.bash_script.clone(),
        powershell_script: paths.powershell_script.clone(),
    })
}

pub fn add(
    paths: &AppPaths,
    name: String,
    command: Vec<String>,
    force: bool,
) -> Result<ApplyResult> {
    let mut config = load_config(paths)?;

    if config.aliases.contains_key(&name) && !force {
        bail!("alias `{name}` already exists; use --force to overwrite");
    }

    config.aliases.insert(
        name,
        Alias {
            command,
            bash: None,
            powershell: None,
        },
    );

    save_config(paths, &config)?;
    apply(paths)
}

pub fn remove(paths: &AppPaths, name: String) -> Result<ApplyResult> {
    let mut config = load_config(paths)?;

    if config.aliases.remove(&name).is_none() {
        bail!("alias `{name}` does not exist");
    }

    save_config(paths, &config)?;
    apply(paths)
}

pub fn list(paths: &AppPaths) -> Result<String> {
    let config = load_config(paths)?;

    if config.aliases.is_empty() {
        return Ok("No aliases configured.\n".to_string());
    }

    let name_width = config
        .aliases
        .keys()
        .map(|name| name.len())
        .max()
        .unwrap_or(0);

    let mut output = String::new();
    for (name, alias) in &config.aliases {
        let (kind, detail) = describe_alias(alias);
        output.push_str(&format!(
            "{name:<name_width$}  {kind:<11}  {detail}\n",
        ));
    }

    Ok(output)
}

pub fn init(paths: &AppPaths) -> Result<InitResult> {
    let targets = default_init_targets()?;
    init_with_targets(paths, &targets)
}

pub fn init_with_targets(paths: &AppPaths, targets: &InitTargets) -> Result<InitResult> {
    paths.ensure_config_dir()?;
    paths.ensure_generated_dir()?;

    if !paths.config_file.exists() {
        save_config(paths, &default_config())?;
    }

    let apply_result = apply(paths)?;
    write_shell_profile(
        &targets.bash_profile,
        &bash_managed_block(&paths.bash_script),
    )?;
    write_shell_profile(
        &targets.powershell_profile,
        &powershell_managed_block(&paths.powershell_script),
    )?;

    Ok(InitResult {
        config_file: paths.config_file.clone(),
        bash_profile: targets.bash_profile.clone(),
        powershell_profile: targets.powershell_profile.clone(),
        bash_script: apply_result.bash_script,
        powershell_script: apply_result.powershell_script,
    })
}

pub fn bash_managed_block(script_path: impl AsRef<Path>) -> String {
    let script_path = script_path.as_ref().display();

    format!(
        "{MANAGED_BLOCK_START}\nif [ -f \"{script_path}\" ]; then\n  . \"{script_path}\"\nfi\n{MANAGED_BLOCK_END}\n"
    )
}

pub fn powershell_managed_block(script_path: impl AsRef<Path>) -> String {
    let script_path = script_path.as_ref().display();

    format!(
        "{MANAGED_BLOCK_START}\n$hunmingProfile = \"{script_path}\"\nif (Test-Path $hunmingProfile) {{\n    . $hunmingProfile\n}}\n{MANAGED_BLOCK_END}\n"
    )
}

pub fn insert_managed_block(existing: &str, block: &str) -> String {
    if let Some((start, end)) = managed_block_range(existing) {
        let mut output = String::with_capacity(existing.len() - (end - start) + block.len());
        output.push_str(&existing[..start]);
        output.push_str(block);
        output.push_str(&existing[end..]);
        return output;
    }

    if existing.is_empty() {
        return block.to_string();
    }

    let mut output = String::with_capacity(existing.len() + block.len() + 2);
    output.push_str(existing);
    if !output.ends_with('\n') {
        output.push('\n');
    }
    if !output.ends_with("\n\n") {
        output.push('\n');
    }
    output.push_str(block);
    output
}

fn managed_block_range(content: &str) -> Option<(usize, usize)> {
    let start = content.find(MANAGED_BLOCK_START)?;
    let after_start = &content[start..];
    let end_rel = after_start.find(MANAGED_BLOCK_END)?;
    let mut end = start + end_rel + MANAGED_BLOCK_END.len();

    if content[end..].starts_with("\r\n") {
        end += 2;
    } else if content[end..].starts_with('\n') {
        end += 1;
    }

    Some((start, end))
}

pub fn write_shell_profile(profile_path: impl AsRef<Path>, block: &str) -> Result<()> {
    let profile_path = profile_path.as_ref();
    let existing = match fs::read_to_string(profile_path) {
        Ok(content) => content,
        Err(err) if err.kind() == ErrorKind::NotFound => String::new(),
        Err(err) => Err(err).with_context(|| {
            format!(
                "failed to read shell profile at {}",
                profile_path.display()
            )
        })?,
    };
    let updated = insert_managed_block(&existing, block);
    atomic_write(profile_path, &updated)
}

fn describe_alias(alias: &Alias) -> (&'static str, String) {
    let has_command = !alias.command.is_empty();
    let has_bash = alias.bash.as_ref().is_some_and(|value| !value.trim().is_empty());
    let has_powershell = alias
        .powershell
        .as_ref()
        .is_some_and(|value| !value.trim().is_empty());

    if has_bash && has_powershell {
        return (
            "shell",
            format!(
                "bash: {} | powershell: {}",
                alias.bash.as_deref().unwrap_or_default(),
                alias.powershell.as_deref().unwrap_or_default()
            ),
        );
    }

    if has_bash {
        return ("bash", alias.bash.as_deref().unwrap_or_default().to_string());
    }

    if has_powershell {
        return (
            "powershell",
            alias.powershell.as_deref().unwrap_or_default().to_string(),
        );
    }

    if has_command {
        return ("command", alias.command.join(" "));
    }

    ("command", String::new())
}

fn default_init_targets() -> Result<InitTargets> {
    let base_dirs =
        BaseDirs::new().context("failed to determine home directory for hunming init")?;
    let home_dir = base_dirs.home_dir();

    let bash_profile = home_dir.join(".bashrc");
    let powershell_profile = if cfg!(windows) {
        home_dir
            .join("Documents")
            .join("PowerShell")
            .join("Microsoft.PowerShell_profile.ps1")
    } else {
        home_dir
            .join(".config")
            .join("powershell")
            .join("Microsoft.PowerShell_profile.ps1")
    };

    Ok(InitTargets {
        bash_profile,
        powershell_profile,
    })
}
