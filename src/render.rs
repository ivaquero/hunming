use crate::model::{Alias, Config, Profile};
use std::fmt::Write as _;

pub fn render_bash(config: &Config) -> String {
    render_bash_with_profile(config, None)
}

pub fn render_bash_with_profile(config: &Config, profile: Option<Profile>) -> String {
    let mut output = String::new();
    let mut first = true;

    for (name, alias) in &config.aliases {
        if let Some(function) = render_bash_function(name, alias, profile) {
            if !first {
                output.push('\n');
                output.push('\n');
            }
            first = false;
            output.push_str(&function);
        }
    }

    output
}

pub fn render_zsh(config: &Config) -> String {
    render_bash(config)
}

pub fn render_powershell(config: &Config) -> String {
    render_powershell_with_profile(config, None)
}

pub fn render_powershell_with_profile(config: &Config, profile: Option<Profile>) -> String {
    let mut output = String::new();
    let mut first = true;

    for (name, alias) in &config.aliases {
        if let Some(function) = render_powershell_function(name, alias, profile) {
            if !first {
                output.push('\n');
                output.push('\n');
            }
            first = false;
            output.push_str(&function);
        }
    }

    output
}

fn render_bash_function(name: &str, alias: &Alias, profile: Option<Profile>) -> Option<String> {
    if !alias.is_active_for_current_platform() || !alias.is_active_for_profile(profile) {
        return None;
    }

    let command = if !alias.command.is_empty() {
        Some(alias.command.join(" "))
    } else {
        alias.bash.as_ref().map(|value| value.trim().to_string())
    }?;

    if command.trim().is_empty() {
        return None;
    }

    let mut output = String::new();
    writeln!(&mut output, "{name}() {{").ok()?;
    if alias.forward_args {
        writeln!(&mut output, "  {command} \"$@\"").ok()?;
    } else {
        writeln!(&mut output, "  {command}").ok()?;
    }
    output.push('}');
    output.push('\n');

    Some(output)
}

fn render_powershell_function(
    name: &str,
    alias: &Alias,
    profile: Option<Profile>,
) -> Option<String> {
    if !alias.is_active_for_current_platform() || !alias.is_active_for_profile(profile) {
        return None;
    }

    let command = alias
        .powershell
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            if alias.command.is_empty() {
                None
            } else {
                Some(alias.command.join(" "))
            }
        })?;

    if command.trim().is_empty() {
        return None;
    }

    let mut output = String::new();
    writeln!(&mut output, "function {name} {{").ok()?;
    if alias.forward_args {
        writeln!(&mut output, "    {command} @args").ok()?;
    } else {
        writeln!(&mut output, "    {command}").ok()?;
    }
    output.push('}');
    output.push('\n');

    Some(output)
}
