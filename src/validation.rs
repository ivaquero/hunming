use crate::model::{Alias, Config};
use anyhow::{Result, bail};
use std::collections::BTreeSet;

pub fn validate_config(config: &Config) -> Result<()> {
    for (name, alias) in &config.aliases {
        validate_alias(name, alias)?;
    }

    Ok(())
}

pub fn validate_alias(name: &str, alias: &Alias) -> Result<()> {
    validate_alias_name(name)?;

    let has_command = !alias.command.is_empty();
    let has_bash = alias
        .bash
        .as_ref()
        .is_some_and(|value| !value.trim().is_empty());
    let has_powershell = alias
        .powershell
        .as_ref()
        .is_some_and(|value| !value.trim().is_empty());

    if !has_command && !has_bash && !has_powershell {
        bail!("alias `{name}` must define command, bash, or powershell");
    }

    let mut seen_tags = BTreeSet::new();
    for tag in &alias.tags {
        let trimmed = tag.trim();
        if trimmed.is_empty() {
            bail!("alias `{name}` has an empty tag");
        }
        if trimmed != tag {
            bail!("alias `{name}` has a tag with surrounding whitespace: `{tag}`");
        }
        if !seen_tags.insert(tag) {
            bail!("alias `{name}` has duplicate tag `{tag}`");
        }
    }

    if alias.platforms.is_empty() {
        return Ok(());
    }

    Ok(())
}

pub fn validate_alias_name(name: &str) -> Result<()> {
    let mut chars = name.chars();

    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {}
        _ => bail!("invalid alias name `{name}`: must match ^[A-Za-z_][A-Za-z0-9_-]*$"),
    }

    if chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-') {
        Ok(())
    } else {
        bail!("invalid alias name `{name}`: must match ^[A-Za-z_][A-Za-z0-9_-]*$")
    }
}
