mod common;

use vile_core::validate::{macro_buffer_bytes, validate};
use vile_core::{Macro, MacroAction};

#[test]
fn fixtures_have_no_warnings() {
    for name in ["corne.vil", "corne_v32.vil"] {
        assert!(validate(&common::load(name)).is_empty(), "{name}");
    }
}

#[test]
fn out_of_range_references_warn() {
    let mut doc = common::load("corne.vil");
    let mo = doc.catalog().parse("MO(12)").unwrap();
    doc.set(1, 0, 0, mo).unwrap();
    let m20 = doc.catalog().parse("M20").unwrap();
    doc.set(1, 0, 1, m20).unwrap();
    let td = doc.catalog().parse("TD(40)").unwrap();
    doc.set(1, 0, 2, td).unwrap();
    let issues = validate(&doc);
    let paths: Vec<&str> = issues.iter().map(|i| i.path.as_str()).collect();
    assert_eq!(
        paths,
        vec!["layout[1][0][0]", "layout[1][0][1]", "layout[1][0][2]"],
        "{issues:?}"
    );
    assert!(issues[0].message.contains("layer 12"), "{issues:?}");
}

#[test]
fn macro_buffer_size_matches_vial_encoding() {
    let mut doc = common::load("corne.vil");
    for m in doc.macros.iter_mut() {
        *m = Macro::default();
    }
    assert_eq!(macro_buffer_bytes(&doc), 16);
    let a = doc.catalog().parse("KC_A").unwrap();
    let shifted = doc.catalog().parse("LSFT(KC_A)").unwrap();
    doc.macros[0] = Macro {
        actions: vec![
            MacroAction::Tap(vec![a, shifted]),
            MacroAction::Text("hi".to_string()),
            MacroAction::Delay(10),
        ],
    };
    // tap KC_A: 3 bytes, tap LSFT(KC_A): 4 bytes, text: 2 bytes, delay: 4 bytes, plus 16 terminators
    assert_eq!(macro_buffer_bytes(&doc), 3 + 4 + 2 + 4 + 16);
}
