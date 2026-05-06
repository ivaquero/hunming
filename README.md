# HunMing

Cross-platform alias manager for Bash and PowerShell.

## What it does

- Stores aliases in `aliases.toml`
- Generates shell functions for Bash, Zsh, and PowerShell
- Writes managed blocks into shell profiles
- Backs up shell profiles before overwriting them

## Usage

- `hunming init`: create the config file, generated scripts, and shell profile blocks.
- `hunming add`: add or update one alias definition.
- `hunming add --profile work|personal`: scope the alias to a work or personal profile.
- `hunming list`: show all configured aliases in a compact table.
- `hunming show <name>`: print one alias in TOML form.
- `hunming apply`: regenerate shell scripts from `aliases.toml`.
- `hunming backup`: back up shell profiles before you edit them.
- `hunming restore`: restore shell profiles from the last backup.
- `hunming doctor`: check the current installation and optionally repair it with `--fix`.

## Profiles

Aliases can be scoped to a work or personal profile:

```toml
[aliases.gs]
command = ["git", "status", "--short"]
profile = "work"
```

Set the active profile with:

```bash
export HUNMING_PROFILE=work
```

Valid values are `work` and `personal`.

## Config

Default config locations:

- Unix: `~/.config/hunming/aliases.toml`
- Windows: `%APPDATA%/hunming/aliases.toml`

Generated scripts live under the `generated/` directory next to the config file.

## Backups

When HunMing updates a shell profile, it writes a sibling backup file named
`<profile>.hunming.bak` first. Use `hunming restore` to restore from that backup.
