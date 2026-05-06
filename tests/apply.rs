use hunming::install::InitShell;
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
            description: None,
            command: vec!["git".into(), "status".into(), "--short".into()],
            tags: vec!["git".into()],
            bash: None,
            powershell: Some("git status --short".into()),
            forward_args: true,
            platforms: Vec::new(),
        },
    );

    let config = Config {
        version: 1,
        aliases,
    };

    hunming::config::save_config(&paths, &config).expect("config should save");

    let result = apply(&paths, None).expect("apply should succeed");

    assert_eq!(result.bash_script, paths.bash_script);
    assert_eq!(result.zsh_script, paths.zsh_script);
    assert_eq!(result.powershell_script, paths.powershell_script);
    assert_eq!(
        fs::read_to_string(&paths.bash_script).expect("bash script should be readable"),
        "gs() {\n  git status --short \"$@\"\n}\n"
    );
    assert_eq!(
        fs::read_to_string(&paths.zsh_script).expect("zsh script should be readable"),
        "gs() {\n  git status --short \"$@\"\n}\n"
    );
    assert_eq!(
        fs::read_to_string(&paths.powershell_script).expect("powershell script should be readable"),
        "function gs {\n    git status --short @args\n}\n"
    );
}

#[test]
fn apply_writes_empty_scripts_for_default_config() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));

    apply(&paths, None).expect("apply should succeed");

    assert!(paths.config_file.exists());
    assert_eq!(
        fs::read_to_string(&paths.bash_script).expect("bash script should exist"),
        ""
    );
    assert_eq!(
        fs::read_to_string(&paths.zsh_script).expect("zsh script should exist"),
        ""
    );
    assert_eq!(
        fs::read_to_string(&paths.powershell_script).expect("powershell script should exist"),
        ""
    );
}

#[test]
fn apply_with_shell_bash_only_updates_bash_script() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));

    let mut aliases = BTreeMap::new();
    aliases.insert(
        "gs".to_string(),
        Alias {
            description: None,
            command: vec!["git".into(), "status".into(), "--short".into()],
            tags: vec!["git".into()],
            bash: Some("git status --short".into()),
            powershell: Some("git status --short".into()),
            forward_args: true,
            platforms: Vec::new(),
        },
    );

    hunming::config::save_config(
        &paths,
        &Config {
            version: 1,
            aliases,
        },
    )
    .expect("config should save");

    fs::create_dir_all(paths.generated_dir.clone()).expect("generated dir should exist");
    fs::write(&paths.powershell_script, "keep me").expect("seed powershell script");

    apply(&paths, Some(InitShell::Bash)).expect("apply should succeed");

    assert_eq!(
        fs::read_to_string(&paths.bash_script).expect("bash script should be readable"),
        "gs() {\n  git status --short \"$@\"\n}\n"
    );
    assert!(!paths.zsh_script.exists());
    assert_eq!(
        fs::read_to_string(&paths.powershell_script).expect("powershell script should be readable"),
        "keep me"
    );
}

#[test]
fn apply_with_shell_powershell_only_updates_powershell_script() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));

    let mut aliases = BTreeMap::new();
    aliases.insert(
        "gs".to_string(),
        Alias {
            description: None,
            command: vec!["git".into(), "status".into(), "--short".into()],
            tags: vec!["git".into()],
            bash: Some("git status --short".into()),
            powershell: Some("git status --short".into()),
            forward_args: true,
            platforms: Vec::new(),
        },
    );

    hunming::config::save_config(
        &paths,
        &Config {
            version: 1,
            aliases,
        },
    )
    .expect("config should save");

    fs::create_dir_all(paths.generated_dir.clone()).expect("generated dir should exist");
    fs::write(&paths.bash_script, "keep me").expect("seed bash script");

    apply(&paths, Some(InitShell::Powershell)).expect("apply should succeed");

    assert_eq!(
        fs::read_to_string(&paths.bash_script).expect("bash script should be readable"),
        "keep me"
    );
    assert!(!paths.zsh_script.exists());
    assert_eq!(
        fs::read_to_string(&paths.powershell_script).expect("powershell script should be readable"),
        "function gs {\n    git status --short @args\n}\n"
    );
}

#[test]
fn apply_with_shell_zsh_only_updates_zsh_script() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));

    let mut aliases = BTreeMap::new();
    aliases.insert(
        "gs".to_string(),
        Alias {
            description: None,
            command: vec!["git".into(), "status".into(), "--short".into()],
            tags: vec!["git".into()],
            bash: Some("git status --short".into()),
            powershell: Some("git status --short".into()),
            forward_args: true,
            platforms: Vec::new(),
        },
    );

    hunming::config::save_config(
        &paths,
        &Config {
            version: 1,
            aliases,
        },
    )
    .expect("config should save");

    fs::create_dir_all(paths.generated_dir.clone()).expect("generated dir should exist");
    fs::write(&paths.bash_script, "keep me").expect("seed bash script");
    fs::write(&paths.powershell_script, "keep me").expect("seed powershell script");

    apply(&paths, Some(InitShell::Zsh)).expect("apply should succeed");

    assert_eq!(
        fs::read_to_string(&paths.bash_script).expect("bash script should be readable"),
        "keep me"
    );
    assert_eq!(
        fs::read_to_string(&paths.zsh_script).expect("zsh script should be readable"),
        "gs() {\n  git status --short \"$@\"\n}\n"
    );
    assert_eq!(
        fs::read_to_string(&paths.powershell_script).expect("powershell script should be readable"),
        "keep me"
    );
}
