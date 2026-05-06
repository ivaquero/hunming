use hunming::install::{
    InitShell, InitTargets, MANAGED_BLOCK_END, MANAGED_BLOCK_START, bash_managed_block,
    init_with_targets, init_with_targets_and_shell, powershell_managed_block,
};
use hunming::paths::AppPaths;
use std::fs;
use tempfile::tempdir;

#[test]
fn init_creates_config_generated_and_profiles() {
    let temp = tempdir().expect("temp dir should be created");
    let config_dir = temp.path().join("hunming");
    let paths = AppPaths::from_config_dir(&config_dir);
    let targets = InitTargets {
        bash_profile: temp.path().join(".bashrc"),
        zsh_profile: temp.path().join(".zshrc"),
        powershell_profile: temp
            .path()
            .join("PowerShell")
            .join("Microsoft.PowerShell_profile.ps1"),
    };

    let result = init_with_targets(&paths, &targets).expect("init should succeed");

    assert_eq!(result.config_file, paths.config_file);
    assert!(paths.config_file.exists());
    assert!(paths.bash_script.exists());
    assert!(paths.zsh_script.exists());
    assert!(paths.powershell_script.exists());

    let bash_profile = fs::read_to_string(&targets.bash_profile).expect("bash profile readable");
    assert!(bash_profile.contains(MANAGED_BLOCK_START));
    assert!(bash_profile.contains(&paths.bash_script.display().to_string()));
    assert_eq!(bash_profile, bash_managed_block(&paths.bash_script));

    let powershell_profile =
        fs::read_to_string(&targets.powershell_profile).expect("powershell profile readable");
    assert!(powershell_profile.contains(MANAGED_BLOCK_START));
    assert!(powershell_profile.contains(&paths.powershell_script.display().to_string()));
    assert_eq!(
        powershell_profile,
        powershell_managed_block(&paths.powershell_script)
    );

    let zsh_profile = fs::read_to_string(&targets.zsh_profile).expect("zsh profile readable");
    assert!(zsh_profile.contains(MANAGED_BLOCK_START));
    assert!(zsh_profile.contains(&paths.zsh_script.display().to_string()));
    assert_eq!(zsh_profile, bash_managed_block(&paths.zsh_script));
}

#[test]
fn init_replaces_existing_blocks_without_touching_user_content() {
    let temp = tempdir().expect("temp dir should be created");
    let config_dir = temp.path().join("hunming");
    let paths = AppPaths::from_config_dir(&config_dir);
    let targets = InitTargets {
        bash_profile: temp.path().join(".bashrc"),
        zsh_profile: temp.path().join(".zshrc"),
        powershell_profile: temp
            .path()
            .join("PowerShell")
            .join("Microsoft.PowerShell_profile.ps1"),
    };

    let original_bash = format!(
        "export PATH=\"$HOME/bin:$PATH\"\n{start}\nold bash\n{end}\n",
        start = MANAGED_BLOCK_START,
        end = MANAGED_BLOCK_END
    );
    fs::create_dir_all(targets.bash_profile.parent().expect("bash profile parent"))
        .expect("bash profile dir should be created");
    fs::write(&targets.bash_profile, original_bash).expect("seed bash profile");

    let original_powershell = format!(
        "Write-Host \"hello\"\n{start}\nold ps1\n{end}\n",
        start = MANAGED_BLOCK_START,
        end = MANAGED_BLOCK_END
    );
    fs::create_dir_all(
        targets
            .powershell_profile
            .parent()
            .expect("powershell profile parent"),
    )
    .expect("powershell profile dir should be created");
    fs::write(&targets.powershell_profile, original_powershell).expect("seed powershell profile");

    init_with_targets(&paths, &targets).expect("init should succeed");

    let bash_profile = fs::read_to_string(&targets.bash_profile).expect("bash profile readable");
    assert!(bash_profile.starts_with("export PATH=\"$HOME/bin:$PATH\"\n"));
    assert_eq!(bash_profile.matches(MANAGED_BLOCK_START).count(), 1);
    assert!(!bash_profile.contains("old bash"));

    let powershell_profile =
        fs::read_to_string(&targets.powershell_profile).expect("powershell profile readable");
    assert!(powershell_profile.starts_with("Write-Host \"hello\"\n"));
    assert_eq!(powershell_profile.matches(MANAGED_BLOCK_START).count(), 1);
    assert!(!powershell_profile.contains("old ps1"));

    let zsh_profile = fs::read_to_string(&targets.zsh_profile).expect("zsh profile readable");
    assert!(zsh_profile.starts_with(&bash_managed_block(&paths.zsh_script)));
    assert_eq!(zsh_profile.matches(MANAGED_BLOCK_START).count(), 1);
}

#[test]
fn init_with_shell_bash_only_updates_bash_profile() {
    let temp = tempdir().expect("temp dir should be created");
    let config_dir = temp.path().join("hunming");
    let paths = AppPaths::from_config_dir(&config_dir);
    let targets = InitTargets {
        bash_profile: temp.path().join(".bashrc"),
        zsh_profile: temp.path().join(".zshrc"),
        powershell_profile: temp
            .path()
            .join("PowerShell")
            .join("Microsoft.PowerShell_profile.ps1"),
    };

    init_with_targets_and_shell(&paths, &targets, Some(InitShell::Bash))
        .expect("init should succeed");

    assert!(paths.config_file.exists());
    assert!(paths.bash_script.exists());
    assert!(paths.zsh_script.exists());
    assert!(paths.powershell_script.exists());
    assert!(targets.bash_profile.exists());
    assert!(!targets.zsh_profile.exists());
    assert!(!targets.powershell_profile.exists());
}

#[test]
fn init_with_shell_zsh_only_updates_zsh_profile() {
    let temp = tempdir().expect("temp dir should be created");
    let config_dir = temp.path().join("hunming");
    let paths = AppPaths::from_config_dir(&config_dir);
    let targets = InitTargets {
        bash_profile: temp.path().join(".bashrc"),
        zsh_profile: temp.path().join(".zshrc"),
        powershell_profile: temp
            .path()
            .join("PowerShell")
            .join("Microsoft.PowerShell_profile.ps1"),
    };

    init_with_targets_and_shell(&paths, &targets, Some(InitShell::Zsh))
        .expect("init should succeed");

    assert!(paths.config_file.exists());
    assert!(paths.bash_script.exists());
    assert!(paths.zsh_script.exists());
    assert!(paths.powershell_script.exists());
    assert!(!targets.bash_profile.exists());
    assert!(targets.zsh_profile.exists());
    assert!(!targets.powershell_profile.exists());
}

#[test]
fn init_with_shell_powershell_only_updates_powershell_profile() {
    let temp = tempdir().expect("temp dir should be created");
    let config_dir = temp.path().join("hunming");
    let paths = AppPaths::from_config_dir(&config_dir);
    let targets = InitTargets {
        bash_profile: temp.path().join(".bashrc"),
        zsh_profile: temp.path().join(".zshrc"),
        powershell_profile: temp
            .path()
            .join("PowerShell")
            .join("Microsoft.PowerShell_profile.ps1"),
    };

    init_with_targets_and_shell(&paths, &targets, Some(InitShell::Powershell))
        .expect("init should succeed");

    assert!(paths.config_file.exists());
    assert!(paths.bash_script.exists());
    assert!(paths.zsh_script.exists());
    assert!(paths.powershell_script.exists());
    assert!(!targets.bash_profile.exists());
    assert!(!targets.zsh_profile.exists());
    assert!(targets.powershell_profile.exists());
}
