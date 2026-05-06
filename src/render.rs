use crate::model::{Alias, Config};
use std::fmt::Write as _;

pub fn render_bash(config: &Config) -> String {
    let mut output = String::new();
    let mut first = true;

    for (name, alias) in &config.aliases {
        if let Some(function) = render_bash_function(name, alias) {
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

pub fn render_powershell(config: &Config) -> String {
    let mut output = String::new();
    let mut first = true;

    for (name, alias) in &config.aliases {
        if let Some(function) = render_powershell_function(name, alias) {
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

fn render_bash_function(name: &str, alias: &Alias) -> Option<String> {
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
    writeln!(&mut output, "  {command} \"$@\"").ok()?;
    output.push('}');
    output.push('\n');

    Some(output)
}

fn render_powershell_function(name: &str, alias: &Alias) -> Option<String> {
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
    writeln!(&mut output, "    {command} @args").ok()?;
    output.push('}');
    output.push('\n');

    Some(output)
}
