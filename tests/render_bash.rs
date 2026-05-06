use hunming::model::{Alias, Config};
use hunming::render::render_bash;
use std::collections::BTreeMap;

#[test]
fn renders_command_based_bash_functions() {
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

    let rendered = render_bash(&config);

    assert_eq!(rendered, "gs() {\n  git status --short \"$@\"\n}\n");
}

#[test]
fn renders_explicit_bash_functions() {
    let mut aliases = BTreeMap::new();
    aliases.insert(
        "ll".to_string(),
        Alias {
            description: None,
            command: Vec::new(),
            bash: Some("ls -lah".into()),
            powershell: None,
            forward_args: true,
            platforms: Vec::new(),
        },
    );

    let config = Config {
        version: 1,
        aliases,
    };

    let rendered = render_bash(&config);

    assert_eq!(rendered, "ll() {\n  ls -lah \"$@\"\n}\n");
}

#[test]
fn skips_empty_aliases() {
    let mut aliases = BTreeMap::new();
    aliases.insert("empty".to_string(), Alias::default());

    let config = Config {
        version: 1,
        aliases,
    };

    assert!(render_bash(&config).is_empty());
}

#[test]
fn respects_forward_args_flag() {
    let mut aliases = BTreeMap::new();
    aliases.insert(
        "gs".to_string(),
        Alias {
            description: None,
            command: vec!["git".into(), "status".into()],
            bash: None,
            powershell: None,
            forward_args: false,
            platforms: Vec::new(),
        },
    );

    let config = Config {
        version: 1,
        aliases,
    };

    assert_eq!(render_bash(&config), "gs() {\n  git status\n}\n");
}

#[test]
fn filters_other_platforms() {
    let mut aliases = BTreeMap::new();
    aliases.insert(
        "local".to_string(),
        Alias {
            description: None,
            command: vec!["echo".into(), "local".into()],
            bash: None,
            powershell: None,
            forward_args: true,
            platforms: vec![current_platform()],
        },
    );
    aliases.insert(
        "remote".to_string(),
        Alias {
            description: None,
            command: vec!["echo".into(), "remote".into()],
            bash: None,
            powershell: None,
            forward_args: true,
            platforms: vec![other_platform()],
        },
    );

    let config = Config {
        version: 1,
        aliases,
    };

    assert_eq!(render_bash(&config), "local() {\n  echo local \"$@\"\n}\n");
}

fn current_platform() -> hunming::model::Platform {
    #[cfg(target_os = "windows")]
    {
        hunming::model::Platform::Windows
    }
    #[cfg(target_os = "macos")]
    {
        hunming::model::Platform::Macos
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        hunming::model::Platform::Linux
    }
}

fn other_platform() -> hunming::model::Platform {
    match current_platform() {
        hunming::model::Platform::Windows => hunming::model::Platform::Macos,
        hunming::model::Platform::Macos => hunming::model::Platform::Windows,
        hunming::model::Platform::Linux => hunming::model::Platform::Windows,
    }
}
