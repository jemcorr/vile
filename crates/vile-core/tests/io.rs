use vile_core::io::write_atomic;

#[test]
fn atomic_write_creates_and_replaces() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("out.vil");
    write_atomic(&path, b"first").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"first");
    write_atomic(&path, b"second").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"second");
    let names: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(names.len(), 1, "temporary files left behind: {names:?}");
}

#[test]
fn atomic_write_keeps_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("out.vil");
    std::fs::write(&path, b"x").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640)).unwrap();
    write_atomic(&path, b"y").unwrap();
    let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o640);
}

#[test]
fn atomic_write_fails_for_missing_directory() {
    let dir = tempfile::tempdir().unwrap();
    assert!(write_atomic(&dir.path().join("missing/out.vil"), b"x").is_err());
}
