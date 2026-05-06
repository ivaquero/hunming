use hunming::config::{load_config, save_config};
use hunming::install::add;
use hunming::model::{Alias, Config};
use hunming::paths::AppPaths;
use std::collections::BTreeMap;
use std::fs;
use tempfile::tempdir;

#[test]
fn add_creates_new_alias_and_updates_scripts() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));

    add(
        &paths,
        "gs".to_string(),
        vec!["git".into(), "status".into(), "--short".into()],
        false,
    )
    .expect("add should succeed");

    let config = load_config(&paths).expect("config should load");
    assert_eq!(
        config.aliases["gs"].command,
        vec!["git", "status", "--short"]
    );
    assert_eq!(
        fs::read_to_string(&paths.bash_script).expect("bash script should exist"),
        "gs() {\n  git status --short \"$@\"\n}\n"
    );
    assert_eq!(
        fs::read_to_string(&paths.powershell_script).expect("powershell script should exist"),
        "function gs {\n    git status --short @args\n}\n"
    );
}

#[test]
fn add_rejects_existing_alias_without_force() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));

    let mut aliases = BTreeMap::new();
    aliases.insert(
        "gs".to_string(),
        Alias {
            description: None,
            command: vec!["git".into(), "status".into()],
            bash: None,
            powershell: None,
            forward_args: true,
            platforms: Vec::new(),
        },
    );

    save_config(
        &paths,
        &Config {
            version: 1,
            aliases,
        },
    )
    .expect("config should save");

    let error = add(
        &paths,
        "gs".to_string(),
        vec!["git".into(), "status".into(), "--short".into()],
        false,
    )
    .expect_err("add should reject duplicates");

    assert!(error.to_string().contains("use --force"));

    let config = load_config(&paths).expect("config should load");
    assert_eq!(config.aliases["gs"].command, vec!["git", "status"]);
}

#[test]
fn add_replaces_existing_alias_with_force() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));

    let mut aliases = BTreeMap::new();
    aliases.insert(
        "gs".to_string(),
        Alias {
            description: None,
            command: vec!["git".into(), "status".into()],
            bash: None,
            powershell: None,
            forward_args: true,
            platforms: Vec::new(),
        },
    );

    save_config(
        &paths,
        &Config {
            version: 1,
            aliases,
        },
    )
    .expect("config should save");

    add(
        &paths,
        "gs".to_string(),
        vec!["git".into(), "status".into(), "--short".into()],
        true,
    )
    .expect("add should overwrite existing alias");

    let config = load_config(&paths).expect("config should load");
    assert_eq!(
        config.aliases["gs"].command,
        vec!["git", "status", "--short"]
    );
    assert_eq!(
        fs::read_to_string(&paths.bash_script).expect("bash script should exist"),
        "gs() {\n  git status --short \"$@\"\n}\n"
    );
}
