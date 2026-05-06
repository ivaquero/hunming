use hunming::model::{Alias, Config, Profile};
use hunming::render::render_powershell_with_profile;
use std::collections::BTreeMap;

#[test]
fn renders_command_based_powershell_functions() {
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
            profile: None,
        },
    );

    let config = Config {
        version: 1,
        aliases,
    };

    let rendered = render_powershell_with_profile(&config, None);

    assert_eq!(rendered, "function gs {\n    git status --short @args\n}\n");
}

#[test]
fn renders_explicit_powershell_functions() {
    let mut aliases = BTreeMap::new();
    aliases.insert(
        "ll".to_string(),
        Alias {
            description: None,
            command: Vec::new(),
            tags: vec!["files".into()],
            bash: None,
            powershell: Some("Get-ChildItem -Force".into()),
            forward_args: true,
            platforms: Vec::new(),
            profile: None,
        },
    );

    let config = Config {
        version: 1,
        aliases,
    };

    let rendered = render_powershell_with_profile(&config, None);

    assert_eq!(
        rendered,
        "function ll {\n    Get-ChildItem -Force @args\n}\n"
    );
}

#[test]
fn falls_back_to_command_when_powershell_is_missing() {
    let mut aliases = BTreeMap::new();
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

    let config = Config {
        version: 1,
        aliases,
    };

    assert_eq!(
        render_powershell_with_profile(&config, None),
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

    assert!(render_powershell_with_profile(&config, None).is_empty());
}

#[test]
fn respects_forward_args_flag() {
    let mut aliases = BTreeMap::new();
    aliases.insert(
        "gs".to_string(),
        Alias {
            description: None,
            command: vec!["git".into(), "status".into()],
            tags: vec!["git".into()],
            bash: None,
            powershell: None,
            forward_args: false,
            platforms: Vec::new(),
            profile: None,
        },
    );

    let config = Config {
        version: 1,
        aliases,
    };

    assert_eq!(
        render_powershell_with_profile(&config, None),
        "function gs {\n    git status\n}\n"
    );
}

#[test]
fn filters_other_platforms() {
    let mut aliases = BTreeMap::new();
    aliases.insert(
        "local".to_string(),
        Alias {
            description: None,
            command: vec!["echo".into(), "local".into()],
            tags: vec!["local".into()],
            bash: None,
            powershell: None,
            forward_args: true,
            platforms: vec![current_platform()],
            profile: None,
        },
    );
    aliases.insert(
        "remote".to_string(),
        Alias {
            description: None,
            command: vec!["echo".into(), "remote".into()],
            tags: vec!["remote".into()],
            bash: None,
            powershell: None,
            forward_args: true,
            platforms: vec![other_platform()],
            profile: None,
        },
    );

    let config = Config {
        version: 1,
        aliases,
    };

    assert_eq!(
        render_powershell_with_profile(&config, None),
        "function local {\n    echo local @args\n}\n"
    );
}

#[test]
fn filters_other_profiles() {
    let mut aliases = BTreeMap::new();
    aliases.insert(
        "work".to_string(),
        Alias {
            description: None,
            command: vec!["echo".into(), "work".into()],
            tags: vec!["work".into()],
            bash: None,
            powershell: None,
            forward_args: true,
            platforms: Vec::new(),
            profile: Some(Profile::Work),
        },
    );
    aliases.insert(
        "personal".to_string(),
        Alias {
            description: None,
            command: vec!["echo".into(), "personal".into()],
            tags: vec!["personal".into()],
            bash: None,
            powershell: None,
            forward_args: true,
            platforms: Vec::new(),
            profile: Some(Profile::Personal),
        },
    );

    let config = Config {
        version: 1,
        aliases,
    };

    assert_eq!(
        render_powershell_with_profile(&config, Some(Profile::Work)),
        "function work {\n    echo work @args\n}\n"
    );
    assert_eq!(
        render_powershell_with_profile(&config, Some(Profile::Personal)),
        "function personal {\n    echo personal @args\n}\n"
    );
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
