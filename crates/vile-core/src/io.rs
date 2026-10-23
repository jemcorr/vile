//! Atomic file writes: write a temporary file next to the target, flush it,
//! then rename it over the target. A crash never leaves a half-written file.

use std::io::Write as _;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;

/// Replace `path` with `bytes` atomically. An existing file keeps its
/// permissions; a new file gets mode 0644.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let dir = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
    tmp.write_all(bytes)?;
    tmp.as_file().sync_all()?;
    let permissions = match std::fs::metadata(path) {
        Ok(meta) => meta.permissions(),
        Err(_) => std::fs::Permissions::from_mode(0o644),
    };
    std::fs::set_permissions(tmp.path(), permissions)?;
    tmp.persist(path).map_err(|e| e.error)?;
    Ok(())
}
