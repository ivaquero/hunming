use hunming::fs::atomic_write;
use std::fs;
use tempfile::tempdir;

#[test]
fn atomic_write_creates_parent_directories() {
    let temp = tempdir().expect("temp dir should be created");
    let path = temp.path().join("nested").join("config.txt");

    atomic_write(&path, "hello world").expect("atomic write should succeed");

    assert_eq!(
        fs::read_to_string(&path).expect("file should be readable"),
        "hello world"
    );
    assert!(path.parent().expect("path should have parent").is_dir());
}

#[test]
fn atomic_write_replaces_existing_content() {
    let temp = tempdir().expect("temp dir should be created");
    let path = temp.path().join("data.txt");

    fs::write(&path, "old").expect("seed file should be written");
    atomic_write(&path, "new").expect("atomic write should succeed");

    assert_eq!(
        fs::read_to_string(&path).expect("file should be readable"),
        "new"
    );
}
