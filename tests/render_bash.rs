use hunming::model::{Alias, Config};
use hunming::render::render_bash;
use std::collections::BTreeMap;

#[test]
fn renders_command_based_bash_functions() {
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

    let rendered = render_bash(&config);

    assert_eq!(rendered, "gs() {\n  git status --short \"$@\"\n}\n");
}

#[test]
fn renders_explicit_bash_functions() {
    let mut aliases = BTreeMap::new();
    aliases.insert(
        "ll".to_string(),
        Alias {
            command: Vec::new(),
            bash: Some("ls -lah".into()),
            powershell: None,
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
