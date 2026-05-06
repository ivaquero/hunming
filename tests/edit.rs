use hunming::install::edit_with_opener;
use hunming::paths::AppPaths;
use std::fs;
use tempfile::tempdir;

#[test]
fn edit_opens_config_and_reapplies_scripts() {
    let temp = tempdir().expect("temp dir should be created");
    let paths = AppPaths::from_config_dir(temp.path().join("hunming"));

    edit_with_opener(&paths, |config_file| {
        fs::write(
            config_file,
            r#"
version = 1

[aliases.gs]
command = ["git", "status", "--short"]
"#,
        )
        .expect("config should be written");
        Ok(())
    })
    .expect("edit should succeed");

    assert!(paths.config_file.exists());
    assert_eq!(
        fs::read_to_string(&paths.bash_script).expect("bash script should exist"),
        "gs() {\n  git status --short \"$@\"\n}\n"
    );
    assert_eq!(
        fs::read_to_string(&paths.powershell_script).expect("powershell script should exist"),
        "function gs {\n    git status --short @args\n}\n"
    );
}
