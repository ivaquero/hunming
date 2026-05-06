use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

pub fn atomic_write(path: &Path, content: &str) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).with_context(|| {
        format!("failed to create parent directory at {}", parent.display())
    })?;

    let mut temp_file = NamedTempFile::new_in(parent).with_context(|| {
        format!("failed to create temporary file in {}", parent.display())
    })?;

    temp_file
        .write_all(content.as_bytes())
        .context("failed to write temporary content")?;
    temp_file
        .flush()
        .context("failed to flush temporary content")?;
    temp_file
        .as_file()
        .sync_all()
        .context("failed to sync temporary content")?;

    temp_file
        .persist(path)
        .map(|_| ())
        .with_context(|| format!("failed to replace {}", path.display()))?;

    Ok(())
}
