mod common;

use common::kc;
use vile_core::diff::{diff, format_diff};
use vile_core::{DiffSection, Keycode, Macro};

#[test]
fn identical_documents_have_an_empty_diff() {
    let doc = common::load("corne.vil");
    assert!(diff(&doc, &doc.clone(), Some(&common::board())).is_empty());
    assert_eq!(format_diff(&[]), "");
}

#[test]
fn key_change_is_one_semantic_line() {
    let old = common::load("corne_v32.vil");
    let mut new = old.clone();
    let lt = kc(&new, "LT8(KC_ESCAPE)");
    new.set(0, 3, 4, lt).unwrap();
    let entries = diff(&old, &new, Some(&common::board()));
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].section, DiffSection::Layout);
    assert_eq!(entries[0].layer, Some(0));
    assert_eq!(entries[0].key_id.as_deref(), Some("thumb-left-mid"));
    assert_eq!(entries[0].old, "MO(7)");
    assert_eq!(entries[0].new, "LT8(KC_ESCAPE)");
    assert_eq!(
        format_diff(&entries),
        "L0 thumb-left-mid: MO(7) -> LT8(KC_ESCAPE)\n"
    );
}

#[test]
fn without_a_board_positions_use_matrix_names() {
    let old = common::load("corne_v32.vil");
    let mut new = old.clone();
    new.set(0, 3, 4, Keycode::NO).unwrap();
    assert_eq!(
        format_diff(&diff(&old, &new, None)),
        "L0 r3c4: MO(7) -> KC_NO\n"
    );
}

#[test]
fn layout_lines_are_ordered_by_layer_then_board_order() {
    let old = common::load("corne_v32.vil");
    let mut new = old.clone();
    new.set(1, 0, 0, Keycode::NO).unwrap(); // L1 top-left-outer
    new.set(0, 7, 3, Keycode::TRNS).unwrap(); // L0 thumb-right-outer
    new.set(0, 0, 1, Keycode::TRNS).unwrap(); // L0 top-left-pinky
    let text = format_diff(&diff(&old, &new, Some(&common::board())));
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 3, "{text}");
    assert!(lines[0].starts_with("L0 top-left-pinky: "), "{text}");
    assert!(lines[1].starts_with("L0 thumb-right-outer: "), "{text}");
    assert!(
        lines[2].starts_with("L1 top-left-outer: KC_TRNS -> KC_NO"),
        "{text}"
    );
}

#[test]
fn other_sections_have_their_own_lines() {
    let old = common::load("corne_v32.vil");
    let mut new = old.clone();
    new.macros[1] = Macro::default();
    new.tap_dance.as_mut().unwrap()[0].tapping_term = 200;
    new.tap_dance.as_mut().unwrap()[0].on_tap = Keycode::NO;
    new.combo.as_mut().unwrap()[4].output = Keycode::NO;
    new.key_override.as_mut().unwrap()[2].options = 135;
    new.alt_repeat_key.as_mut().unwrap()[3].keycode = kc(&new, "KC_A");
    new.settings.as_mut().unwrap()[0].value = 5;
    new.encoder_layout.as_mut().unwrap()[1][2][1] = vile_core::Slot::Key(Keycode::NO);
    new.layout_options = 1;
    let text = format_diff(&diff(&old, &new, Some(&common::board())));
    let expected = [
        "layout_options: 0 -> 1",
        "L1 encoder 2 ccw: KC_TRNS -> KC_NO",
        "M1: tap LSFT(KC_QUOTE) KC_SPACE -> (empty)",
        "TD(0) tap: KC_F -> KC_NO",
        "TD(0) term: 80 -> 200",
        "combo 4: LGUI_T(KC_S) + RGUI_T(KC_H) => KC_QUOTE -> LGUI_T(KC_S) + RGUI_T(KC_H) => KC_NO",
        "key_override 2 options: 7 -> 135",
        "alt_repeat_key 3 keycode: KC_NO -> KC_A",
        "setting 1: 4 -> 5",
    ];
    assert_eq!(text, expected.join("\n") + "\n");
}

#[test]
fn macro_text_is_quoted() {
    let old = common::load("corne_v32.vil");
    let mut new = old.clone();
    new.macros[3] = Macro {
        actions: vec![
            vile_core::MacroAction::Text("say \"hi\"".to_string()),
            vile_core::MacroAction::Delay(100),
        ],
    };
    let text = format_diff(&diff(&old, &new, None));
    assert_eq!(
        text,
        "M3: text \"//\" -> text \"say \\\"hi\\\"\"; delay 100\n"
    );
}
