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
            description: None,
            command: Vec::new(),
            tags: vec!["files".into()],
            bash: Some("ls -lah".into()),
            powershell: Some("Get-ChildItem -Force".into()),
            forward_args: true,
            platforms: Vec::new(),
            profile: None,
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
            profile: None,
        },
    );
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

    save_config(
        &paths,
        &Config {
            version: 1,
            aliases,
        },
    )
    .expect("config should save");

    let output = list(&paths).expect("list should succeed");

    assert_eq!(output.lines().count(), 3);
    assert!(output.lines().next().unwrap().starts_with("gco"));
    assert!(output.contains("git, status"));
    assert!(output.contains("files"));
    assert!(output.contains("git checkout"));
}
