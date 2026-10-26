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

#[test]
fn feature_clipboard_is_typed_and_paste_undoes_content_and_name_together() {
    let (_dir, mut s) = open("corne_v32.vil");
    s.apply_op(OpDto::Rename {
        kind: "combo".into(),
        index: 0,
        name: "dead acute".into(),
    })
    .unwrap();
    s.apply_op(OpDto::Rename {
        kind: "combo".into(),
        index: 1,
        name: "navigation".into(),
    })
    .unwrap();
    let before = s.view().unwrap();
    s.apply_op(OpDto::CopyFeatureSlot {
        kind: "combo".into(),
        index: 0,
    })
    .unwrap();
    assert!(
        s.apply_op(OpDto::PasteFeatureSlot {
            kind: "macro".into(),
            index: 1
        })
        .is_err()
    );
    s.apply_op(OpDto::PasteFeatureSlot {
        kind: "combo".into(),
        index: 1,
    })
    .unwrap();
    let pasted = s.view().unwrap();
    assert_eq!(pasted.combos[1].keys, before.combos[0].keys);
    assert_eq!(pasted.combos[1].output, before.combos[0].output);
    assert_eq!(pasted.names.combos[&1], "dead acute");
    s.undo().unwrap();
    let undone = s.view().unwrap();
    assert_eq!(undone.combos[1], before.combos[1]);
    assert_eq!(undone.names.combos[&1], "navigation");

    s.apply_op(OpDto::Rename {
        kind: "macro".into(),
        index: 0,
        name: "Jupyter cell".into(),
    })
    .unwrap();
    s.apply_op(OpDto::Rename {
        kind: "macro".into(),
        index: 1,
        name: "close cell".into(),
    })
    .unwrap();
    let before = s.view().unwrap();
    s.apply_op(OpDto::SwapFeatureSlots {
        kind: "macro".into(),
        a: 0,
        b: 1,
    })
    .unwrap();
    let swapped = s.view().unwrap();
    assert_eq!(swapped.macros[0].actions, before.macros[1].actions);
    assert_eq!(swapped.macros[1].actions, before.macros[0].actions);
    let keycodes = |view: &vile_session::ViewModel| {
        view.layers
            .iter()
            .flat_map(|layer| layer.iter().map(|cell| cell.wire.clone()))
            .collect::<Vec<_>>()
    };
    // Swapping renumbers references: the M0 key on layer 0 now plays slot 1, and nothing else moves.
    let renumbered: Vec<String> = keycodes(&before)
        .into_iter()
        .map(|wire| if wire == "M0" { "M1".to_string() } else { wire })
        .collect();
    assert_eq!(keycodes(&swapped), renumbered);
    assert_eq!(
        swapped.layers[0]
            .iter()
            .find(|cell| cell.id == "r6c1")
            .unwrap()
            .wire,
        "M1"
    );
    assert_eq!(swapped.names.macros[&0], "close cell");
    assert_eq!(swapped.names.macros[&1], "Jupyter cell");
    s.undo().unwrap();
    let restored = s.view().unwrap();
    assert_eq!(restored.macros[0].actions, before.macros[0].actions);
    assert_eq!(restored.macros[1].actions, before.macros[1].actions);
    assert_eq!(restored.names.macros, before.names.macros);
    assert!(
        s.apply_op(OpDto::SwapFeatureSlots {
            kind: "combo".into(),
            a: 0,
            b: 999
        })
        .is_err()
    );
}

#[test]
fn feature_slot_operations_support_all_fixed_index_families() {
    let (_dir, mut s) = open("corne_v32.vil");
    for kind in ["tapDance", "keyOverride", "altRepeat"] {
        let before = s.view().unwrap();
        s.apply_op(OpDto::CopyFeatureSlot {
            kind: kind.into(),
            index: 0,
        })
        .unwrap();
        s.apply_op(OpDto::PasteFeatureSlot {
            kind: kind.into(),
            index: 1,
        })
        .unwrap();
        let after = s.view().unwrap();
        match kind {
            "tapDance" => {
                assert_eq!(after.tap_dances[1].on_tap, before.tap_dances[0].on_tap);
                assert_eq!(
                    after.tap_dances[1].tapping_term,
                    before.tap_dances[0].tapping_term
                );
            }
            "keyOverride" => {
                assert_eq!(
                    after.key_overrides[1].trigger,
                    before.key_overrides[0].trigger
                );
                assert_eq!(
                    after.key_overrides[1].options,
                    before.key_overrides[0].options
                );
            }
            "altRepeat" => {
                assert_eq!(
                    after.alt_repeat_keys[1].keycode,
                    before.alt_repeat_keys[0].keycode
                );
                assert_eq!(
                    after.alt_repeat_keys[1].options,
                    before.alt_repeat_keys[0].options
                );
            }
            _ => unreachable!(),
        }
        s.undo().unwrap();
    }
}
