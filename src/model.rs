use clap::ValueEnum;
use std::env;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
#[value(rename_all = "lower")]
pub enum Profile {
    Work,
    Personal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Alias {
    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub command: Vec<String>,

    #[serde(default)]
    pub tags: Vec<String>,

    pub bash: Option<String>,
    pub powershell: Option<String>,

    #[serde(default = "default_forward_args")]
    pub forward_args: bool,

    #[serde(default)]
    pub platforms: Vec<Platform>,

    #[serde(default)]
    pub profile: Option<Profile>,
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
            tags: Vec::new(),
            bash: None,
            powershell: None,
            forward_args: true,
            platforms: Vec::new(),
            profile: None,
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

    pub fn is_active_for_current_profile(&self) -> bool {
        self.is_active_for_profile(Profile::current())
    }

    pub fn is_active_for_profile(&self, profile: Option<Profile>) -> bool {
        match self.profile {
            None => true,
            Some(expected) => match profile {
                None => true,
                Some(current) => current == expected,
            },
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active_for_current_platform() && self.is_active_for_current_profile()
    }
}

impl Profile {
    pub fn current() -> Option<Self> {
        let value = env::var("HUNMING_PROFILE").ok()?;
        let value = value.trim().to_ascii_lowercase();

        match value.as_str() {
            "work" => Some(Self::Work),
            "personal" => Some(Self::Personal),
            _ => None,
        }
    }
}
