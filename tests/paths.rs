use hunming::paths::AppPaths;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn builds_unix_paths_from_home_directory() {
    let paths = AppPaths::from_unix_home("/Users/alice");

    assert_eq!(
        paths.config_dir,
        PathBuf::from("/Users/alice/.config/hunming")
    );
    assert_eq!(
        paths.config_file,
        PathBuf::from("/Users/alice/.config/hunming/aliases.toml")
    );
    assert_eq!(
        paths.generated_dir,
        PathBuf::from("/Users/alice/.config/hunming/generated")
    );
    assert_eq!(
        paths.bash_script,
        PathBuf::from("/Users/alice/.config/hunming/generated/bash.sh")
    );
    assert_eq!(
        paths.zsh_script,
        PathBuf::from("/Users/alice/.config/hunming/generated/zsh.sh")
    );
    assert_eq!(
        paths.powershell_script,
        PathBuf::from("/Users/alice/.config/hunming/generated/powershell.ps1")
    );
}

#[test]
fn builds_windows_paths_from_appdata_directory() {
    let paths = AppPaths::from_windows_appdata(r"C:\Users\alice\AppData\Roaming");

    assert_eq!(
        paths.config_dir,
        PathBuf::from(r"C:\Users\alice\AppData\Roaming/hunming")
    );
    assert_eq!(
        paths.config_file,
        PathBuf::from(r"C:\Users\alice\AppData\Roaming/hunming/aliases.toml")
    );
    assert_eq!(
        paths.generated_dir,
        PathBuf::from(r"C:\Users\alice\AppData\Roaming/hunming/generated")
    );
    assert_eq!(
        paths.bash_script,
        PathBuf::from(r"C:\Users\alice\AppData\Roaming/hunming/generated/bash.sh")
    );
    assert_eq!(
        paths.zsh_script,
        PathBuf::from(r"C:\Users\alice\AppData\Roaming/hunming/generated/zsh.sh")
    );
    assert_eq!(
        paths.powershell_script,
        PathBuf::from(r"C:\Users\alice\AppData\Roaming/hunming/generated/powershell.ps1")
    );
}

#[test]
fn creates_missing_directories() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));

    paths
        .ensure_config_dir()
        .expect("config directory should be created");
    paths
        .ensure_generated_dir()
        .expect("generated directory should be created");

    assert!(paths.config_dir.is_dir());
    assert!(paths.generated_dir.is_dir());
}

#[test]
fn builds_paths_from_custom_config_file() {
    let temp = tempdir().expect("temp dir should be created");
    let config_file = temp.path().join("profiles").join("aliases.toml");
    let paths = AppPaths::from_config_file(&config_file);

    assert_eq!(paths.config_file, config_file);
    assert_eq!(paths.config_dir, temp.path().join("profiles"));
    assert_eq!(
        paths.generated_dir,
        temp.path().join("profiles").join("generated")
    );
    assert_eq!(
        paths.bash_script,
        temp.path()
            .join("profiles")
            .join("generated")
            .join("bash.sh")
    );
}
