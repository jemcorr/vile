mod common;

use vile_core::Document;

#[test]
fn empty_macro_list_is_refused() {
    let raw = common::fixture_text("corne.vil");
    let start = raw.find("\"macro\": [").expect("macro key");
    let end = start
        + raw[start..]
            .find("], \"vial_protocol\"")
            .expect("end of macro list")
        + 1;
    let broken = format!("{}\"macro\": []{}", &raw[..start], &raw[end..]);
    let err = Document::from_vil_bytes(broken.as_bytes())
        .unwrap_err()
        .to_string();
    assert!(err.contains("macro"), "{err}");
}

#[test]
fn unknown_keycode_in_file_is_a_load_error_with_path() {
    let raw = common::fixture_text("corne_v32.vil").replacen("\"KC_GESC\"", "\"KC_GESCX\"", 1);
    let err = Document::from_vil_bytes(raw.as_bytes())
        .unwrap_err()
        .to_string();
    assert!(err.contains("KC_GESCX"), "{err}");
    assert!(err.contains("layout[0][0][0]"), "{err}");
}

#[test]
fn unknown_top_level_key_is_refused() {
    let raw = common::fixture_text("corne.vil").replacen(
        "\"settings\": {",
        "\"bogus\": 1, \"settings\": {",
        1,
    );
    let err = Document::from_vil_bytes(raw.as_bytes())
        .unwrap_err()
        .to_string();
    assert!(err.contains("bogus"), "{err}");
}

#[test]
fn non_canonical_spelling_loads_with_a_warning() {
    let aliased = common::fixture_text("corne_v32.vil").replacen("\"KC_BSPACE\"", "\"KC_BSPC\"", 1);
    let (doc, report) = Document::from_vil_bytes(aliased.as_bytes()).expect("alias loads");
    assert!(!report.canonical);
    assert_eq!(report.warnings.len(), 1, "{:?}", report.warnings);
    assert_eq!(report.warnings[0].path, "layout[0][7][5]");
    assert!(
        report.warnings[0].message.contains("KC_BSPACE"),
        "{:?}",
        report.warnings
    );
    assert_eq!(doc.to_vil_bytes(), common::fixture_bytes("corne_v32.vil"));
}

#[test]
fn json_errors_are_reported() {
    assert!(Document::from_vil_bytes(b"{").is_err());
    assert!(Document::from_vil_bytes(b"[]").is_err());
    assert!(Document::from_vil_bytes(b"{}").is_err());
}
#[test]
fn control_characters_in_macro_text_are_load_errors() {
    let raw = common::fixture_text("corne_v32.vil").replacen("\"# %%\"", "\"a\\u0001b\"", 1);
    let err = Document::from_vil_bytes(raw.as_bytes())
        .unwrap_err()
        .to_string();
    assert!(err.contains("macro[2]"), "{err}");
}

#[test]
fn too_long_delay_is_a_load_error() {
    let raw = common::fixture_text("corne_v32.vil").replacen(
        "[\"text\", \"//\"]",
        "[\"delay\", 70000]",
        1,
    );
    let err = Document::from_vil_bytes(raw.as_bytes())
        .unwrap_err()
        .to_string();
    assert!(err.contains("delay"), "{err}");
}
