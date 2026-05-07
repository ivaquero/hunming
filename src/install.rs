use crate::config::default_config;
use crate::config::load_config;
use crate::config::load_config_from_path;
use crate::config::save_config;
use crate::fs::atomic_write;
use crate::model::{Alias, Config, Profile};
use crate::paths::AppPaths;
use crate::render::{render_bash_with_profile, render_powershell_with_profile};
use crate::validation::validate_alias_name;
use anyhow::{Context, Result, bail};
use clap::ValueEnum;
use directories::BaseDirs;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

pub const MANAGED_BLOCK_START: &str = "# >>> hunming init >>>";
pub const MANAGED_BLOCK_END: &str = "# <<< hunming init <<<";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyResult {
    pub bash_script: PathBuf,
    pub zsh_script: PathBuf,
    pub powershell_script: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileResult {
    pub profile_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitResult {
    pub config_file: PathBuf,
    pub bash_profile: PathBuf,
    pub zsh_profile: PathBuf,
    pub powershell_profile: PathBuf,
    pub bash_script: PathBuf,
    pub zsh_script: PathBuf,
    pub powershell_script: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitTargets {
    pub bash_profile: PathBuf,
    pub zsh_profile: PathBuf,
    pub powershell_profile: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorTargets {
    pub bash_rc_profile: PathBuf,
    pub bash_login_profile: PathBuf,
    pub zsh_profile: PathBuf,
    pub powershell_profile: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "lower")]
pub enum InitShell {
    Bash,
    Zsh,
    Powershell,
}

pub fn apply(paths: &AppPaths, shell: Option<InitShell>) -> Result<ApplyResult> {
    apply_with_profile(paths, shell, None)
}

pub fn apply_with_profile(
    paths: &AppPaths,
    shell: Option<InitShell>,
    profile: Option<Profile>,
) -> Result<ApplyResult> {
    let config = load_config(paths)?;
    paths.ensure_generated_dir()?;

    let bash_script = if shell.is_none() || matches!(shell, Some(InitShell::Bash)) {
        Some(render_bash_with_profile(&config, profile))
    } else {
        None
    };
    let zsh_script = if shell.is_none() || matches!(shell, Some(InitShell::Zsh)) {
        Some(render_bash_with_profile(&config, profile))
    } else {
        None
    };
    let powershell_script = if shell.is_none() || matches!(shell, Some(InitShell::Powershell)) {
        Some(render_powershell_with_profile(&config, profile))
    } else {
        None
    };

    if let Some(script) = &bash_script {
        atomic_write(&paths.bash_script, script)?;
    }
    if let Some(script) = &zsh_script {
        atomic_write(&paths.zsh_script, script)?;
    }
    if let Some(script) = &powershell_script {
        atomic_write(&paths.powershell_script, script)?;
    }

    Ok(ApplyResult {
        bash_script: paths.bash_script.clone(),
        zsh_script: paths.zsh_script.clone(),
        powershell_script: paths.powershell_script.clone(),
    })
}

pub fn edit(paths: &AppPaths) -> Result<ApplyResult> {
    edit_with_profile(paths, None)
}

pub fn edit_with_profile(paths: &AppPaths, profile: Option<Profile>) -> Result<ApplyResult> {
    edit_with_opener_and_profile(paths, profile, |config_file| {
        let editor = resolve_editor()?;
        run_editor(&editor, config_file)
    })
}

pub fn edit_with_opener<F>(paths: &AppPaths, opener: F) -> Result<ApplyResult>
where
    F: FnOnce(&Path) -> Result<()>,
{
    edit_with_opener_and_profile(paths, None, opener)
}

pub fn edit_with_opener_and_profile<F>(
    paths: &AppPaths,
    profile: Option<Profile>,
    opener: F,
) -> Result<ApplyResult>
where
    F: FnOnce(&Path) -> Result<()>,
{
    paths.ensure_config_dir()?;

    if !paths.config_file.exists() {
        save_config(paths, &default_config())?;
    }

    opener(&paths.config_file)?;
    apply_with_profile(paths, None, profile)
}

pub fn backup(_paths: &AppPaths, shell: Option<InitShell>) -> Result<ProfileResult> {
    let targets = default_init_targets()?;
    backup_with_targets(&targets, shell)
}

pub fn backup_with_targets(
    targets: &InitTargets,
    shell: Option<InitShell>,
) -> Result<ProfileResult> {
    let profiles = selected_profile_paths(targets, shell);
    let mut backup_paths = Vec::new();

    for profile in profiles {
        if let Some(backup_path) = backup_shell_profile(&profile)? {
            backup_paths.push(backup_path);
        }
    }

    if backup_paths.is_empty() {
        bail!("no shell profiles found to back up");
    }

    Ok(ProfileResult {
        profile_paths: backup_paths,
    })
}

pub fn restore(_paths: &AppPaths, shell: Option<InitShell>) -> Result<ProfileResult> {
    let targets = default_init_targets()?;
    restore_with_targets(&targets, shell)
}

pub fn restore_with_targets(
    targets: &InitTargets,
    shell: Option<InitShell>,
) -> Result<ProfileResult> {
    let profiles = selected_profile_paths(targets, shell);

    for profile in &profiles {
        let backup_path = shell_profile_backup_path(profile);
        if !backup_path.exists() {
            bail!(
                "backup for {} is missing at {}",
                profile.display(),
                backup_path.display()
            );
        }
    }

    let mut restored_paths = Vec::new();
    for profile in profiles {
        restore_shell_profile(&profile)?;
        restored_paths.push(profile);
    }

    Ok(ProfileResult {
        profile_paths: restored_paths,
    })
}

pub fn add(
    paths: &AppPaths,
    name: String,
    bash: Option<String>,
    powershell: Option<String>,
    profile: Option<Profile>,
    tags: Vec<String>,
    command: Vec<String>,
    force: bool,
) -> Result<ApplyResult> {
    add_with_profile(
        paths, name, bash, powershell, profile, tags, command, force, None,
    )
}

pub fn add_with_profile(
    paths: &AppPaths,
    name: String,
    bash: Option<String>,
    powershell: Option<String>,
    profile: Option<Profile>,
    tags: Vec<String>,
    command: Vec<String>,
    force: bool,
    render_profile: Option<Profile>,
) -> Result<ApplyResult> {
    validate_alias_name(&name)?;
    let mut config = load_config(paths)?;

    if config.aliases.contains_key(&name) && !force {
        bail!("alias `{name}` already exists; use --force to overwrite");
    }

    config.aliases.insert(
        name,
        Alias {
            description: None,
            command,
            tags: normalize_tags(tags),
            bash: normalize_optional(bash),
            powershell: normalize_optional(powershell),
            forward_args: true,
            platforms: Vec::new(),
            profile,
        },
    );

    save_config(paths, &config)?;
    apply_with_profile(paths, None, render_profile)
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn resolve_editor() -> Result<Vec<String>> {
    for key in ["VISUAL", "EDITOR"] {
        if let Some(value) = env::var_os(key) {
            let value = value.to_string_lossy().trim().to_string();
            if !value.is_empty() {
                return Ok(value
                    .split_whitespace()
                    .map(|part| part.to_string())
                    .collect());
            }
        }
    }

    if cfg!(windows) {
        Ok(vec!["notepad".to_string()])
    } else {
        Ok(vec!["vi".to_string()])
    }
}

fn run_editor(editor: &[String], config_file: &Path) -> Result<()> {
    let Some((program, args)) = editor.split_first() else {
        bail!("no editor configured");
    };

    let status = Command::new(program)
        .args(args)
        .arg(config_file)
        .status()
        .with_context(|| format!("failed to launch editor `{program}`"))?;

    if status.success() {
        Ok(())
    } else {
        bail!("editor `{program}` exited with status {status}")
    }
}

pub fn remove(paths: &AppPaths, name: String) -> Result<ApplyResult> {
    remove_with_profile(paths, name, None)
}

pub fn remove_with_profile(
    paths: &AppPaths,
    name: String,
    render_profile: Option<Profile>,
) -> Result<ApplyResult> {
    validate_alias_name(&name)?;
    let mut config = load_config(paths)?;

    if config.aliases.remove(&name).is_none() {
        bail!("alias `{name}` does not exist");
    }

    save_config(paths, &config)?;
    apply_with_profile(paths, None, render_profile)
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
    let kind_width = config
        .aliases
        .values()
        .map(|alias| describe_alias(alias).0.len())
        .max()
        .unwrap_or(0);
    let tags_width = config
        .aliases
        .values()
        .map(|alias| format_tags(&alias.tags).len())
        .max()
        .unwrap_or(1)
        .max(4);

    let mut output = String::new();
    for (name, alias) in &config.aliases {
        let (kind, detail) = describe_alias(alias);
        let tags = format_tags(&alias.tags);
        output.push_str(&format!(
            "{name:<name_width$}  {kind:<kind_width$}  {tags:<tags_width$}  {detail}\n",
        ));
    }

    Ok(output)
}

pub fn show(paths: &AppPaths, name: String) -> Result<String> {
    validate_alias_name(&name)?;
    let config = load_config(paths)?;
    let alias = config
        .aliases
        .get(&name)
        .ok_or_else(|| anyhow::anyhow!("alias `{name}` does not exist"))?;

    Ok(render_alias_definition(&name, alias))
}

pub fn init(paths: &AppPaths, shell: Option<InitShell>) -> Result<InitResult> {
    init_with_profile(paths, shell, None)
}

pub fn init_with_profile(
    paths: &AppPaths,
    shell: Option<InitShell>,
    profile: Option<Profile>,
) -> Result<InitResult> {
    let targets = default_init_targets()?;
    init_with_targets_and_shell_and_profile(paths, &targets, shell, profile)
}

pub fn init_with_targets(paths: &AppPaths, targets: &InitTargets) -> Result<InitResult> {
    init_with_targets_and_shell_and_profile(paths, targets, None, None)
}

pub fn init_with_targets_and_shell(
    paths: &AppPaths,
    targets: &InitTargets,
    shell: Option<InitShell>,
) -> Result<InitResult> {
    init_with_targets_and_shell_and_profile(paths, targets, shell, None)
}

pub fn init_with_targets_and_shell_and_profile(
    paths: &AppPaths,
    targets: &InitTargets,
    shell: Option<InitShell>,
    profile: Option<Profile>,
) -> Result<InitResult> {
    paths.ensure_config_dir()?;
    paths.ensure_generated_dir()?;

    if !paths.config_file.exists() {
        save_config(paths, &default_config())?;
    }

    let apply_result = apply_with_profile(paths, None, profile)?;

    if shell.is_none() || matches!(shell, Some(InitShell::Bash)) {
        write_shell_profile(
            &targets.bash_profile,
            &bash_managed_block(&paths.bash_script),
        )?;
    }

    if shell.is_none() || matches!(shell, Some(InitShell::Zsh)) {
        write_shell_profile(&targets.zsh_profile, &bash_managed_block(&paths.zsh_script))?;
    }

    if shell.is_none() || matches!(shell, Some(InitShell::Powershell)) {
        write_shell_profile(
            &targets.powershell_profile,
            &powershell_managed_block(&paths.powershell_script),
        )?;
    }

    Ok(InitResult {
        config_file: paths.config_file.clone(),
        bash_profile: targets.bash_profile.clone(),
        zsh_profile: targets.zsh_profile.clone(),
        powershell_profile: targets.powershell_profile.clone(),
        bash_script: apply_result.bash_script,
        zsh_script: apply_result.zsh_script,
        powershell_script: apply_result.powershell_script,
    })
}

pub fn doctor(paths: &AppPaths, fix: bool) -> Result<String> {
    doctor_with_profile(paths, fix, None)
}

pub fn doctor_with_profile(
    paths: &AppPaths,
    fix: bool,
    profile: Option<Profile>,
) -> Result<String> {
    let targets = default_doctor_targets()?;
    doctor_with_targets_and_profile(paths, &targets, fix, profile)
}

pub fn doctor_with_targets(paths: &AppPaths, targets: &DoctorTargets, fix: bool) -> Result<String> {
    doctor_with_targets_and_profile(paths, targets, fix, None)
}

pub fn doctor_with_targets_and_profile(
    paths: &AppPaths,
    targets: &DoctorTargets,
    fix: bool,
    profile: Option<Profile>,
) -> Result<String> {
    let initial_state = inspect_config(paths);

    if fix {
        repair_doctor(paths, targets, &initial_state, profile)?;
    }

    let config_state = inspect_config(paths);
    let mut report = DoctorReport::default();

    match &config_state {
        ConfigState::Missing => report.warn("config file missing"),
        ConfigState::Invalid(error) => report.warn(format!("config file is invalid: {error}")),
        ConfigState::Valid(_) => report.ok("config file exists"),
    }

    if paths.bash_script.exists() {
        report.ok("generated bash file exists");
    } else {
        report.warn("generated bash file missing");
    }

    if paths.zsh_script.exists() {
        report.ok("generated zsh file exists");
    } else {
        report.warn("generated zsh file missing");
    }

    if paths.powershell_script.exists() {
        report.ok("generated powershell file exists");
    } else {
        report.warn("generated powershell file missing");
    }

    check_profile_block(
        &mut report,
        "~/.bashrc",
        &targets.bash_rc_profile,
        &bash_managed_block(&paths.bash_script),
    );

    check_profile_block(
        &mut report,
        "~/.zshrc",
        &targets.zsh_profile,
        &bash_managed_block(&paths.zsh_script),
    );

    if profile_sources_bash_rc(&targets.bash_login_profile) {
        report.ok("~/.bash_profile sources ~/.bashrc");
    } else {
        report.warn("~/.bash_profile does not source ~/.bashrc");
    }

    check_profile_block(
        &mut report,
        "PowerShell profile",
        &targets.powershell_profile,
        &powershell_managed_block(&paths.powershell_script),
    );

    report.warn("PowerShell execution policy may block profile loading");

    if let ConfigState::Valid(config) = &config_state {
        report.ok("no duplicated alias names");

        let mut shadowed = Vec::new();
        for name in config.aliases.keys() {
            if command_exists(name) {
                shadowed.push(name.clone());
            }
        }

        if shadowed.is_empty() {
            report.ok("no aliases shadow existing command");
        } else {
            for name in shadowed {
                report.warn(format!("alias \"{name}\" shadows existing command"));
            }
        }
    }

    Ok(report.finish())
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
            format!("failed to read shell profile at {}", profile_path.display())
        })?,
    };
    let updated = insert_managed_block(&existing, block);
    if existing == updated {
        return Ok(());
    }

    backup_shell_profile(profile_path)?;
    atomic_write(profile_path, &updated)
}

fn backup_shell_profile(profile_path: impl AsRef<Path>) -> Result<Option<PathBuf>> {
    let profile_path = profile_path.as_ref();
    if !profile_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(profile_path)
        .with_context(|| format!("failed to read shell profile at {}", profile_path.display()))?;
    let backup_path = shell_profile_backup_path(profile_path);
    atomic_write(&backup_path, &content)?;
    Ok(Some(backup_path))
}

fn restore_shell_profile(profile_path: impl AsRef<Path>) -> Result<PathBuf> {
    let profile_path = profile_path.as_ref();
    let backup_path = shell_profile_backup_path(profile_path);
    let content = fs::read_to_string(&backup_path)
        .with_context(|| format!("failed to read backup at {}", backup_path.display()))?;
    atomic_write(profile_path, &content)?;
    Ok(profile_path.to_path_buf())
}

fn shell_profile_backup_path(profile_path: impl AsRef<Path>) -> PathBuf {
    let profile_path = profile_path.as_ref();
    let backup_name = match profile_path.file_name() {
        Some(name) => format!("{}.hunming.bak", name.to_string_lossy()),
        None => "hunming.bak".to_string(),
    };

    profile_path.with_file_name(backup_name)
}

fn describe_alias(alias: &Alias) -> (&'static str, String) {
    let has_command = !alias.command.is_empty();
    let has_bash = alias
        .bash
        .as_ref()
        .is_some_and(|value| !value.trim().is_empty());
    let has_powershell = alias
        .powershell
        .as_ref()
        .is_some_and(|value| !value.trim().is_empty());
    let profile = alias.profile.map(format_profile);

    if has_bash && has_powershell {
        return (
            "shell",
            format!(
                "{}bash: {} | powershell: {}",
                profile
                    .as_ref()
                    .map(|value| format!("profile: {value} | "))
                    .unwrap_or_default(),
                alias.bash.as_deref().unwrap_or_default(),
                alias.powershell.as_deref().unwrap_or_default()
            ),
        );
    }

    if has_bash {
        return (
            "bash",
            describe_with_profile(alias.bash.as_deref(), profile),
        );
    }

    if has_powershell {
        return (
            "powershell",
            describe_with_profile(alias.powershell.as_deref(), profile),
        );
    }

    if has_command {
        let mut detail = alias.command.join(" ");
        if !alias.forward_args {
            detail.push_str(" (no args)");
        }
        if let Some(profile) = profile {
            detail = format!("profile: {profile} | {detail}");
        }
        return ("command", detail);
    }

    (
        "command",
        profile.map_or_else(String::new, |profile| format!("profile: {profile}")),
    )
}

fn render_alias_definition(name: &str, alias: &Alias) -> String {
    let mut output = String::new();
    output.push_str(&format!("[aliases.{name}]\n"));

    if let Some(description) = &alias.description {
        output.push_str("description = ");
        output.push_str(&toml_string(description));
        output.push('\n');
    }

    if !alias.command.is_empty() {
        output.push_str("command = ");
        output.push_str(&toml_strings(&alias.command));
        output.push('\n');
    }

    if !alias.tags.is_empty() {
        output.push_str("tags = ");
        output.push_str(&toml_strings(&alias.tags));
        output.push('\n');
    }

    if let Some(profile) = alias.profile {
        output.push_str("profile = ");
        output.push_str(&toml_profile(profile));
        output.push('\n');
    }

    if let Some(bash) = &alias.bash {
        output.push_str("bash = ");
        output.push_str(&toml_string(bash));
        output.push('\n');
    }

    if let Some(powershell) = &alias.powershell {
        output.push_str("powershell = ");
        output.push_str(&toml_string(powershell));
        output.push('\n');
    }

    if !alias.forward_args {
        output.push_str("forward_args = false\n");
    }

    if !alias.platforms.is_empty() {
        output.push_str("platforms = ");
        output.push_str(&toml_platforms(&alias.platforms));
        output.push('\n');
    }

    output
}

fn toml_string(value: &str) -> String {
    format!("{value:?}")
}

fn toml_strings(values: &[String]) -> String {
    let mut output = String::from("[");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push_str(&toml_string(value));
    }
    output.push(']');
    output
}

fn format_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        "-".to_string()
    } else {
        tags.join(", ")
    }
}

fn toml_platforms(values: &[crate::model::Platform]) -> String {
    let mut output = String::from("[");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let item = match value {
            crate::model::Platform::Windows => "\"windows\"",
            crate::model::Platform::Macos => "\"macos\"",
            crate::model::Platform::Linux => "\"linux\"",
        };
        output.push_str(item);
    }
    output.push(']');
    output
}

fn toml_profile(profile: Profile) -> String {
    match profile {
        Profile::Work => toml_string("work"),
        Profile::Personal => toml_string("personal"),
    }
}

fn format_profile(profile: Profile) -> String {
    match profile {
        Profile::Work => "work".to_string(),
        Profile::Personal => "personal".to_string(),
    }
}

fn describe_with_profile(detail: Option<&str>, profile: Option<String>) -> String {
    let detail = detail.unwrap_or_default();

    match profile {
        Some(profile) if detail.is_empty() => format!("profile: {profile}"),
        Some(profile) => format!("profile: {profile} | {detail}"),
        None => detail.to_string(),
    }
}

fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    let mut seen = std::collections::BTreeSet::new();

    for tag in tags {
        let tag = tag.trim();
        if tag.is_empty() {
            continue;
        }

        let tag = tag.to_string();
        if seen.insert(tag.clone()) {
            normalized.push(tag);
        }
    }

    normalized
}

fn repair_doctor(
    paths: &AppPaths,
    targets: &DoctorTargets,
    config_state: &ConfigState,
    profile: Option<Profile>,
) -> Result<()> {
    match config_state {
        ConfigState::Missing => {
            save_config(paths, &default_config())?;
            apply_with_profile(paths, None, profile)?;
            write_shell_profile(
                &targets.bash_rc_profile,
                &bash_managed_block(&paths.bash_script),
            )?;
            write_shell_profile(&targets.zsh_profile, &bash_managed_block(&paths.zsh_script))?;
            write_shell_profile(
                &targets.powershell_profile,
                &powershell_managed_block(&paths.powershell_script),
            )?;
        }
        ConfigState::Valid(_) => {
            apply_with_profile(paths, None, profile)?;
            write_shell_profile(
                &targets.bash_rc_profile,
                &bash_managed_block(&paths.bash_script),
            )?;
            write_shell_profile(&targets.zsh_profile, &bash_managed_block(&paths.zsh_script))?;
            write_shell_profile(
                &targets.powershell_profile,
                &powershell_managed_block(&paths.powershell_script),
            )?;
        }
        ConfigState::Invalid(_) => {}
    }

    Ok(())
}

fn inspect_config(paths: &AppPaths) -> ConfigState {
    if !paths.config_file.exists() {
        return ConfigState::Missing;
    }

    match load_config_from_path(&paths.config_file) {
        Ok(config) => ConfigState::Valid(config),
        Err(err) => ConfigState::Invalid(err.to_string()),
    }
}

fn check_profile_block(report: &mut DoctorReport, label: &str, path: &Path, block: &str) {
    match fs::read_to_string(path) {
        Ok(content) if content.contains(block) => {
            report.ok(format!("{label} contains humming managed block"));
        }
        Ok(_) => {
            report.warn(format!("{label} does not contain humming managed block"));
        }
        Err(err) if err.kind() == ErrorKind::NotFound => {
            report.warn(format!("{label} missing"));
        }
        Err(err) => {
            report.warn(format!(
                "failed to read {label} at {}: {err}",
                path.display()
            ));
        }
    }
}

fn profile_sources_bash_rc(path: &Path) -> bool {
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };

    content.contains(". ~/.bashrc")
        || content.contains("source ~/.bashrc")
        || content.contains(". \"$HOME/.bashrc\"")
        || content.contains("source \"$HOME/.bashrc\"")
        || content.contains(". ${HOME}/.bashrc")
        || content.contains("source ${HOME}/.bashrc")
}

fn command_exists(name: &str) -> bool {
    let Some(path_var) = env::var_os("PATH") else {
        return false;
    };

    for dir in env::split_paths(&path_var) {
        if cfg!(windows) {
            for candidate in command_candidates_windows(&dir, name) {
                if candidate.is_file() {
                    return true;
                }
            }
        } else if dir.join(name).is_file() {
            return true;
        }
    }

    false
}

#[cfg(windows)]
fn command_candidates_windows(dir: &Path, name: &str) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    candidates.push(dir.join(name));

    let exts = env::var_os("PATHEXT")
        .map(|value| {
            value
                .to_string_lossy()
                .split(';')
                .filter(|item| !item.is_empty())
                .map(|item| item.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec![".COM".into(), ".EXE".into(), ".BAT".into(), ".CMD".into()]);

    if Path::new(name).extension().is_none() {
        for ext in exts {
            candidates.push(dir.join(format!("{name}{ext}")));
        }
    }

    candidates
}

#[cfg(not(windows))]
fn command_candidates_windows(_dir: &Path, _name: &str) -> Vec<PathBuf> {
    Vec::new()
}

fn selected_profile_paths(targets: &InitTargets, shell: Option<InitShell>) -> Vec<PathBuf> {
    match shell {
        Some(InitShell::Bash) => vec![targets.bash_profile.clone()],
        Some(InitShell::Zsh) => vec![targets.zsh_profile.clone()],
        Some(InitShell::Powershell) => vec![targets.powershell_profile.clone()],
        None => vec![
            targets.bash_profile.clone(),
            targets.zsh_profile.clone(),
            targets.powershell_profile.clone(),
        ],
    }
}

#[derive(Default)]
struct DoctorReport {
    lines: Vec<String>,
}

impl DoctorReport {
    fn ok(&mut self, message: impl Into<String>) {
        self.lines.push(format!("[✓] {}", message.into()));
    }

    fn warn(&mut self, message: impl Into<String>) {
        self.lines.push(format!("[!] {}", message.into()));
    }

    fn finish(self) -> String {
        if self.lines.is_empty() {
            String::new()
        } else {
            format!("{}\n", self.lines.join("\n"))
        }
    }
}

enum ConfigState {
    Missing,
    Invalid(String),
    Valid(Config),
}

fn default_init_targets() -> Result<InitTargets> {
    let base_dirs =
        BaseDirs::new().context("failed to determine home directory for hunming init")?;
    let home_dir = base_dirs.home_dir();

    let bash_profile = home_dir.join(".bashrc");
    let zsh_profile = home_dir.join(".zshrc");
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
        zsh_profile,
        powershell_profile,
    })
}

fn default_doctor_targets() -> Result<DoctorTargets> {
    let base_dirs =
        BaseDirs::new().context("failed to determine home directory for hunming doctor")?;
    let home_dir = base_dirs.home_dir();

    let bash_rc_profile = home_dir.join(".bashrc");
    let bash_login_profile = home_dir.join(".bash_profile");
    let zsh_profile = home_dir.join(".zshrc");
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

    Ok(DoctorTargets {
        bash_rc_profile,
        bash_login_profile,
        zsh_profile,
        powershell_profile,
    })
}
