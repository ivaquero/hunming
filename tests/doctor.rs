use hunming::config::save_config;
use hunming::install::{
    DoctorTargets, bash_managed_block, doctor_with_targets, powershell_managed_block,
    write_shell_profile,
};
use hunming::model::{Alias, Config};
use hunming::paths::AppPaths;
use std::collections::BTreeMap;
use std::fs;
use tempfile::tempdir;

#[test]
fn doctor_reports_expected_checks_and_shadowing() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));
    let targets = DoctorTargets {
        bash_rc_profile: temp.path().join(".bashrc"),
        bash_login_profile: temp.path().join(".bash_profile"),
        zsh_profile: temp.path().join(".zshrc"),
        powershell_profile: temp
            .path()
            .join("PowerShell")
            .join("Microsoft.PowerShell_profile.ps1"),
    };

    let mut aliases = BTreeMap::new();
    aliases.insert(
        "ls".to_string(),
        Alias {
            description: None,
            command: vec!["ls".into()],
            tags: vec!["shell".into()],
            bash: None,
            powershell: None,
            forward_args: true,
            platforms: Vec::new(),
        },
    );
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

    save_config(
        &paths,
        &Config {
            version: 1,
            aliases,
        },
    )
    .expect("config should save");
    hunming::install::apply(&paths, None).expect("apply should succeed");
    write_shell_profile(
        &targets.bash_rc_profile,
        &bash_managed_block(&paths.bash_script),
    )
    .expect("bashrc should write");
    write_shell_profile(&targets.zsh_profile, &bash_managed_block(&paths.zsh_script))
        .expect("zsh profile should write");
    write_shell_profile(
        &targets.powershell_profile,
        &powershell_managed_block(&paths.powershell_script),
    )
    .expect("powershell profile should write");
    fs::write(
        &targets.bash_login_profile,
        "export PATH=\"$HOME/bin:$PATH\"\n",
    )
    .expect("bash profile should seed");

    let report = doctor_with_targets(&paths, &targets, false).expect("doctor should succeed");

    assert!(report.contains("[✓] config file exists"));
    assert!(report.contains("[✓] generated bash file exists"));
    assert!(report.contains("[✓] ~/.bashrc contains humming managed block"));
    assert!(report.contains("[✓] ~/.zshrc contains humming managed block"));
    assert!(report.contains("[!] ~/.bash_profile does not source ~/.bashrc"));
    assert!(report.contains("[✓] PowerShell profile contains humming managed block"));
    assert!(report.contains("[!] alias \"ls\" shadows existing command"));
}

#[test]
fn doctor_fix_creates_missing_files() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));
    let targets = DoctorTargets {
        bash_rc_profile: temp.path().join(".bashrc"),
        bash_login_profile: temp.path().join(".bash_profile"),
        zsh_profile: temp.path().join(".zshrc"),
        powershell_profile: temp
            .path()
            .join("PowerShell")
            .join("Microsoft.PowerShell_profile.ps1"),
    };

    let report = doctor_with_targets(&paths, &targets, true).expect("doctor should succeed");

    assert!(paths.config_file.exists());
    assert!(paths.bash_script.exists());
    assert!(paths.zsh_script.exists());
    assert!(paths.powershell_script.exists());
    assert!(targets.bash_rc_profile.exists());
    assert!(targets.zsh_profile.exists());
    assert!(targets.powershell_profile.exists());
    assert!(report.contains("[✓] config file exists"));
    assert!(report.contains("[✓] ~/.bashrc contains humming managed block"));
    assert!(report.contains("[✓] ~/.zshrc contains humming managed block"));
    assert!(report.contains("[✓] PowerShell profile contains humming managed block"));
}
