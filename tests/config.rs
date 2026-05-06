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
            tags: vec!["git".into(), "status".into()],
            bash: None,
            powershell: None,
            forward_args: true,
            platforms: Vec::new(),
            profile: None,
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
tags = ["list"]
"#;
    fs::write(&paths.config_file, original).expect("seed config should be written");

    let loaded = load_config(&paths).expect("config should load");

    assert_eq!(loaded.version, 1);
    assert_eq!(loaded.aliases["ll"].bash.as_deref(), Some("ls -lah"));
    assert_eq!(loaded.aliases["ll"].tags, vec!["list"]);

    let current = fs::read_to_string(&paths.config_file).expect("config should still be readable");
    assert_eq!(current, original);
}

#[test]
fn load_config_expands_includes_from_relative_files() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));
    paths.ensure_config_dir().expect("config dir should exist");

    let root = r#"
version = 1
include = ["shared.toml", "more.toml"]

[aliases.root]
command = ["git", "status"]
tags = ["git"]
"#;
    let shared = r#"
version = 1

[aliases.shared]
command = ["git", "checkout"]
tags = ["git"]
"#;
    let more = r#"
version = 1

[aliases.more]
bash = "ls -lah"
tags = ["files"]
"#;

    fs::write(&paths.config_file, root).expect("root config should be written");
    fs::write(paths.config_dir.join("shared.toml"), shared)
        .expect("shared config should be written");
    fs::write(paths.config_dir.join("more.toml"), more).expect("more config should be written");

    let config = load_config(&paths).expect("config should load");

    assert_eq!(config.aliases["root"].command, vec!["git", "status"]);
    assert_eq!(config.aliases["root"].tags, vec!["git"]);
    assert_eq!(config.aliases["shared"].command, vec!["git", "checkout"]);
    assert_eq!(config.aliases["shared"].tags, vec!["git"]);
    assert_eq!(config.aliases["more"].bash.as_deref(), Some("ls -lah"));
    assert_eq!(config.aliases["more"].tags, vec!["files"]);
}

#[test]
fn load_config_rejects_duplicate_aliases_across_includes() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));
    paths.ensure_config_dir().expect("config dir should exist");

    let root = r#"
version = 1
include = ["shared.toml"]

[aliases.root]
command = ["git", "status"]
"#;
    let shared = r#"
version = 1

[aliases.root]
command = ["git", "checkout"]
"#;

    fs::write(&paths.config_file, root).expect("root config should be written");
    fs::write(paths.config_dir.join("shared.toml"), shared)
        .expect("shared config should be written");

    let error = load_config(&paths).expect_err("config should reject duplicates");

    assert!(error.to_string().contains("defined more than once"));
}
