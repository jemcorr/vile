mod common;

use vile_core::{KeyRef, Keycode, Pos, ops};

fn pos(id: &str) -> Pos {
    let b = common::board();
    let k = b.key_by_id(id).unwrap_or_else(|| panic!("no key {id}"));
    Pos {
        row: k.row,
        col: k.col,
    }
}

#[test]
fn fill_clear_and_transparent_change_only_the_selection() {
    let original = common::load("corne_v32.vil");
    let mut doc = original.clone();
    let sel = [pos("home-left-pinky"), pos("home-left-ring")];
    let a = common::kc(&doc, "KC_A");
    ops::set_keys(&mut doc, 3, &sel, a).unwrap();
    assert_eq!(ops::read_keys(&doc, 3, &sel).unwrap(), vec![a, a]);
    ops::clear_keys(&mut doc, 3, &sel).unwrap();
    assert_eq!(
        ops::read_keys(&doc, 3, &sel).unwrap(),
        vec![Keycode::NO, Keycode::NO]
    );
    ops::transparent_keys(&mut doc, 3, &sel).unwrap();
    assert_eq!(
        ops::read_keys(&doc, 3, &sel).unwrap(),
        vec![Keycode::TRNS, Keycode::TRNS]
    );
    // Layer 3 of the fixture is fully transparent, so the document is back to the original.
    assert_eq!(doc, original);
}

#[test]
fn absent_positions_and_missing_layers_are_rejected() {
    let mut doc = common::load("corne.vil");
    let before = doc.clone();
    let a = common::kc(&doc, "KC_A");
    assert!(
        ops::set_keys(
            &mut doc,
            0,
            &[pos("top-left-outer"), Pos { row: 3, col: 0 }],
            a
        )
        .is_err()
    );
    assert!(ops::set_keys(&mut doc, 10, &[pos("top-left-outer")], a).is_err());
    assert!(ops::read_keys(&doc, 0, &[Pos { row: 9, col: 0 }]).is_err());
    assert_eq!(
        doc, before,
        "a failed operation must not change the document"
    );
}

#[test]
fn write_keys_checks_lengths() {
    let mut doc = common::load("corne.vil");
    let sel = [pos("top-left-ring"), pos("top-left-middle")];
    assert!(ops::write_keys(&mut doc, 0, &sel, &[Keycode::NO]).is_err());
    ops::write_keys(&mut doc, 0, &sel, &[Keycode::NO, Keycode::TRNS]).unwrap();
    assert_eq!(
        ops::read_keys(&doc, 0, &sel).unwrap(),
        vec![Keycode::NO, Keycode::TRNS]
    );
}

#[test]
fn swap_keys_across_layers() {
    let mut doc = common::load("corne_v32.vil");
    let a = [KeyRef {
        layer: 0,
        pos: pos("thumb-left-mid"),
    }];
    let b = [KeyRef {
        layer: 9,
        pos: pos("top-left-pinky"),
    }];
    let ka = ops::read_keys(&doc, 0, &[a[0].pos]).unwrap()[0];
    let kb = ops::read_keys(&doc, 9, &[b[0].pos]).unwrap()[0];
    ops::swap_keys(&mut doc, &a, &b).unwrap();
    assert_eq!(ops::read_keys(&doc, 0, &[a[0].pos]).unwrap()[0], kb);
    assert_eq!(ops::read_keys(&doc, 9, &[b[0].pos]).unwrap()[0], ka);
}

#[test]
fn swap_keys_rejects_bad_selections() {
    let mut doc = common::load("corne.vil");
    let x = KeyRef {
        layer: 0,
        pos: pos("top-left-ring"),
    };
    let y = KeyRef {
        layer: 0,
        pos: pos("top-left-middle"),
    };
    assert!(ops::swap_keys(&mut doc, &[x], &[x, y]).is_err());
    assert!(ops::swap_keys(&mut doc, &[x, y], &[y, x]).is_err());
}

#[test]
fn paste_targets_rules() {
    let copied = [pos("home-left-pinky"), pos("home-left-ring")];
    assert_eq!(ops::paste_targets(&copied, &[]).unwrap(), copied.to_vec());
    assert_eq!(
        ops::paste_targets(&copied, &[copied[1], copied[0]]).unwrap(),
        copied.to_vec()
    );
    let other = [pos("top-left-pinky"), pos("top-left-ring")];
    assert_eq!(ops::paste_targets(&copied, &other).unwrap(), other.to_vec());
    assert!(ops::paste_targets(&copied, &[pos("top-left-pinky")]).is_err());
}
