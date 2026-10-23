mod common;

use common::{kc, wire_at};
use vile_core::{Combo, Keycode, Macro, MacroAction, Slot, ops};

#[test]
fn fill_layer_sets_every_present_key_and_keeps_absent_positions() {
    let original = common::load("corne.vil");
    let mut doc = original.clone();
    let warnings = ops::fill_layer(&mut doc, 0, Keycode::NO).unwrap();
    assert!(warnings.is_empty());
    for r in 0..8 {
        for c in 0..7 {
            match (original.get(0, r, c).unwrap(), doc.get(0, r, c).unwrap()) {
                (Slot::Absent, Slot::Absent) => {}
                (Slot::Key(_), Slot::Key(k)) => assert_eq!(k, Keycode::NO, "r{r}c{c}"),
                other => panic!("shape changed at r{r}c{c}: {other:?}"),
            }
        }
    }
    assert_eq!(doc.layout[1..], original.layout[1..]);
    assert_eq!(doc.macros, original.macros);
}

#[test]
fn fill_layer_rejects_missing_layer() {
    let mut doc = common::load("corne.vil");
    let before = doc.clone();
    assert!(ops::fill_layer(&mut doc, 10, Keycode::NO).is_err());
    assert_eq!(doc, before);
}

#[test]
fn copy_layer_copies_verbatim() {
    let mut doc = common::load("corne_v32.vil");
    ops::copy_layer(&mut doc, 0, 5).unwrap();
    assert_eq!(doc.layout[5], doc.layout[0]);
    assert_eq!(wire_at(&doc, 5, 3, 4), "MO(7)");
}

#[test]
fn layer_swap_renumbers_references() {
    let mut doc = common::load("corne_v32.vil");
    // Facts of the immutable v32 fixture.
    assert_eq!(wire_at(&doc, 0, 3, 4), "MO(7)");
    assert_eq!(wire_at(&doc, 0, 7, 4), "MO(8)");
    assert_eq!(wire_at(&doc, 7, 7, 4), "MO(9)");
    assert_eq!(wire_at(&doc, 8, 3, 4), "MO(9)");
    // Add references of other kinds in other sections.
    let lt = kc(&doc, "LT8(KC_ESCAPE)");
    doc.set(1, 0, 0, lt).unwrap();
    let combo = Combo {
        keys: [kc(&doc, "KC_A"), kc(&doc, "KC_B"), Keycode::NO, Keycode::NO],
        output: kc(&doc, "TG(7)"),
    };
    doc.combo.as_mut().unwrap()[10] = combo;
    doc.macros[5] = Macro {
        actions: vec![MacroAction::Tap(vec![kc(&doc, "OSL(8)")])],
    };
    doc.tap_dance.as_mut().unwrap()[9].on_hold = kc(&doc, "TT(7)");
    doc.key_override.as_mut().unwrap()[0].layers = 0b0000_0000_1000_0001; // layers 0 and 7
    doc.alt_repeat_key.as_mut().unwrap()[0].alt_keycode = kc(&doc, "DF(8)");
    let old_l7 = doc.layout[7].clone();
    let old_l8 = doc.layout[8].clone();

    let warnings = ops::swap_layers(&mut doc, 7, 8).unwrap();

    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(wire_at(&doc, 0, 3, 4), "MO(8)");
    assert_eq!(wire_at(&doc, 0, 7, 4), "MO(7)");
    assert_eq!(wire_at(&doc, 1, 0, 0), "LT7(KC_ESCAPE)");
    let cat = doc.catalog();
    assert_eq!(
        cat.wire_name(doc.combo.as_ref().unwrap()[10].output),
        "TG(8)"
    );
    assert_eq!(
        doc.macros[5],
        Macro {
            actions: vec![MacroAction::Tap(vec![kc(&doc, "OSL(7)")])]
        }
    );
    assert_eq!(
        cat.wire_name(doc.tap_dance.as_ref().unwrap()[9].on_hold),
        "TT(8)"
    );
    assert_eq!(
        doc.key_override.as_ref().unwrap()[0].layers,
        0b0000_0001_0000_0001
    ); // layers 0 and 8
    assert_eq!(
        cat.wire_name(doc.alt_repeat_key.as_ref().unwrap()[0].alt_keycode),
        "DF(7)"
    );
    // Old L7 and L8 only reference layer 9, so their contents move unchanged.
    assert_eq!(doc.layout[7], old_l8);
    assert_eq!(doc.layout[8], old_l7);
    // Untouched references stay.
    assert_eq!(wire_at(&doc, 0, 0, 6), "TO(2)");
    assert_eq!(wire_at(&doc, 6, 0, 0), "TO(1)");
}

#[test]
fn layer_swap_twice_is_identity() {
    let original = common::load("corne_v32.vil");
    let mut doc = original.clone();
    ops::swap_layers(&mut doc, 7, 8).unwrap();
    assert_ne!(doc, original);
    ops::swap_layers(&mut doc, 7, 8).unwrap();
    assert_eq!(doc, original);
    assert_eq!(doc.to_vil_bytes(), common::fixture_bytes("corne_v32.vil"));
}

#[test]
fn swap_with_itself_changes_nothing() {
    let original = common::load("corne_v32.vil");
    let mut doc = original.clone();
    ops::swap_layers(&mut doc, 3, 3).unwrap();
    assert_eq!(doc, original);
}

#[test]
fn permute_rejects_invalid_permutations() {
    let mut doc = common::load("corne.vil");
    let before = doc.clone();
    assert!(ops::permute_layers(&mut doc, &[0, 1, 2]).is_err());
    assert!(ops::permute_layers(&mut doc, &[0, 0, 2, 3, 4, 5, 6, 7, 8, 9]).is_err());
    assert!(ops::permute_layers(&mut doc, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 10]).is_err());
    assert_eq!(doc, before);
}

#[test]
fn tri_layer_keys_produce_a_warning() {
    let mut doc = common::load("corne.vil");
    let fn13 = kc(&doc, "FN_MO13");
    doc.set(2, 0, 0, fn13).unwrap();
    let warnings = ops::swap_layers(&mut doc, 1, 4).unwrap();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("FN_MO13"), "{warnings:?}");
    assert_eq!(wire_at(&doc, 2, 0, 0), "FN_MO13");
}

#[test]
fn layer_ops_keep_all_macro_slots() {
    let original = common::load("corne.vil");
    let mut doc = original.clone();
    ops::fill_layer(&mut doc, 2, Keycode::TRNS).unwrap();
    ops::copy_layer(&mut doc, 0, 3).unwrap();
    ops::swap_layers(&mut doc, 1, 2).unwrap();
    assert_eq!(doc.macros.len(), 16);
    assert_eq!(doc.macros, original.macros);
}
