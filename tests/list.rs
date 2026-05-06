use hunming::config::save_config;
use hunming::install::list;
use hunming::model::{Alias, Config};
use hunming::paths::AppPaths;
use std::collections::BTreeMap;
use tempfile::tempdir;

#[test]
fn list_reports_empty_configuration() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));

    let output = list(&paths).expect("list should succeed");

    assert_eq!(output, "No aliases configured.\n");
}

#[test]
fn list_sorts_aliases_and_shows_kind() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));

    let mut aliases = BTreeMap::new();
    aliases.insert(
        "ll".to_string(),
        Alias {
            command: Vec::new(),
            bash: Some("ls -lah".into()),
            powershell: Some("Get-ChildItem -Force".into()),
        },
    );
    aliases.insert(
        "gco".to_string(),
        Alias {
            command: vec!["git".into(), "checkout".into()],
            bash: None,
            powershell: None,
        },
    );
    aliases.insert(
        "gs".to_string(),
        Alias {
            command: vec!["git".into(), "status".into(), "--short".into()],
            bash: None,
            powershell: None,
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

    let output = list(&paths).expect("list should succeed");

    assert_eq!(
        output,
        "gco  command      git checkout\n\
gs   command      git status --short\n\
ll   shell        bash: ls -lah | powershell: Get-ChildItem -Force\n"
    );
}
