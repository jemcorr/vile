mod common;

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
