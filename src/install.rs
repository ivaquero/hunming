use crate::config::load_config;
use crate::fs::atomic_write;
use crate::paths::AppPaths;
use crate::render::{render_bash, render_powershell};
use anyhow::Result;
use std::path::Path;
use std::path::PathBuf;

pub const MANAGED_BLOCK_START: &str = "# >>> hunming init >>>";
pub const MANAGED_BLOCK_END: &str = "# <<< hunming init <<<";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyResult {
    pub bash_script: PathBuf,
    pub powershell_script: PathBuf,
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
