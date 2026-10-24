use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::str::{contains, starts_with};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

fn temp_copy(name: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(name);
    std::fs::copy(fixture(name), &path).unwrap();
    (dir, path)
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
        .args(["0", "r3c4", "LT8(KC_ESCAPE)", "-o"])
        .arg(&out)
        .assert()
        .success();
    vile()
        .arg("diff")
        .arg(fixture("corne_v32.vil"))
        .arg(&out)
        .assert()
        .success()
        .stdout("L0 r3c4: MO(7) -> LT8(KC_ESCAPE)\n");
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
        .stdout("L0 r0c0: KC_GESC -> KC_ESCAPE\n");
}

#[test]
fn set_rejects_unknown_keycodes_and_positions() {
    vile()
        .arg("set")
        .arg(fixture("corne_v32.vil"))
        .args(["0", "r3c4", "KC_FOO"])
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
        .stdout(contains("L0 r3c4: MO(7) -> MO(8)\n"))
        .stdout(contains("L0 r7c4: MO(8) -> MO(9)\n"));
}

#[test]
fn layer_mirror_requires_definition_pairing() {
    vile()
        .args(["layer", "mirror"])
        .arg(fixture("corne_v32.vil"))
        .args(["1", "--mode", "left-to-right"])
        .assert()
        .failure()
        .stderr(contains("mirror pairs"));
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
        .args(["0", "r3c4", "LT8(KC_ESCAPE)", "--write"])
        .assert()
        .success();
    vile()
        .arg("diff")
        .arg(&path)
        .arg("--git")
        .assert()
        .success()
        .stdout("L0 r3c4: MO(7) -> LT8(KC_ESCAPE)\n");
}

#[test]
fn cli_preserves_names_through_layer_reordering() {
    let (dir, path) = temp_copy("corne.vil");
    let (mut editor, _) = vile_core::Editor::open(&path).unwrap();
    editor
        .apply_named(|_, names| {
            names.layers.insert(7, "Symbols".into());
            Ok(Vec::new())
        })
        .unwrap();
    editor.save().unwrap();
    let before = std::fs::read(&path).unwrap();
    vile()
        .args(["layer", "swap"])
        .arg(&path)
        .args(["7", "8"])
        .assert()
        .success()
        .stdout(contains("L8 name:  -> Symbols"));
    assert_eq!(std::fs::read(&path).unwrap(), before);
    vile()
        .args(["layer", "swap"])
        .arg(&path)
        .args(["7", "8", "--write"])
        .assert()
        .success();
    let (editor, _) = vile_core::Editor::open(&path).unwrap();
    assert_eq!(
        editor.names().layers.get(&8).map(String::as_str),
        Some("Symbols")
    );
    assert!(editor.metadata_warnings().is_empty());
    assert!(dir.path().join("corne.vil.vile.json").exists());
}
#[test]
fn cli_definition_supplies_aliases_and_explicit_mirror_pairs() {
    let (dir, path) = temp_copy("corne_v32.vil");
    let doc = vile_core::Document::load(&path).unwrap().0;
    let rows: Vec<Vec<String>> = (0..doc.rows())
        .map(|r| {
            (0..doc.cols())
                .filter(|&c| matches!(doc.get(0, r, c), Some(vile_core::Slot::Key(_))))
                .map(|c| format!("{r},{c}"))
                .collect()
        })
        .collect();
    let definition = serde_json::json!({"matrix":{"rows":8,"cols":7},"layouts":{"keymap":rows},"vile":{"key_names":{"r3c4":"left thumb"},"mirror_pairs":[["r0c1","r4c1"]]}});
    let geometry = dir.path().join("vial.json");
    std::fs::write(&geometry, serde_json::to_vec(&definition).unwrap()).unwrap();
    vile()
        .arg("--definition")
        .arg(&geometry)
        .arg("set")
        .arg(&path)
        .args(["0", "left thumb", "KC_ESCAPE"])
        .assert()
        .success()
        .stdout(contains("L0 left thumb: MO(7) -> KC_ESCAPE"));
    vile()
        .arg("--definition")
        .arg(&geometry)
        .args(["layer", "mirror"])
        .arg(&path)
        .args(["1", "--mode", "left-to-right"])
        .assert()
        .success()
        .stdout(contains("L1 r4c1: KC_TRNS -> KC_Q"));
}
