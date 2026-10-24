mod common;

use std::path::Path;
use std::process::Command;

use vile_core::{Document, Editor, Pos, SaveError, ops};

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .status()
        .expect("git runs");
    assert!(status.success(), "git {args:?} failed");
}

fn committed_copy(name: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.vil");
    std::fs::write(&path, common::fixture_bytes(name)).unwrap();
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["add", "config.vil"]);
    git(
        dir.path(),
        &[
            "-c",
            "user.name=vile",
            "-c",
            "user.email=vile@example.invalid",
            "commit",
            "-q",
            "-m",
            "fixture",
        ],
    );
    (dir, path)
}

#[test]
fn reads_the_committed_version_from_git_head() {
    let (_dir, path) = committed_copy("corne_v32.vil");
    std::fs::write(&path, b"working copy").unwrap();
    assert_eq!(
        vile_core::git::read_head_version(&path).unwrap(),
        common::fixture_bytes("corne_v32.vil")
    );
}

#[test]
fn git_head_outside_a_repository_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.vil");
    std::fs::write(&path, common::fixture_bytes("corne.vil")).unwrap();
    assert!(vile_core::git::read_head_version(&path).is_err());
}

#[test]
fn editor_diff_against_git_head() {
    let (_dir, path) = committed_copy("corne_v32.vil");
    let (mut ed, _) = Editor::open(&path).unwrap();
    let lt = common::kc(ed.doc(), "LT8(KC_ESCAPE)");
    ed.apply(|d| ops::set_keys(d, 0, &[Pos { row: 3, col: 4 }], lt))
        .unwrap();
    ed.save().unwrap();
    ed.set_diff_base_git().unwrap();
    let lines: Vec<String> = ed.diff().into_iter().map(|e| e.line).collect();
    assert_eq!(lines, vec!["L0 r3c4: MO(7) -> LT8(KC_ESCAPE)".to_string()]);
    ed.set_diff_base_disk();
    assert!(ed.diff().is_empty());
}

#[test]
fn save_writes_canonical_bytes_and_clears_dirty() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("x.vil");
    std::fs::write(&path, common::fixture_bytes("corne.vil")).unwrap();
    let (mut ed, report) = Editor::open(&path).unwrap();
    assert!(report.canonical);
    assert_eq!(ed.board().map(|b| b.id.as_str()), Some("matrix"));
    let a = common::kc(ed.doc(), "KC_A");
    ed.apply(|d| ops::set_keys(d, 2, &[Pos { row: 0, col: 0 }], a))
        .unwrap();
    assert!(ed.is_dirty());
    ed.save().unwrap();
    assert!(!ed.is_dirty());
    let bytes = std::fs::read(&path).unwrap();
    let (reloaded, report) = Document::from_vil_bytes(&bytes).unwrap();
    assert!(report.canonical);
    assert_eq!(&reloaded, ed.doc());
    assert_eq!(bytes, ed.doc().to_vil_bytes());
}

#[test]
fn save_refuses_when_the_file_changed_on_disk() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("x.vil");
    std::fs::write(&path, common::fixture_bytes("corne.vil")).unwrap();
    let (mut ed, _) = Editor::open(&path).unwrap();
    std::fs::write(&path, common::fixture_bytes("corne_v32.vil")).unwrap();
    let a = common::kc(ed.doc(), "KC_A");
    ed.apply(|d| ops::set_keys(d, 2, &[Pos { row: 0, col: 0 }], a))
        .unwrap();
    assert!(matches!(ed.save(), Err(SaveError::ChangedOnDisk(_))));
    assert_eq!(
        std::fs::read(&path).unwrap(),
        common::fixture_bytes("corne_v32.vil")
    );
}

#[test]
fn save_as_writes_a_new_file_and_switches_path() {
    let dir = tempfile::tempdir().unwrap();
    let mut ed = Editor::new(common::load("corne.vil"), None, Some(common::board()));
    assert!(matches!(ed.save(), Err(SaveError::NoPath)));
    let path = dir.path().join("new.vil");
    ed.save_as(&path).unwrap();
    assert_eq!(ed.path(), Some(path.as_path()));
    assert_eq!(
        std::fs::read(&path).unwrap(),
        common::fixture_bytes("corne.vil")
    );
}
