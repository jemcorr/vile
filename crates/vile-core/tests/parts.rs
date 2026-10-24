use vile_core::Keycode;
use vile_core::keycode::{HoldBehavior, KeyParts, KeyPartsPatch, compose, decompose};

#[test]
fn every_numeric_keycode_round_trips_losslessly() {
    for code in 0..=u16::MAX {
        assert_eq!(
            compose(decompose(Keycode(code))),
            Ok(Keycode(code)),
            "{code:04x}"
        );
    }
}
#[test]
fn patches_preserve_the_unedited_parts_and_reject_invalid_combinations() {
    let original = Keycode(0x2800 | 0x16);
    assert_eq!(
        KeyPartsPatch {
            base: Some(7),
            ..Default::default()
        }
        .apply(original),
        Ok(Keycode(0x2807))
    );
    assert!(
        KeyPartsPatch {
            base: Some(0x7700),
            ..Default::default()
        }
        .apply(original)
        .is_err()
    );
    assert!(
        compose(KeyParts::Tap {
            base: 4,
            wrap_mods: 2,
            hold: HoldBehavior::Mod { mods: 8 }
        })
        .is_err()
    );
    assert!(
        compose(KeyParts::Tap {
            base: 4,
            wrap_mods: 0,
            hold: HoldBehavior::Layer { layer: 16 }
        })
        .is_err()
    );
    assert!(compose(KeyParts::OneShot { mods: 0x20 }).is_err());
    assert!(compose(KeyParts::OneShot { mods: 0x10 }).is_err());
}
