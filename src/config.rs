use crate::fs::atomic_write;
use crate::model::Config;
use crate::paths::AppPaths;
use crate::validation::validate_config;
use anyhow::{Context, Result};
use std::fs;

pub fn default_config() -> Config {
    Config::default()
}

pub fn load_config(paths: &AppPaths) -> Result<Config> {
    if !paths.config_file.exists() {
        let config = default_config();
        save_config(paths, &config)?;
        return Ok(config);
    }

    let content = fs::read_to_string(&paths.config_file).with_context(|| {
        format!(
            "failed to read config file at {}",
            paths.config_file.display()
        )
    })?;

    let config: Config = toml::from_str(&content).with_context(|| {
        format!(
            "failed to parse config file at {}",
            paths.config_file.display()
        )
    })?;

    validate_config(&config)?;

    Ok(config)
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
