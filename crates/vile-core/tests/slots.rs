mod common;

use vile_core::ops::OpError;
use vile_core::slots::{self, FeatureSlotKind, FeatureSlotValue};
use vile_core::{AltRepeatKey, Keycode};

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
