use crate::model::{Alias, Config};
use anyhow::{bail, Result};

pub fn validate_config(config: &Config) -> Result<()> {
    for (name, alias) in &config.aliases {
        validate_alias(name, alias)?;
    }

    Ok(())
}

pub fn validate_alias(name: &str, alias: &Alias) -> Result<()> {
    validate_alias_name(name)?;

    let has_command = !alias.command.is_empty();
    let has_bash = alias.bash.as_ref().is_some_and(|value| !value.trim().is_empty());
    let has_powershell = alias
        .powershell
        .as_ref()
        .is_some_and(|value| !value.trim().is_empty());

    if !has_command && !has_bash && !has_powershell {
        bail!("alias `{name}` must define command, bash, or powershell");
    }

    Ok(())
}

pub fn validate_alias_name(name: &str) -> Result<()> {
    let mut chars = name.chars();

    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {}
        _ => bail!(
            "invalid alias name `{name}`: must match ^[A-Za-z_][A-Za-z0-9_-]*$"
        ),
    }

    if chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-') {
        Ok(())
    } else {
        bail!(
            "invalid alias name `{name}`: must match ^[A-Za-z_][A-Za-z0-9_-]*$"
        )
    }
}
