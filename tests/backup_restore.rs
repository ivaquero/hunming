use hunming::install::{
    InitShell, InitTargets, backup_with_targets, bash_managed_block, restore_with_targets,
    write_shell_profile,
};
use std::fs;
use tempfile::tempdir;

#[test]
fn write_shell_profile_creates_a_backup_before_overwriting() {
    let temp = tempdir().expect("temp dir should be created");
    let profile = temp.path().join(".bashrc");
    let original = "export PATH=\"$HOME/bin:$PATH\"\n";
    fs::write(&profile, original).expect("profile should be seeded");

    write_shell_profile(
        &profile,
        &bash_managed_block("/tmp/hunming/generated/bash.sh"),
    )
    .expect("profile should be written");

    let backup = temp.path().join(".bashrc.hunming.bak");
    assert_eq!(
        fs::read_to_string(&backup).expect("backup should be readable"),
        original
    );
    assert!(
        fs::read_to_string(&profile)
            .expect("profile should be readable")
            .contains("hunming init")
    );
}

#[test]
fn backup_and_restore_selected_profiles() {
    let temp = tempdir().expect("temp dir should be created");
    let targets = InitTargets {
        bash_profile: temp.path().join(".bashrc"),
        zsh_profile: temp.path().join(".zshrc"),
        powershell_profile: temp
            .path()
            .join("PowerShell")
            .join("Microsoft.PowerShell_profile.ps1"),
    };

    fs::write(&targets.bash_profile, "bash before\n").expect("bash profile should be seeded");
    fs::write(&targets.zsh_profile, "zsh before\n").expect("zsh profile should be seeded");
    fs::write(&targets.powershell_profile, "powershell before\n")
        .expect("powershell profile should be seeded");

    let backup =
        backup_with_targets(&targets, Some(InitShell::Bash)).expect("backup should succeed");

    assert_eq!(
        backup.profile_paths,
        vec![temp.path().join(".bashrc.hunming.bak")]
    );

    fs::write(&targets.bash_profile, "bash after\n").expect("bash profile should be changed");

    let restored =
        restore_with_targets(&targets, Some(InitShell::Bash)).expect("restore should succeed");

    assert_eq!(restored.profile_paths, vec![targets.bash_profile.clone()]);
    assert_eq!(
        fs::read_to_string(&targets.bash_profile).expect("bash profile should be readable"),
        "bash before\n"
    );
}
