//! Read the committed (git HEAD) version of a file by running `git show`.
//! Read-only: this module never runs a git command that changes a repository.

use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("git is not installed or not on PATH")]
    NotFound,
    #[error("git failed: {0}")]
    Failed(String),
    #[error("path has no file name: {0}")]
    BadPath(String),
}

/// The bytes of `path` as committed at HEAD of the repository that contains it.
pub fn read_head_version(path: &Path) -> Result<Vec<u8>, GitError> {
    let name = path
        .file_name()
        .ok_or_else(|| GitError::BadPath(path.display().to_string()))?
        .to_string_lossy()
        .into_owned();
    let dir: PathBuf = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => PathBuf::from("."),
    };
    let output = Command::new("git")
        .arg("-C")
        .arg(&dir)
        .arg("show")
        .arg(format!("HEAD:./{name}"))
        .output()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => GitError::NotFound,
            _ => GitError::Failed(e.to_string()),
        })?;
    if !output.status.success() {
        return Err(GitError::Failed(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    Ok(output.stdout)
}
