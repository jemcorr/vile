use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::str::{contains, starts_with};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

fn vile() -> Command {
    Command::cargo_bin("vile").expect("vile binary")
}

fn git(dir: &Path, args: &[&str]) {
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

#[test]
fn set_one_key_and_diff_exactly() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.vil");
    vile()
        .arg("set")
        .arg(fixture("corne_v32.vil"))
        .args(["0", "thumb-left-mid", "LT8(KC_ESCAPE)", "-o"])
        .arg(&out)
        .assert()
        .success();
    vile()
        .arg("diff")
        .arg(fixture("corne_v32.vil"))
        .arg(&out)
        .assert()
        .success()
        .stdout("L0 thumb-left-mid: MO(7) -> LT8(KC_ESCAPE)\n");
}

#[test]
fn set_accepts_lenient_input_and_matrix_positions() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.vil");
    vile()
        .arg("set")
        .arg(fixture("corne_v32.vil"))
        .args(["0", "r0c0", "esc", "-o"])
        .arg(&out)
        .assert()
        .success()
        .stdout("L0 top-left-outer: KC_GESC -> KC_ESCAPE\n");
}

#[test]
fn set_rejects_unknown_keycodes_and_positions() {
    vile()
        .arg("set")
        .arg(fixture("corne_v32.vil"))
        .args(["0", "thumb-left-mid", "KC_FOO"])
        .assert()
        .code(1)
        .stderr(contains("unknown keycode"));
    vile()
        .arg("set")
        .arg(fixture("corne_v32.vil"))
        .args(["0", "r3c0", "KC_A"])
        .assert()
        .code(1)
        .stderr(contains("r3c0"));
}

#[test]
fn layer_move_renumbers() {
    vile()
        .args(["layer", "move"])
        .arg(fixture("corne_v32.vil"))
        .args(["9", "1"])
        .assert()
        .success()
        .stdout(contains("L0 thumb-left-mid: MO(7) -> MO(8)\n"))
        .stdout(contains("L0 thumb-right-mid: MO(8) -> MO(9)\n"));
}

#[test]
fn layer_mirror_left_to_right() {
    vile()
        .args(["layer", "mirror"])
        .arg(fixture("corne_v32.vil"))
        .args(["1", "--mode", "left-to-right"])
        .assert()
        .success()
        .stdout(contains("L1 top-right-pinky: KC_TRNS -> KC_Q\n"));
}

#[test]
fn keycodes_search() {
    vile()
        .args(["keycodes", "esc", "--limit", "3"])
        .assert()
        .success()
        .stdout(starts_with("KC_ESCAPE\t"));
    vile()
        .args(["keycodes", "zzzzzz"])
        .assert()
        .success()
        .stdout("");
}

#[test]
fn show_prints_a_layer() {
    vile()
        .arg("show")
        .arg(fixture("corne_v32.vil"))
        .args(["--layer", "0"])
        .assert()
        .success()
        .stdout(contains("L0"))
        .stdout(contains("MO7"));
}

#[test]
fn diff_against_git_head() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.vil");
    std::fs::copy(fixture("corne_v32.vil"), &path).unwrap();
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
            "x",
        ],
    );
    vile()
        .arg("diff")
        .arg(&path)
        .arg("--git")
        .assert()
        .success()
        .stdout("no changes\n");
    vile()
        .arg("set")
        .arg(&path)
        .args(["0", "thumb-left-mid", "LT8(KC_ESCAPE)", "--write"])
        .assert()
        .success();
    vile()
        .arg("diff")
        .arg(&path)
        .arg("--git")
        .assert()
        .success()
        .stdout("L0 thumb-left-mid: MO(7) -> LT8(KC_ESCAPE)\n");
}
