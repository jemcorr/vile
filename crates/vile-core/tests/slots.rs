mod common;

use std::collections::BTreeMap;

use vile_core::ops::OpError;
use vile_core::slots::{self, FeatureSlotKind, FeatureSlotValue, Place, SlotField};
use vile_core::{AltRepeatKey, Document, Keycode};

/// Writes `kc` at one existing keycode path.
fn plant(doc: &mut Document, path: &str, kc: Keycode) {
    let map = BTreeMap::from([(path.to_string(), kc)]);
    doc.replace_keycodes(&map)
        .unwrap_or_else(|e| panic!("cannot plant at {path}: {e:?}"));
}

#[test]
fn is_set_matches_the_slot_editors() {
    let doc = common::load("corne.vil");
    let get = |kind, index| slots::get(&doc, kind, index).unwrap();
    assert!(slots::is_set(&get(FeatureSlotKind::Combo, 0)));
    assert!(!slots::is_set(&get(FeatureSlotKind::Combo, 10)));
    assert!(slots::is_set(&get(FeatureSlotKind::Macro, 0)));
    assert!(!slots::is_set(&get(FeatureSlotKind::Macro, 1)));
    assert!(!slots::is_set(&get(FeatureSlotKind::TapDance, 0)));
    assert!(!slots::is_set(&get(FeatureSlotKind::KeyOverride, 0)));
    assert!(!slots::is_set(&get(FeatureSlotKind::AltRepeat, 0)));

    // An alt repeat key counts as set when its options alone are non-zero.
    let options_only = FeatureSlotValue::AltRepeat(AltRepeatKey {
        keycode: Keycode::NO,
        alt_keycode: Keycode::NO,
        allowed_mods: 0,
        options: 8,
    });
    assert!(slots::is_set(&options_only));
}

#[test]
fn get_reports_the_slot_count() {
    let doc = common::load("corne.vil");
    assert_eq!(
        slots::get(&doc, FeatureSlotKind::Combo, 999),
        Err(OpError::NoSuchSlot {
            section: "combo",
            index: 999,
            count: 32,
        })
    );
}

#[test]
fn every_keycode_path_has_a_place() {
    let doc = common::load("corne.vil");
    for (path, _) in doc.keycodes() {
        assert!(slots::place_of(&path).is_some(), "no place for {path}");
    }
}

#[test]
fn fixture_uses() {
    let doc = common::load("corne.vil");
    let found: Vec<_> = slots::uses(&doc)
        .into_iter()
        .map(|u| (u.kind, u.index, u.path, u.place))
        .collect();
    let key = |layer, row, col| Place::Key { layer, row, col };
    assert_eq!(
        found,
        vec![
            (
                FeatureSlotKind::Macro,
                0,
                "layout[0][6][1]".to_string(),
                key(0, 6, 1)
            ),
            (
                FeatureSlotKind::Macro,
                2,
                "layout[7][4][0]".to_string(),
                key(7, 4, 0)
            ),
            (
                FeatureSlotKind::Macro,
                3,
                "layout[7][5][0]".to_string(),
                key(7, 5, 0)
            ),
        ]
    );
}

#[test]
fn every_field_form_is_found() {
    let forms: [(&str, Place); 10] = [
        (
            "layout[3][0][0]",
            Place::Key {
                layer: 3,
                row: 0,
                col: 0,
            },
        ),
        (
            "encoder_layout[2][1][1]",
            Place::Knob {
                layer: 2,
                knob: 1,
                clockwise: false,
            },
        ),
        (
            "macro[0][0][1]",
            Place::Slot {
                kind: FeatureSlotKind::Macro,
                index: 0,
                field: SlotField::MacroKey { action: 0, key: 0 },
            },
        ),
        (
            "tap_dance[4][2]",
            Place::Slot {
                kind: FeatureSlotKind::TapDance,
                index: 4,
                field: SlotField::TapDance(2),
            },
        ),
        (
            "combo[5][1]",
            Place::Slot {
                kind: FeatureSlotKind::Combo,
                index: 5,
                field: SlotField::ComboKey(1),
            },
        ),
        (
            "combo[5][4]",
            Place::Slot {
                kind: FeatureSlotKind::Combo,
                index: 5,
                field: SlotField::ComboOutput,
            },
        ),
        (
            "key_override[1].replacement",
            Place::Slot {
                kind: FeatureSlotKind::KeyOverride,
                index: 1,
                field: SlotField::OverrideReplacement,
            },
        ),
        (
            "key_override[1].trigger",
            Place::Slot {
                kind: FeatureSlotKind::KeyOverride,
                index: 1,
                field: SlotField::OverrideTrigger,
            },
        ),
        (
            "alt_repeat_key[2].keycode",
            Place::Slot {
                kind: FeatureSlotKind::AltRepeat,
                index: 2,
                field: SlotField::AltKey,
            },
        ),
        (
            "alt_repeat_key[2].alt_keycode",
            Place::Slot {
                kind: FeatureSlotKind::AltRepeat,
                index: 2,
                field: SlotField::AltAlternate,
            },
        ),
    ];
    for (kind, index) in [(FeatureSlotKind::Macro, 2), (FeatureSlotKind::TapDance, 2)] {
        let code = slots::slot_keycode(kind, index).unwrap();
        for (path, expected) in &forms {
            let mut doc = common::load("corne.vil");
            plant(&mut doc, path, code);
            let hits: Vec<_> = slots::uses(&doc)
                .into_iter()
                .filter(|u| u.path == *path)
                .collect();
            assert_eq!(hits.len(), 1, "{path}");
            assert_eq!(
                (hits[0].kind, hits[0].index, hits[0].place),
                (kind, index, *expected),
                "{path}"
            );
        }
    }
}

#[test]
fn dangling_references_are_not_uses() {
    let mut doc = common::load("corne.vil");
    let m20 = common::kc(&doc, "M20");
    let td40 = common::kc(&doc, "TD(40)");
    plant(&mut doc, "layout[0][0][0]", m20);
    plant(&mut doc, "layout[0][0][1]", td40);
    assert!(
        slots::uses(&doc)
            .iter()
            .all(|u| !u.path.starts_with("layout[0][0]"))
    );
}

#[test]
fn other_codes_are_not_uses() {
    let mut doc = common::load("corne.vil");
    for (i, name) in ["DYN_MACRO_PLAY1", "USER00", "FN_MO13", "LT1(KC_A)"]
        .iter()
        .enumerate()
    {
        let kc = common::kc(&doc, name);
        plant(&mut doc, &format!("layout[0][1][{i}]"), kc);
    }
    assert!(
        slots::uses(&doc)
            .iter()
            .all(|u| !u.path.starts_with("layout[0][1]"))
    );
    assert_eq!(
        slots::referenced_slot(Keycode(0x7700)),
        Some((FeatureSlotKind::Macro, 0))
    );
    assert_eq!(
        slots::referenced_slot(Keycode(0x77FF)),
        Some((FeatureSlotKind::Macro, 255))
    );
    assert_eq!(
        slots::referenced_slot(Keycode(0x5700)),
        Some((FeatureSlotKind::TapDance, 0))
    );
    assert_eq!(slots::referenced_slot(Keycode(0x7C56)), None);
}

#[test]
fn slot_keycode_round_trips() {
    assert_eq!(
        slots::slot_keycode(FeatureSlotKind::Macro, 255),
        Some(Keycode(0x77FF))
    );
    assert_eq!(
        slots::slot_keycode(FeatureSlotKind::TapDance, 0),
        Some(Keycode(0x5700))
    );
    assert_eq!(slots::slot_keycode(FeatureSlotKind::Macro, 256), None);
    assert_eq!(slots::slot_keycode(FeatureSlotKind::Combo, 0), None);
    for index in [0, 7, 255] {
        let kc = slots::slot_keycode(FeatureSlotKind::TapDance, index).unwrap();
        assert_eq!(
            slots::referenced_slot(kc),
            Some((FeatureSlotKind::TapDance, index))
        );
    }
}
