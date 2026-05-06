use hunming::install::apply;
use hunming::model::{Alias, Config};
use hunming::paths::AppPaths;
use std::collections::BTreeMap;
use std::fs;
use tempfile::tempdir;

#[test]
fn apply_creates_scripts_for_existing_config() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));

    let mut aliases = BTreeMap::new();
    aliases.insert(
        "gs".to_string(),
        Alias {
            command: vec!["git".into(), "status".into(), "--short".into()],
            bash: None,
            powershell: Some("git status --short".into()),
        },
    );

    let config = Config {
        version: 1,
        aliases,
    };

    hunming::config::save_config(&paths, &config).expect("config should save");

    let result = apply(&paths).expect("apply should succeed");

    assert_eq!(result.bash_script, paths.bash_script);
    assert_eq!(result.powershell_script, paths.powershell_script);
    assert_eq!(
        fs::read_to_string(&paths.bash_script).expect("bash script should be readable"),
        "gs() {\n  git status --short \"$@\"\n}\n"
    );
    assert_eq!(
        fs::read_to_string(&paths.powershell_script)
            .expect("powershell script should be readable"),
        "function gs {\n    git status --short @args\n}\n"
    );
}

#[test]
fn apply_writes_empty_scripts_for_default_config() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));

    apply(&paths).expect("apply should succeed");

    assert!(paths.config_file.exists());
    assert_eq!(fs::read_to_string(&paths.bash_script).expect("bash script should exist"), "");
    assert_eq!(
        fs::read_to_string(&paths.powershell_script).expect("powershell script should exist"),
        ""
    );
}
