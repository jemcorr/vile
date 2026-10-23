mod common;

use vile_session::{MacroActionDto, OpDto, Session};

fn open(name: &str) -> (tempfile::TempDir, Session) {
    let (dir, path) = common::temp_copy(name);
    let mut s = Session::new();
    s.open_path(&path).unwrap();
    (dir, s)
}

fn strings(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn combos() {
    let (_dir, mut s) = open("corne_v32.vil");
    let v = s
        .apply_op(OpDto::SetCombo {
            index: 6,
            keys: strings(&["j", "k", "KC_NO", "KC_NO"]),
            output: "esc".to_string(),
        })
        .unwrap();
    assert_eq!(
        v.combos[6].keys,
        strings(&["KC_J", "KC_K", "KC_NO", "KC_NO"])
    );
    assert_eq!(v.combos[6].output, "KC_ESCAPE");
    assert_eq!(v.combos[6].output_label, "Esc");
    assert!(
        s.apply_op(OpDto::SetCombo {
            index: 6,
            keys: strings(&["j", "k"]),
            output: "esc".to_string()
        })
        .is_err()
    );
}

#[test]
fn macros() {
    let (_dir, mut s) = open("corne_v32.vil");
    let actions = vec![
        MacroActionDto {
            kind: "tap".to_string(),
            keys: strings(&["lsft(quote)", "space"]),
            text: String::new(),
            delay: 0,
        },
        MacroActionDto {
            kind: "delay".to_string(),
            keys: vec![],
            text: String::new(),
            delay: 20,
        },
        MacroActionDto {
            kind: "text".to_string(),
            keys: vec![],
            text: "ok".to_string(),
            delay: 0,
        },
    ];
    let v = s.apply_op(OpDto::SetMacro { index: 1, actions }).unwrap();
    assert_eq!(v.macros.len(), 16);
    assert_eq!(
        v.macros[1].actions[0].keys,
        strings(&["LSFT(KC_QUOTE)", "KC_SPACE"])
    );
    assert_eq!(v.macros[1].actions[1].delay, 20);
    assert_eq!(v.macros[1].actions[2].text, "ok");
    let bad = vec![MacroActionDto {
        kind: "jump".to_string(),
        keys: vec![],
        text: String::new(),
        delay: 0,
    }];
    assert!(
        s.apply_op(OpDto::SetMacro {
            index: 1,
            actions: bad
        })
        .is_err()
    );
}

#[test]
fn tap_dance_key_override_and_alt_repeat() {
    let (_dir, mut s) = open("corne_v32.vil");
    let v = s
        .apply_op(OpDto::SetTapDance {
            index: 10,
            on_tap: "f".to_string(),
            on_hold: "KC_LGUI".to_string(),
            on_double_tap: "KC_NO".to_string(),
            on_tap_hold: "KC_NO".to_string(),
            tapping_term: 180,
        })
        .unwrap();
    assert_eq!(v.tap_dances[10].on_tap, "KC_F");
    assert_eq!(v.tap_dances[10].tapping_term, 180);
    let v = s
        .apply_op(OpDto::SetKeyOverride {
            index: 3,
            trigger: "bspc".to_string(),
            replacement: "del".to_string(),
            layers: 0xFFFF,
            trigger_mods: 0x02,
            negative_mod_mask: 0,
            suppressed_mods: 0x02,
            options: 0x87,
        })
        .unwrap();
    assert_eq!(v.key_overrides[3].trigger, "KC_BSPACE");
    assert_eq!(v.key_overrides[3].replacement, "KC_DELETE");
    assert_eq!(v.key_overrides[3].options, 0x87);
    let v = s
        .apply_op(OpDto::SetAltRepeat {
            index: 2,
            keycode: "a".to_string(),
            alt_keycode: "b".to_string(),
            allowed_mods: 0,
            options: 0x08,
        })
        .unwrap();
    assert_eq!(v.alt_repeat_keys[2].keycode, "KC_A");
    assert_eq!(v.alt_repeat_keys[2].alt_keycode, "KC_B");
    let text = v
        .diff
        .iter()
        .map(|d| d.line.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("TD(10) tap: KC_NO -> KC_F"), "{text}");
    assert!(
        text.contains("key_override 3 trigger: KC_NO -> KC_BSPACE"),
        "{text}"
    );
    assert!(
        text.contains("alt_repeat_key 2 alt_keycode: KC_NO -> KC_B"),
        "{text}"
    );
}
