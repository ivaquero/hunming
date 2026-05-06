use hunming::model::{Alias, Config};
use hunming::render::render_powershell;
use std::collections::BTreeMap;

#[test]
fn renders_command_based_powershell_functions() {
    let mut aliases = BTreeMap::new();
    aliases.insert(
        "gs".to_string(),
        Alias {
            command: vec!["git".into(), "status".into(), "--short".into()],
            bash: None,
            powershell: None,
        },
    );

    let config = Config {
        version: 1,
        aliases,
    };

    let rendered = render_powershell(&config);

    assert_eq!(rendered, "function gs {\n    git status --short @args\n}\n");
}

#[test]
fn renders_explicit_powershell_functions() {
    let mut aliases = BTreeMap::new();
    aliases.insert(
        "ll".to_string(),
        Alias {
            command: Vec::new(),
            bash: None,
            powershell: Some("Get-ChildItem -Force".into()),
        },
    );

    let config = Config {
        version: 1,
        aliases,
    };

    let rendered = render_powershell(&config);

    assert_eq!(rendered, "function ll {\n    Get-ChildItem -Force @args\n}\n");
}

#[test]
fn falls_back_to_command_when_powershell_is_missing() {
    let mut aliases = BTreeMap::new();
    aliases.insert(
        "gco".to_string(),
        Alias {
            command: vec!["git".into(), "checkout".into()],
            bash: None,
            powershell: None,
        },
    );

    let config = Config {
        version: 1,
        aliases,
    };

    assert_eq!(
        render_powershell(&config),
        "function gco {\n    git checkout @args\n}\n"
    );
}

#[test]
fn skips_empty_aliases() {
    let mut aliases = BTreeMap::new();
    aliases.insert("empty".to_string(), Alias::default());

    let config = Config {
        version: 1,
        aliases,
    };

    assert!(render_powershell(&config).is_empty());
}
