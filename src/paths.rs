use anyhow::{Context, Result};
use directories::BaseDirs;
use std::fs;
use std::path::{Path, PathBuf};

const APP_NAME: &str = "hunming";
const GENERATED_DIR_NAME: &str = "generated";
const CONFIG_FILE_NAME: &str = "aliases.toml";
const BASH_FILE_NAME: &str = "bash.sh";
const ZSH_FILE_NAME: &str = "zsh.sh";
const POWERSHELL_FILE_NAME: &str = "powershell.ps1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    pub config_dir: PathBuf,
    pub config_file: PathBuf,
    pub generated_dir: PathBuf,
    pub bash_script: PathBuf,
    pub zsh_script: PathBuf,
    pub powershell_script: PathBuf,
}

impl AppPaths {
    pub fn new() -> Result<Self> {
        let base_dirs =
            BaseDirs::new().context("failed to determine home directory for hunming")?;

        if cfg!(windows) {
            Ok(Self::from_windows_appdata(base_dirs.config_dir()))
        } else {
            Ok(Self::from_unix_home(base_dirs.home_dir()))
        }
    }

    pub fn from_config_dir(config_dir: impl Into<PathBuf>) -> Self {
        let config_dir = config_dir.into();
        let generated_dir = config_dir.join(GENERATED_DIR_NAME);

        Self {
            config_file: config_dir.join(CONFIG_FILE_NAME),
            bash_script: generated_dir.join(BASH_FILE_NAME),
            zsh_script: generated_dir.join(ZSH_FILE_NAME),
            powershell_script: generated_dir.join(POWERSHELL_FILE_NAME),
            generated_dir,
            config_dir,
        }
    }

    pub fn from_unix_home(home_dir: impl AsRef<Path>) -> Self {
        Self::from_config_dir(home_dir.as_ref().join(".config").join(APP_NAME))
    }

    pub fn from_windows_appdata(appdata_dir: impl AsRef<Path>) -> Self {
        Self::from_config_dir(appdata_dir.as_ref().join(APP_NAME))
    }

    pub fn ensure_config_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.config_dir).with_context(|| {
            format!(
                "failed to create config directory at {}",
                self.config_dir.display()
            )
        })?;

        Ok(())
    }

    pub fn ensure_generated_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.generated_dir).with_context(|| {
            format!(
                "failed to create generated directory at {}",
                self.generated_dir.display()
            )
        })?;

        Ok(())
    }
}
