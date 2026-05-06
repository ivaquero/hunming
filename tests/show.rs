use hunming::config::save_config;
use hunming::install::show;
use hunming::model::{Alias, Config, Platform, Profile};
use hunming::paths::AppPaths;
use std::collections::BTreeMap;
use tempfile::tempdir;

#[test]
fn show_renders_alias_definition() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));

    let mut aliases = BTreeMap::new();
    aliases.insert(
        "ll".to_string(),
        Alias {
            description: Some("List files".into()),
            command: vec!["ls".into(), "-lah".into()],
            tags: vec!["files".into()],
            bash: Some("ls -lah".into()),
            powershell: Some("Get-ChildItem -Force".into()),
            forward_args: false,
            platforms: vec![Platform::Linux],
            profile: Some(Profile::Work),
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

    let output = show(&paths, "ll".to_string()).expect("show should succeed");

    assert_eq!(
        output,
        "[aliases.ll]\n\
description = \"List files\"\n\
command = [\"ls\", \"-lah\"]\n\
tags = [\"files\"]\n\
bash = \"ls -lah\"\n\
powershell = \"Get-ChildItem -Force\"\n\
forward_args = false\n\
profile = \"work\"\n\
platforms = [\"linux\"]\n"
    );
}

#[test]
fn show_rejects_missing_alias() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));

    let error = show(&paths, "gs".to_string()).expect_err("show should reject missing aliases");

    assert!(error.to_string().contains("does not exist"));
}
