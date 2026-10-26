mod common;

use vile_core::ops::OpError;
use vile_core::slots::{self, FeatureSlotKind, FeatureSlotValue};
use vile_core::{AltRepeatKey, Clipboard, Editor, KeyOverride, TapDance};
use vile_core::{Combo, Keycode, Macro, MacroAction, ops};

#[test]
fn set_combo_and_bounds() {
    let mut doc = common::load("corne.vil");
    let cat = doc.catalog();
    let k = |s: &str| cat.parse(s).unwrap();
    let combo = Combo {
        keys: [k("KC_J"), k("KC_K"), Keycode::NO, Keycode::NO],
        output: k("KC_ESCAPE"),
    };
    ops::set_combo(&mut doc, 5, combo).unwrap();
    assert_eq!(doc.combo.as_ref().unwrap()[5], combo);
    assert!(ops::set_combo(&mut doc, 32, combo).is_err());
}

#[test]
fn set_macro_validates_contents() {
    let mut doc = common::load("corne.vil");
    let a = common::kc(&doc, "KC_A");
    let m = Macro {
        actions: vec![
            MacroAction::Down(vec![a]),
            MacroAction::Delay(50),
            MacroAction::Up(vec![a]),
        ],
    };
    ops::set_macro(&mut doc, 4, m.clone()).unwrap();
    assert_eq!(doc.macros[4], m);
    assert_eq!(doc.macros.len(), 16);
    assert!(ops::set_macro(&mut doc, 16, m.clone()).is_err());
    assert!(
        ops::set_macro(
            &mut doc,
            4,
            Macro {
                actions: vec![MacroAction::Text("a\u{0}b".to_string())]
            }
        )
        .is_err()
    );
    assert!(
        ops::set_macro(
            &mut doc,
            4,
            Macro {
                actions: vec![MacroAction::Delay(65025)]
            }
        )
        .is_err()
    );
    let out = String::from_utf8(doc.to_vil_bytes()).unwrap();
    assert!(
        out.contains("[[\"down\", \"KC_A\"], [\"delay\", 50], [\"up\", \"KC_A\"]]"),
        "{out}"
    );
}

#[test]
fn set_tap_dance_key_override_and_alt_repeat() {
    let mut doc = common::load("corne.vil");
    let cat = doc.catalog();
    let k = |s: &str| cat.parse(s).unwrap();
    let mut td = doc.tap_dance.as_ref().unwrap()[0];
    td.on_tap = k("KC_F");
    td.on_hold = k("KC_LGUI");
    td.tapping_term = 180;
    ops::set_tap_dance(&mut doc, 0, td).unwrap();
    assert_eq!(doc.tap_dance.as_ref().unwrap()[0], td);
    let mut ko = doc.key_override.as_ref().unwrap()[1];
    ko.trigger = k("KC_BSPACE");
    ko.replacement = k("KC_DELETE");
    ko.trigger_mods = 0x02;
    ko.options = 0x87;
    ops::set_key_override(&mut doc, 1, ko).unwrap();
    assert_eq!(doc.key_override.as_ref().unwrap()[1], ko);
    let mut ar = doc.alt_repeat_key.as_ref().unwrap()[2];
    ar.keycode = k("KC_A");
    ar.alt_keycode = k("KC_B");
    ar.options = 0x08;
    ops::set_alt_repeat_key(&mut doc, 2, ar).unwrap();
    assert_eq!(doc.alt_repeat_key.as_ref().unwrap()[2], ar);
    assert!(ops::set_tap_dance(&mut doc, 32, td).is_err());
    assert!(ops::set_key_override(&mut doc, 32, ko).is_err());
    assert!(ops::set_alt_repeat_key(&mut doc, 32, ar).is_err());
    let (reloaded, report) = vile_core::Document::from_vil_bytes(&doc.to_vil_bytes()).unwrap();
    assert!(report.canonical);
    assert_eq!(reloaded, doc);
}

#[test]
fn clear_resets_to_vial_empty_values() {
    let doc = common::load("corne.vil");
    let key_a = common::kc(&doc, "KC_A");
    let key_b = common::kc(&doc, "KC_B");
    let cases = [
        (
            FeatureSlotKind::Combo,
            FeatureSlotValue::Combo(Combo {
                keys: [key_a, Keycode::NO, Keycode::NO, Keycode::NO],
                output: key_b,
            }),
        ),
        (
            FeatureSlotKind::Macro,
            FeatureSlotValue::Macro(Macro {
                actions: vec![MacroAction::Tap(vec![key_a])],
            }),
        ),
        (
            FeatureSlotKind::TapDance,
            FeatureSlotValue::TapDance(TapDance {
                on_tap: key_a,
                on_hold: Keycode::NO,
                on_double_tap: Keycode::NO,
                on_tap_hold: Keycode::NO,
                tapping_term: 80,
            }),
        ),
        (
            FeatureSlotKind::KeyOverride,
            FeatureSlotValue::KeyOverride(KeyOverride {
                trigger: key_a,
                replacement: key_b,
                layers: 0xFFFF,
                trigger_mods: 0,
                negative_mod_mask: 0,
                suppressed_mods: 0,
                options: 135,
            }),
        ),
        (
            FeatureSlotKind::AltRepeat,
            FeatureSlotValue::AltRepeat(AltRepeatKey {
                keycode: key_a,
                alt_keycode: key_b,
                allowed_mods: 0,
                options: 8,
            }),
        ),
    ];
    for (kind, value) in cases {
        let mut ed = Editor::new(doc.clone(), None, None);
        ed.apply(|d| slots::set(d, kind, 0, value)).unwrap();
        ed.apply_named(|_, names| {
            slots::set_name(names, kind, 0, Some("x".into()));
            Ok(Vec::new())
        })
        .unwrap();
        let named = (ed.doc().clone(), ed.names().clone());
        ed.clear_feature_slots(kind, &[0, 1]).unwrap();
        assert_eq!(slots::get(ed.doc(), kind, 0).unwrap(), slots::empty(kind));
        assert_eq!(slots::get(ed.doc(), kind, 1).unwrap(), slots::empty(kind));
        assert!(slots::name_of(ed.names(), kind, 0).is_none());
        assert!(ed.undo());
        assert_eq!((ed.doc().clone(), ed.names().clone()), named);
    }
}

#[test]
fn clearing_an_empty_slot_adds_no_undo_step() {
    let mut ed = Editor::new(common::load("corne.vil"), None, None);
    assert_eq!(
        slots::get(ed.doc(), FeatureSlotKind::Combo, 10).unwrap(),
        slots::empty(FeatureSlotKind::Combo)
    );
    ed.clear_feature_slots(FeatureSlotKind::Combo, &[10])
        .unwrap();
    assert!(!ed.can_undo());
}

#[test]
fn clear_keeps_references() {
    let mut ed = Editor::new(common::load("corne.vil"), None, None);
    ed.clear_feature_slots(FeatureSlotKind::Macro, &[2])
        .unwrap();
    assert_eq!(common::wire_at(ed.doc(), 7, 4, 0), "M2");
    assert!(ed.doc().macros[2].actions.is_empty());
}

#[test]
fn cut_copies_then_clears() {
    let original = common::load("corne.vil");
    let mut ed = Editor::new(original.clone(), None, None);
    ed.cut_feature_slots(FeatureSlotKind::Combo, &[2, 0])
        .unwrap();
    match ed.clipboard() {
        Some(Clipboard::Features { kind, entries }) => {
            assert_eq!(*kind, FeatureSlotKind::Combo);
            assert_eq!(entries.len(), 2);
            assert_eq!(
                entries[0].0,
                slots::value(&original, FeatureSlotKind::Combo, 0).unwrap()
            );
            assert_eq!(
                entries[1].0,
                slots::value(&original, FeatureSlotKind::Combo, 2).unwrap()
            );
        }
        other => panic!("unexpected clipboard {other:?}"),
    }
    assert_eq!(
        slots::get(ed.doc(), FeatureSlotKind::Combo, 0).unwrap(),
        slots::empty(FeatureSlotKind::Combo)
    );
    assert_eq!(
        slots::get(ed.doc(), FeatureSlotKind::Combo, 2).unwrap(),
        slots::empty(FeatureSlotKind::Combo)
    );
    assert!(ed.undo());
    assert!(!ed.can_undo());
    assert_eq!(ed.doc(), &original);
    ed.paste_feature_slots(FeatureSlotKind::Combo, &[10])
        .unwrap();
    assert_eq!(
        slots::get(ed.doc(), FeatureSlotKind::Combo, 10).unwrap(),
        slots::value(&original, FeatureSlotKind::Combo, 0).unwrap()
    );
    assert_eq!(
        slots::get(ed.doc(), FeatureSlotKind::Combo, 11).unwrap(),
        slots::value(&original, FeatureSlotKind::Combo, 2).unwrap()
    );
}

#[test]
fn paste_targets_rules() {
    use slots::paste_targets as targets;
    assert_eq!(targets(2, &[5], 32).unwrap(), vec![5, 6]);
    assert_eq!(targets(2, &[5, 9], 32).unwrap(), vec![5, 9]);
    assert_eq!(targets(2, &[9, 5], 32).unwrap(), vec![5, 9]);
    assert_eq!(
        targets(2, &[1, 2, 3], 32),
        Err(OpError::SizeMismatch {
            expected: 2,
            actual: 3
        })
    );
    assert!(matches!(targets(2, &[31], 32), Err(OpError::Invalid(_))));
    assert!(matches!(targets(2, &[], 32), Err(OpError::Invalid(_))));
}

#[test]
fn copy_to_leaves_the_clipboard() {
    let original = common::load("corne.vil");
    let mut ed = Editor::new(original.clone(), None, None);
    ed.copy_feature_slots(FeatureSlotKind::Combo, &[3]).unwrap();
    ed.copy_feature_slots_to(FeatureSlotKind::Combo, &[0], &[7])
        .unwrap();
    assert_eq!(
        slots::get(ed.doc(), FeatureSlotKind::Combo, 7).unwrap(),
        slots::value(&original, FeatureSlotKind::Combo, 0).unwrap()
    );
    match ed.clipboard() {
        Some(Clipboard::Features { entries, .. }) => assert_eq!(
            entries[0].0,
            slots::value(&original, FeatureSlotKind::Combo, 3).unwrap()
        ),
        other => panic!("unexpected clipboard {other:?}"),
    }
    ed.copy_feature_slots_to(FeatureSlotKind::Combo, &[0, 1], &[1, 2])
        .unwrap();
    assert_eq!(
        slots::get(ed.doc(), FeatureSlotKind::Combo, 1).unwrap(),
        slots::value(&original, FeatureSlotKind::Combo, 0).unwrap()
    );
    assert_eq!(
        slots::get(ed.doc(), FeatureSlotKind::Combo, 2).unwrap(),
        slots::value(&original, FeatureSlotKind::Combo, 1).unwrap()
    );
}

#[test]
fn enabled_bits() {
    let mut ed = Editor::new(common::load("corne.vil"), None, None);
    ed.set_feature_slots_enabled(FeatureSlotKind::KeyOverride, &[0], true)
        .unwrap();
    assert_eq!(ed.doc().key_override.as_ref().unwrap()[0].options, 7 | 0x80);
    ed.set_feature_slots_enabled(FeatureSlotKind::KeyOverride, &[0], false)
        .unwrap();
    assert_eq!(ed.doc().key_override.as_ref().unwrap()[0].options, 7);
    ed.set_feature_slots_enabled(FeatureSlotKind::AltRepeat, &[0], true)
        .unwrap();
    assert_eq!(ed.doc().alt_repeat_key.as_ref().unwrap()[0].options, 8);
    assert_eq!(
        ed.set_feature_slots_enabled(FeatureSlotKind::Combo, &[0], true),
        Err(OpError::Invalid("a combo has no Enabled switch".into()))
    );
}

#[test]
fn wrong_clipboard_messages() {
    let mut ed = Editor::new(common::load("corne.vil"), None, None);
    ed.copy_feature_slots(FeatureSlotKind::Macro, &[2, 3])
        .unwrap();
    assert_eq!(
        ed.paste_feature_slots(FeatureSlotKind::Combo, &[0]),
        Err(OpError::WrongClipboard {
            held: "2 macros".into(),
            wanted: "a combo",
        })
    );
}
