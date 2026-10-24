use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

fn vile() -> Command {
    Command::cargo_bin("vile").expect("vile binary")
}

fn temp_copy(name: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(name);
    std::fs::copy(fixture(name), &path).unwrap();
    (dir, path)
}

fn read(path: &Path) -> Vec<u8> {
    std::fs::read(path).unwrap()
}

#[test]
fn check_accepts_the_fixture() {
    vile()
        .arg("check")
        .arg(fixture("corne.vil"))
        .assert()
        .success()
        .stdout(contains("ok: 10 layers, 8x7 matrix, 46 keys per layer"))
        .stdout(contains("uid: 5010774632021243529"))
        .stdout(contains("board: Matrix grid"))
        .stdout(contains("round trip: byte-identical"));
}

#[test]
fn check_rejects_unknown_keycodes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.vil");
    let text = std::fs::read_to_string(fixture("corne_v32.vil")).unwrap();
    std::fs::write(&path, text.replacen("\"KC_GESC\"", "\"KC_GESCX\"", 1)).unwrap();
    vile()
        .arg("check")
        .arg(&path)
        .assert()
        .code(1)
        .stderr(contains("KC_GESCX").and(contains("layout[0][0][0]")));
}

#[test]
fn layer_swap_dry_run_prints_the_diff_and_writes_nothing() {
    let (_dir, path) = temp_copy("corne_v32.vil");
    vile()
        .args(["layer", "swap"])
        .arg(&path)
        .args(["7", "8"])
        .assert()
        .success()
        .stdout(contains("L0 r3c4: MO(7) -> MO(8)\n"))
        .stdout(contains("L0 r7c4: MO(8) -> MO(7)\n"))
        .stderr(contains("dry run: nothing written"));
    assert_eq!(read(&path), read(&fixture("corne_v32.vil")));
}

#[test]
fn layer_swap_write_twice_restores_the_original_bytes() {
    let (_dir, path) = temp_copy("corne_v32.vil");
    vile()
        .args(["layer", "swap"])
        .arg(&path)
        .args(["7", "8", "--write"])
        .assert()
        .success()
        .stderr(contains("wrote "));
    assert_ne!(read(&path), read(&fixture("corne_v32.vil")));
    vile().arg("check").arg(&path).assert().success();
    vile()
        .args(["layer", "swap"])
        .arg(&path)
        .args(["7", "8", "-w"])
        .assert()
        .success();
    assert_eq!(read(&path), read(&fixture("corne_v32.vil")));
}

#[test]
fn layer_clear_to_output_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.vil");
    vile()
        .args(["layer", "clear"])
        .arg(fixture("corne_v32.vil"))
        .args(["2", "--transparent", "--output"])
        .arg(&out)
        .assert()
        .success()
        .stdout(contains("L2 r0c1: KC_Q -> KC_TRNS\n"));
    vile()
        .arg("diff")
        .arg(fixture("corne_v32.vil"))
        .arg(&out)
        .assert()
        .success()
        .stdout(contains("L2 r0c1: KC_Q -> KC_TRNS\n"));
    vile()
        .args(["layer", "clear"])
        .arg(&out)
        .args(["2", "-w"])
        .assert()
        .success()
        .stdout(contains("L2 r0c1: KC_TRNS -> KC_NO\n"));
}

#[test]
fn layer_copy_overwrites_the_destination() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.vil");
    vile()
        .args(["layer", "copy"])
        .arg(fixture("corne_v32.vil"))
        .args(["0", "5", "-o"])
        .arg(&out)
        .assert()
        .success()
        .stdout(contains("L5 r3c4: KC_TRNS -> MO(7)\n"));
    vile().arg("check").arg(&out).assert().success();
}

#[test]
fn diff_of_identical_files() {
    vile()
        .arg("diff")
        .arg(fixture("corne_v32.vil"))
        .arg(fixture("corne_v32.vil"))
        .assert()
        .success()
        .stdout("no changes\n");
}

#[test]
fn bad_layer_numbers_fail_cleanly() {
    let (_dir, path) = temp_copy("corne.vil");
    vile()
        .args(["layer", "swap"])
        .arg(&path)
        .args(["0", "12", "--write"])
        .assert()
        .code(1)
        .stderr(contains("layer 12 does not exist"));
    assert_eq!(read(&path), read(&fixture("corne.vil")));
}

#[test]
fn write_and_output_conflict() {
    vile()
        .args(["layer", "clear"])
        .arg(fixture("corne.vil"))
        .args(["2", "--write", "--output", "x.vil"])
        .assert()
        .code(2);
}
