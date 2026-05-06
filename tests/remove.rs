use hunming::config::{load_config, save_config};
use hunming::install::remove;
use hunming::model::{Alias, Config};
use hunming::paths::AppPaths;
use std::collections::BTreeMap;
use std::fs;
use tempfile::tempdir;

#[test]
fn remove_deletes_alias_and_updates_scripts() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));

    let mut aliases = BTreeMap::new();
    aliases.insert(
        "gs".to_string(),
        Alias {
            description: None,
            command: vec!["git".into(), "status".into(), "--short".into()],
            tags: vec!["git".into()],
            bash: None,
            powershell: None,
            forward_args: true,
            platforms: Vec::new(),
        },
    );
    aliases.insert(
        "gco".to_string(),
        Alias {
            description: None,
            command: vec!["git".into(), "checkout".into()],
            tags: vec!["git".into()],
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

    remove(&paths, "gs".to_string()).expect("remove should succeed");

    let config = load_config(&paths).expect("config should load");
    assert!(!config.aliases.contains_key("gs"));
    assert!(config.aliases.contains_key("gco"));
    assert_eq!(
        fs::read_to_string(&paths.bash_script).expect("bash script should exist"),
        "gco() {\n  git checkout \"$@\"\n}\n"
    );
    assert_eq!(
        fs::read_to_string(&paths.powershell_script).expect("powershell script should exist"),
        "function gco {\n    git checkout @args\n}\n"
    );
}

#[test]
fn remove_rejects_missing_alias() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));

    let error = remove(&paths, "gs".to_string()).expect_err("remove should reject missing alias");

    assert!(error.to_string().contains("does not exist"));
}
