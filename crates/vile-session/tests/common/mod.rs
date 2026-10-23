#![allow(dead_code)]

use std::path::{Path, PathBuf};

use vile_session::ViewModel;

pub fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

/// Copy a fixture into a fresh temporary directory and return (dir, path).
pub fn temp_copy(name: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(name);
    std::fs::copy(fixture(name), &path).unwrap();
    (dir, path)
}

pub fn key_index(view: &ViewModel, id: &str) -> usize {
    view.board
        .as_ref()
        .expect("board")
        .keys
        .iter()
        .position(|k| k.id == id)
        .unwrap_or_else(|| panic!("no key {id}"))
}

pub fn wire(view: &ViewModel, layer: usize, id: &str) -> String {
    view.layers[layer][key_index(view, id)].wire.clone()
}

pub fn git(dir: &Path, args: &[&str]) {
    let status = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .status()
        .expect("git runs");
    assert!(status.success(), "git {args:?} failed");
}
