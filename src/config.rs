use crate::fs::atomic_write;
use crate::model::{Alias, Config};
use crate::paths::AppPaths;
use crate::validation::validate_config;
use anyhow::bail;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn default_config() -> Config {
    Config::default()
}

pub fn load_config(paths: &AppPaths) -> Result<Config> {
    if !paths.config_file.exists() {
        let config = default_config();
        save_config(paths, &config)?;
        return Ok(config);
    }

    load_config_from_path(&paths.config_file)
}

pub fn load_config_from_path(config_file: &Path) -> Result<Config> {
    load_config_from_path_inner(config_file, &mut Vec::new())
}

pub fn save_config(paths: &AppPaths, config: &Config) -> Result<()> {
    paths.ensure_config_dir()?;
    validate_config(config)?;

    let mut content = toml::to_string_pretty(config).context("failed to serialize config")?;
    if !content.ends_with('\n') {
        content.push('\n');
    }

    atomic_write(&paths.config_file, &content)?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct ConfigDocument {
    #[serde(default = "default_version")]
    version: u32,

    #[serde(default)]
    include: Vec<String>,

    #[serde(default)]
    aliases: BTreeMap<String, Alias>,
}

fn load_config_from_path_inner(config_file: &Path, stack: &mut Vec<PathBuf>) -> Result<Config> {
    let canonical = fs::canonicalize(config_file)
        .with_context(|| format!("failed to resolve config file at {}", config_file.display()))?;

    if stack.contains(&canonical) {
        bail!("circular include detected at {}", config_file.display());
    }

    stack.push(canonical);

    let result = (|| -> Result<Config> {
        let content = fs::read_to_string(config_file)
            .with_context(|| format!("failed to read config file at {}", config_file.display()))?;

        let document: ConfigDocument = toml::from_str(&content)
            .with_context(|| format!("failed to parse config file at {}", config_file.display()))?;

        let mut config = Config {
            version: document.version,
            aliases: BTreeMap::new(),
        };

        for include in document.include {
            let include_file = resolve_include_path(config_file, &include);
            let included =
                load_config_from_path_inner(&include_file, stack).with_context(|| {
                    format!(
                        "failed to load included config file at {}",
                        include_file.display()
                    )
                })?;

            if included.version != config.version {
                bail!(
                    "included config file at {} uses version {} but expected {}",
                    include_file.display(),
                    included.version,
                    config.version
                );
            }

            merge_aliases(&mut config.aliases, included.aliases, &include_file)?;
        }

        merge_aliases(&mut config.aliases, document.aliases, config_file)?;
        validate_config(&config)?;

        Ok(config)
    })();

    stack.pop();
    result
}

fn merge_aliases(
    target: &mut BTreeMap<String, Alias>,
    source: BTreeMap<String, Alias>,
    source_file: &Path,
) -> Result<()> {
    for (name, alias) in source {
        if target.insert(name.clone(), alias).is_some() {
            bail!(
                "alias `{name}` is defined more than once while loading {}",
                source_file.display()
            );
        }
    }

    Ok(())
}

fn resolve_include_path(config_file: &Path, include: &str) -> PathBuf {
    let include_path = Path::new(include);
    if include_path.is_absolute() {
        include_path.to_path_buf()
    } else {
        config_file
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(include_path)
    }
}

const fn default_version() -> u32 {
    1
}
