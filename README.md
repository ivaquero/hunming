# HunMing

Cross-platform alias manager for Bash and PowerShell.

## What it does

- Stores aliases in `aliases.toml`
- Generates shell functions for Bash, Zsh, and PowerShell
- Writes managed blocks into shell profiles
- Backs up shell profiles before overwriting them

## Usage

- `hunming init`: create `aliases.toml`, generate scripts, and install shell-profile blocks.
- `hunming init --shell bash|zsh|powershell`: limit init to one shell profile.
- `hunming --config /path/to/aliases.toml ...`: use a custom config file path for any command.
- `hunming --profile work|personal ...`: render or install only aliases in that profile.
- `hunming add <name> [--bash ...] [--powershell ...] [--profile ...] [--tag ...] [--force] [-- ...]`: add or update one alias.
- `hunming list`: show all configured aliases in a compact table.
- `hunming show <name>`: print one alias definition in TOML form.
- `hunming apply [--shell bash|zsh|powershell]`: regenerate generated shell scripts.
- `hunming edit`: open the config file in your editor, then reapply scripts.
- `hunming template [--output FILE]`: export a starter `aliases.toml` template.
- `hunming backup [--shell bash|zsh|powershell]`: back up shell profiles before changes.
- `hunming restore [--shell bash|zsh|powershell]`: restore shell profiles from the last backup.
- `hunming doctor [--fix]`: check the current installation and optionally repair it.

## Profiles

Aliases can be scoped to a work or personal profile. Profile-scoped aliases only render when you pass the matching `--profile` flag:

```toml
[aliases.gs]
command = ["git", "status", "--short"]
profile = "work"
```

Valid values are `work` and `personal`.

## Config

Default config locations:

- Unix: `~/.config/hunming/aliases.toml`
- Windows: `%APPDATA%/hunming/aliases.toml`

If you want a different location, pass `--config /path/to/aliases.toml`.

Generated scripts live under the `generated/` directory next to the config file.

## Backups

When HunMing updates a shell profile, it writes a sibling backup file named
`<profile>.hunming.bak` first. Use `hunming restore` to restore from that backup.
