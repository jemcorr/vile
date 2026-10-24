mod common;

use serde_json::json;
use vile_core::geometry::from_vial_definition;
use vile_session::Session;

#[test]
fn no_file_means_no_view_and_clear_errors() {
    let mut s = Session::new();
    assert!(s.view().is_none());
    assert!(s.save().unwrap_err().contains("no file is open"));
    assert!(s.undo().unwrap_err().contains("no file is open"));
    assert_eq!(s.resolve_keycode("esc").unwrap(), "KC_ESCAPE");
}

#[test]
fn open_builds_a_complete_view() {
    let mut s = Session::new();
    let v = s.open_path(&common::fixture("corne.vil")).unwrap();
    assert_eq!(v.uid, "5010774632021243529");
    assert_eq!(v.layer_count, 10);
    assert!(!v.dirty);
    assert!(!v.can_undo);
    assert!(!v.can_redo);
    assert!(v.diff.is_empty());
    assert_eq!(v.diff_base, "disk");
    let board = v.board.as_ref().unwrap();
    assert_eq!(board.keys.len(), 46);
    assert_eq!(board.keys[0].id, "r0c0");
    assert_eq!(v.layers.len(), 10);
    assert!(v.layers.iter().all(|l| l.len() == 46));
    assert!(v.layers.iter().flatten().all(|c| !c.changed));
    assert_eq!(v.macros.len(), 16);
    assert_eq!(v.combos.len(), 32);
    assert_eq!(v.tap_dances.len(), 32);
    assert_eq!(v.key_overrides.len(), 32);
    assert_eq!(v.alt_repeat_keys.len(), 32);
    assert!(v.path.as_deref().unwrap().ends_with("corne.vil"));
    assert_eq!(s.view(), Some(v));
}

#[test]
fn known_knobless_layouts_keep_inactive_encoder_bindings_when_saved() {
    let (_dir, path) = common::temp_copy("corne.vil");
    let before = std::fs::read(&path).unwrap();
    let mut session = Session::new();
    session.open_path(&path).unwrap();
    let definition = json!({
        "matrix":{"rows":8,"cols":7},
        "layouts":{
            "labels":[["EX1", "Key", "Encoder"]],
            "keymap":[["0,0", "0,0\n\n\n0,1\n\n\n\n\n\ne"]]
        }
    });
    let board = from_vial_definition(&definition, session.document().unwrap()).unwrap();
    let view = session.attach_board(board, None).unwrap();
    assert_eq!(view.board.unwrap().encoder_indices, Some(vec![]));
    session.save().unwrap();
    assert_eq!(std::fs::read(path).unwrap(), before);
}

#[test]
fn cells_carry_wire_label_and_kind() {
    let mut s = Session::new();
    let v = s.open_path(&common::fixture("corne_v32.vil")).unwrap();
    let cell = &v.layers[0][common::key_index(&v, "r3c4")];
    assert_eq!(cell.id, "r3c4");
    assert_eq!(cell.wire, "MO(7)");
    assert_eq!(cell.label, "MO7");
    assert_eq!(cell.kind, "layer");
    assert_eq!(v.layers[1][common::key_index(&v, "r0c0")].kind, "trns");
    assert_eq!(v.layers[0][common::key_index(&v, "r1c1")].kind, "modTap");
}

#[test]
fn view_json_is_camel_case_with_a_string_uid() {
    let mut s = Session::new();
    let v = s.open_path(&common::fixture("corne.vil")).unwrap();
    let json = serde_json::to_value(&v).unwrap();
    assert_eq!(json["uid"], serde_json::json!("5010774632021243529"));
    assert!(json["layerCount"].is_u64());
    assert!(json["canUndo"].is_boolean());
    assert!(json["diffBase"].is_string());
    assert!(json["layers"][0][0]["wire"].is_string());
    assert!(json["board"]["keys"][0]["x"].is_number());
    assert!(json["tapDances"][0]["tappingTerm"].is_u64());
    assert!(json["keyOverrides"][0]["negativeModMask"].is_u64());
    assert!(json["altRepeatKeys"][0]["altKeycode"].is_string());
    assert!(json["combos"][0]["outputLabel"].is_string());
}

#[test]
fn files_for_other_boards_use_a_matrix_fallback() {
    let raw = std::fs::read_to_string(common::fixture("corne.vil")).unwrap();
    // Turn the absent positions r3c0 and r7c0 into keys on every layer:
    // the file still loads, but the matrix no longer matches the Corne board.
    let changed = raw.replace("[-1, -1, -1, \"", "[\"KC_NO\", -1, -1, \"");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("other.vil");
    std::fs::write(&path, changed).unwrap();
    let mut s = Session::new();
    let view = s.open_path(&path).unwrap();
    assert_eq!(view.board.unwrap().id, "matrix");
    assert_eq!(view.geometry_source, "matrix");
    assert_eq!(view.layers[0].len(), 48);
}

#[test]
fn invalid_files_report_every_problem() {
    let raw = std::fs::read_to_string(common::fixture("corne_v32.vil")).unwrap();
    let broken =
        raw.replacen("\"KC_GESC\"", "\"KC_NOPE\"", 1)
            .replacen("\"KC_BSPACE\"", "\"KC_NADA\"", 1);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("broken.vil");
    std::fs::write(&path, broken).unwrap();
    let err = Session::new().open_path(&path).unwrap_err();
    assert!(err.contains("KC_NOPE") && err.contains("KC_NADA"), "{err}");
}

#[test]
fn save_and_save_as() {
    let (_dir, path) = common::temp_copy("corne.vil");
    let mut s = Session::new();
    s.open_path(&path).unwrap();
    let v = s.save().unwrap();
    assert!(!v.dirty);
    assert_eq!(
        std::fs::read(&path).unwrap(),
        std::fs::read(common::fixture("corne.vil")).unwrap()
    );
    let other = path.with_file_name("copy.vil");
    let v = s.save_as(&other).unwrap();
    assert!(v.path.as_deref().unwrap().ends_with("copy.vil"));
    assert!(other.exists());
}
