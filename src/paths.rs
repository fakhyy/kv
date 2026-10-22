use crate::error::{KvError, Result};
use directories::BaseDirs;
use std::path::{Path, PathBuf};

pub fn default_database_path() -> Result<PathBuf> {
    let base = BaseDirs::new().ok_or(KvError::NoDataDirectory)?;
    Ok(base.data_local_dir().join("kv").join("data.db"))
}

pub fn resolve_database_path(override_path: Option<PathBuf>) -> Result<PathBuf> {
    let path = match override_path {
        Some(path) => path,
        None => default_database_path()?,
    };

    // Do not canonicalize here: the target may not exist yet.
    if path.as_os_str().is_empty() {
        return Err(KvError::NotAFile(path.display().to_string()));
    }

    Ok(path)
}

pub fn ensure_parent_directory(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}
