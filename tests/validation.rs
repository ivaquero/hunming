use hunming::config::{load_config, save_config};
use hunming::install::{add, remove};
use hunming::model::{Alias, Config};
use hunming::paths::AppPaths;
use std::collections::BTreeMap;
use std::fs;
use tempfile::tempdir;

#[test]
fn rejects_invalid_alias_names_on_save() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));

    let mut aliases = BTreeMap::new();
    aliases.insert(
        "1bad".to_string(),
        Alias {
            description: None,
            command: vec!["git".into(), "status".into()],
            tags: vec!["git".into()],
            bash: None,
            powershell: None,
            forward_args: true,
            platforms: Vec::new(),
        },
    );

    let error = save_config(
        &paths,
        &Config {
            version: 1,
            aliases,
        },
    )
    .expect_err("save should reject invalid names");

    assert!(error.to_string().contains("invalid alias name"));
}

#[test]
fn rejects_empty_aliases_on_save() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));

    let mut aliases = BTreeMap::new();
    aliases.insert("empty".to_string(), Alias::default());

    let error = save_config(
        &paths,
        &Config {
            version: 1,
            aliases,
        },
    )
    .expect_err("save should reject empty aliases");

    assert!(
        error
            .to_string()
            .contains("must define command, bash, or powershell")
    );
}

#[test]
fn rejects_invalid_aliases_on_load() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));
    paths.ensure_config_dir().expect("config dir should exist");

    let invalid = r#"
version = 1

[aliases.1bad]
command = ["git", "status"]
"#;
    fs::write(&paths.config_file, invalid).expect("config seed should be written");

    let error = load_config(&paths).expect_err("load should reject invalid names");

    assert!(error.to_string().contains("invalid alias name"));
}

#[test]
fn rejects_empty_aliases_on_load() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));
    paths.ensure_config_dir().expect("config dir should exist");

    let invalid = r#"
version = 1

[aliases.empty]
"#;
    fs::write(&paths.config_file, invalid).expect("config seed should be written");

    let error = load_config(&paths).expect_err("load should reject empty aliases");

    assert!(
        error
            .to_string()
            .contains("must define command, bash, or powershell")
    );
}

#[test]
fn add_rejects_invalid_alias_name() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));

    let error = add(
        &paths,
        "1bad".to_string(),
        None,
        None,
        Vec::new(),
        vec!["git".into(), "status".into()],
        false,
    )
    .expect_err("add should reject invalid names");

    assert!(error.to_string().contains("invalid alias name"));
}

#[test]
fn rejects_invalid_tags_on_save() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));

    let mut aliases = BTreeMap::new();
    aliases.insert(
        "gs".to_string(),
        Alias {
            description: None,
            command: vec!["git".into(), "status".into()],
            tags: vec!["".into()],
            bash: None,
            powershell: None,
            forward_args: true,
            platforms: Vec::new(),
        },
    );

    let error = save_config(
        &paths,
        &Config {
            version: 1,
            aliases,
        },
    )
    .expect_err("save should reject empty tags");

    assert!(error.to_string().contains("empty tag"));
}

#[test]
fn remove_rejects_invalid_alias_name() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));

    let error = remove(&paths, "1bad".to_string()).expect_err("remove should reject invalid names");

    assert!(error.to_string().contains("invalid alias name"));
}
