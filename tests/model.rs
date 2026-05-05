use hunming::model::{Alias, Config};
use std::collections::BTreeMap;

#[test]
fn deserialize_sample_config() {
    let input = r#"
version = 1

[aliases.gs]
command = ["git", "status", "--short"]

[aliases.gco]
command = ["git", "checkout"]

[aliases.ll]
bash = "ls -lah"
powershell = "Get-ChildItem -Force"
"#;

    let config: Config = toml::from_str(input).expect("config should deserialize");

    assert_eq!(config.version, 1);
    assert_eq!(
        config.aliases["gs"].command,
        vec!["git", "status", "--short"]
    );
    assert_eq!(config.aliases["gco"].command, vec!["git", "checkout"]);
    assert_eq!(config.aliases["ll"].bash.as_deref(), Some("ls -lah"));
    assert_eq!(
        config.aliases["ll"].powershell.as_deref(),
        Some("Get-ChildItem -Force")
    );
}

#[test]
fn roundtrip_config() {
    let mut aliases = BTreeMap::new();
    aliases.insert(
        "gs".to_string(),
        Alias {
            command: vec!["git".into(), "status".into(), "--short".into()],
            bash: None,
            powershell: None,
        },
    );

    let config = Config {
        version: 1,
        aliases,
    };

    let toml = toml::to_string(&config).expect("config should serialize");
    let decoded: Config = toml::from_str(&toml).expect("config should deserialize");

    assert_eq!(decoded, config);
}

#[test]
fn empty_command_is_allowed() {
    let input = r#"
version = 1

[aliases.empty]
"#;

    let config: Config = toml::from_str(input).expect("config should deserialize");
    let alias = config.aliases.get("empty").expect("alias should exist");

    assert!(alias.command.is_empty());
    assert_eq!(alias.bash, None);
    assert_eq!(alias.powershell, None);
}
