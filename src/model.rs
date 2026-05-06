use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_version")]
    pub version: u32,

    #[serde(default)]
    pub aliases: BTreeMap<String, Alias>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Windows,
    Macos,
    Linux,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Alias {
    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub command: Vec<String>,

    pub bash: Option<String>,
    pub powershell: Option<String>,

    #[serde(default = "default_forward_args")]
    pub forward_args: bool,

    #[serde(default)]
    pub platforms: Vec<Platform>,
}

const fn default_version() -> u32 {
    1
}

const fn default_forward_args() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: default_version(),
            aliases: BTreeMap::new(),
        }
    }
}

impl Default for Alias {
    fn default() -> Self {
        Self {
            description: None,
            command: Vec::new(),
            bash: None,
            powershell: None,
            forward_args: true,
            platforms: Vec::new(),
        }
    }
}

impl Platform {
    pub fn current() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::Macos
        } else {
            Self::Linux
        }
    }
}

impl Alias {
    pub fn is_active_for_current_platform(&self) -> bool {
        self.platforms.is_empty() || self.platforms.contains(&Platform::current())
    }
}
