use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_version")]
    pub version: u32,

    #[serde(default)]
    pub aliases: BTreeMap<String, Alias>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Alias {
    #[serde(default)]
    pub command: Vec<String>,

    pub bash: Option<String>,
    pub powershell: Option<String>,
}

const fn default_version() -> u32 {
    1
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
            command: Vec::new(),
            bash: None,
            powershell: None,
        }
    }
}
