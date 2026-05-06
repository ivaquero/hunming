use hunming::config::{default_config, load_config, save_config};
use hunming::model::{Alias, Config};
use hunming::paths::AppPaths;
use std::collections::BTreeMap;
use std::fs;
use tempfile::tempdir;

#[test]
fn default_config_is_minimal() {
    let config = default_config();

    assert_eq!(config.version, 1);
    assert!(config.aliases.is_empty());
}

#[test]
fn load_config_creates_default_file_when_missing() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));

    let config = load_config(&paths).expect("config should load");

    assert_eq!(config, default_config());
    assert!(paths.config_file.exists());

    let content = fs::read_to_string(&paths.config_file).expect("config file should be readable");
    assert!(content.contains("version = 1"));
}

#[test]
fn save_and_load_roundtrip_config() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));

    let mut aliases = BTreeMap::new();
    aliases.insert(
        "gs".to_string(),
        Alias {
            description: None,
            command: vec!["git".into(), "status".into(), "--short".into()],
            bash: None,
            powershell: None,
            forward_args: true,
            platforms: Vec::new(),
        },
    );

    let config = Config {
        version: 1,
        aliases,
    };

    save_config(&paths, &config).expect("config should save");
    let loaded = load_config(&paths).expect("config should load");

    assert_eq!(loaded, config);
}

#[test]
fn load_config_does_not_overwrite_existing_content() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));
    paths.ensure_config_dir().expect("config dir should exist");

    let original = r#"
version = 1

[aliases.ll]
bash = "ls -lah"
"#;
    fs::write(&paths.config_file, original).expect("seed config should be written");

    let loaded = load_config(&paths).expect("config should load");

    assert_eq!(loaded.version, 1);
    assert_eq!(loaded.aliases["ll"].bash.as_deref(), Some("ls -lah"));

    let current = fs::read_to_string(&paths.config_file).expect("config should still be readable");
    assert_eq!(current, original);
}
