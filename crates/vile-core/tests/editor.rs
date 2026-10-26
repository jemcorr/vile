mod common;

use vile_core::{Editor, Keycode, Pos, ops};

fn pos(id: &str) -> Pos {
    let b = common::board();
    let k = b.key_by_id(id).unwrap();
    Pos {
        row: k.row,
        col: k.col,
    }
}

#[test]
fn layer_clear_followed_by_undo_restores_the_original() {
    let original = common::load("corne.vil");
    let mut ed = Editor::new(original.clone(), None, Some(common::board()));
    assert!(!ed.can_undo());
    ed.apply(|d| ops::fill_layer(d, 0, Keycode::NO)).unwrap();
    assert_ne!(ed.doc(), &original);
    assert!(ed.is_dirty());
    assert!(ed.can_undo());
    assert!(ed.undo());
    assert_eq!(ed.doc(), &original);
    assert_eq!(ed.doc().to_vil_bytes(), common::fixture_bytes("corne.vil"));
    assert!(!ed.is_dirty());
    assert!(ed.can_redo());
    assert!(ed.redo());
    assert!(ed.undo());
    assert_eq!(ed.doc(), &original);
    assert!(!ed.undo(), "nothing left to undo");
}

#[test]
fn failed_operation_leaves_history_untouched() {
    let mut ed = Editor::new(common::load("corne.vil"), None, None);
    assert!(ed.apply(|d| ops::fill_layer(d, 42, Keycode::NO)).is_err());
    assert!(!ed.can_undo());
    assert!(!ed.is_dirty());
}

#[test]
fn no_op_edit_does_not_create_history() {
    let mut ed = Editor::new(common::load("corne_v32.vil"), None, None);
    ed.apply(|d| ops::fill_layer(d, 3, Keycode::TRNS)).unwrap(); // layer 3 is already all KC_TRNS
    assert!(!ed.can_undo());
}

#[test]
fn new_edit_clears_redo() {
    let mut ed = Editor::new(common::load("corne.vil"), None, None);
    ed.apply(|d| ops::fill_layer(d, 0, Keycode::NO)).unwrap();
    assert!(ed.undo());
    ed.apply(|d| ops::fill_layer(d, 1, Keycode::NO)).unwrap();
    assert!(!ed.can_redo());
}

#[test]
fn undo_history_is_capped() {
    let mut ed = Editor::new(common::load("corne.vil"), None, None);
    let a = common::kc(ed.doc(), "KC_A");
    let b = common::kc(ed.doc(), "KC_B");
    for i in 0..(vile_core::editor::UNDO_LIMIT + 10) {
        let kc = if i % 2 == 0 { a } else { b };
        ed.apply(|d| ops::set_keys(d, 2, &[pos("top-left-outer")], kc))
            .unwrap();
    }
    let mut undone = 0;
    while ed.undo() {
        undone += 1;
    }
    assert_eq!(undone, vile_core::editor::UNDO_LIMIT);
}

#[test]
fn copy_and_paste_keys_through_the_clipboard() {
    let mut ed = Editor::new(common::load("corne.vil"), None, Some(common::board()));
    let sel = [pos("home-left-pinky"), pos("home-left-ring")];
    assert!(ed.paste_keys(4, &[]).is_err(), "empty clipboard");
    ed.copy_keys(0, &sel).unwrap();
    ed.paste_keys(4, &[]).unwrap();
    assert_eq!(
        ops::read_keys(ed.doc(), 4, &sel).unwrap(),
        ops::read_keys(ed.doc(), 0, &sel).unwrap()
    );
    assert!(
        ed.paste_keys(4, &[pos("top-left-outer")]).is_err(),
        "size mismatch"
    );
    let other = [pos("top-left-pinky"), pos("top-left-ring")];
    ed.paste_keys(5, &other).unwrap();
    assert_eq!(
        ops::read_keys(ed.doc(), 5, &other).unwrap(),
        ops::read_keys(ed.doc(), 0, &sel).unwrap()
    );
    assert!(
        ed.paste_layer(1).is_err(),
        "clipboard holds keys, not a layer"
    );
}

#[test]
fn copy_and_paste_a_layer() {
    let original = common::load("corne.vil");
    let mut ed = Editor::new(original.clone(), None, None);
    ed.copy_layer(0).unwrap();
    ed.paste_layer(5).unwrap();
    assert_eq!(ed.doc().layout[5], ed.doc().layout[0]);
    assert!(ed.undo());
    assert_eq!(ed.doc().layout[5], original.layout[5]);
}

#[test]
fn swapping_slots_is_one_undo_step() {
    let mut ed = Editor::new(common::load("corne.vil"), None, None);
    ed.apply_named(|_, names| {
        names.macros.insert(2, "a".into());
        names.macros.insert(3, "b".into());
        Ok(Vec::new())
    })
    .unwrap();
    let named = (ed.doc().clone(), ed.names().clone());
    ed.swap_feature_slots(vile_core::FeatureSlotKind::Macro, 2, 3)
        .unwrap();
    let swapped = (ed.doc().clone(), ed.names().clone());
    assert_eq!(swapped.1.macros[&2], "b");
    assert_eq!(swapped.1.macros[&3], "a");
    assert!(ed.undo());
    assert_eq!((ed.doc().clone(), ed.names().clone()), named);
    assert!(ed.redo());
    assert_eq!((ed.doc().clone(), ed.names().clone()), swapped);
}

#[test]
fn diff_is_against_the_base() {
    let mut ed = Editor::new(common::load("corne_v32.vil"), None, Some(common::board()));
    let lt = common::kc(ed.doc(), "LT8(KC_ESCAPE)");
    ed.apply(|d| ops::set_keys(d, 0, &[pos("thumb-left-mid")], lt))
        .unwrap();
    let lines: Vec<String> = ed.diff().into_iter().map(|e| e.line).collect();
    assert_eq!(
        lines,
        vec!["L0 thumb-left-mid: MO(7) -> LT8(KC_ESCAPE)".to_string()]
    );
}
