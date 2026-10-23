mod common;

use common::wire_at;
use vile_core::{Board, MirrorMode, ops};

#[test]
fn move_layer_renumbers_references() {
    let mut doc = common::load("corne_v32.vil");
    let old_l9 = doc.layout[9].clone();
    // Move layer 9 to index 1: old layers 1..=8 shift up by one.
    ops::move_layer(&mut doc, 9, 1).unwrap();
    assert_eq!(doc.layout[1], old_l9); // old L9 has no layer references
    assert_eq!(wire_at(&doc, 0, 0, 6), "TO(3)");
    assert_eq!(wire_at(&doc, 0, 1, 6), "TO(2)");
    assert_eq!(wire_at(&doc, 0, 3, 4), "MO(8)");
    assert_eq!(wire_at(&doc, 0, 4, 6), "TO(7)");
    assert_eq!(wire_at(&doc, 0, 5, 6), "TO(0)");
    assert_eq!(wire_at(&doc, 0, 7, 4), "MO(9)");
    assert_eq!(wire_at(&doc, 7, 0, 0), "TO(2)"); // old L6
    assert_eq!(wire_at(&doc, 8, 7, 4), "MO(1)"); // old L7
    assert_eq!(wire_at(&doc, 9, 3, 4), "MO(1)"); // old L8
}

#[test]
fn move_layer_back_restores_the_file() {
    let mut doc = common::load("corne_v32.vil");
    ops::move_layer(&mut doc, 9, 1).unwrap();
    ops::move_layer(&mut doc, 1, 9).unwrap();
    assert_eq!(doc.to_vil_bytes(), common::fixture_bytes("corne_v32.vil"));
}

#[test]
fn move_to_same_index_changes_nothing() {
    let original = common::load("corne_v32.vil");
    let mut doc = original.clone();
    ops::move_layer(&mut doc, 4, 4).unwrap();
    assert_eq!(doc, original);
    assert!(ops::move_layer(&mut doc, 4, 10).is_err());
}

#[test]
fn mirror_left_to_right_copies_mirrored_positions() {
    let board = Board::corne_v4_1();
    let mut doc = common::load("corne.vil");
    let before_left: Vec<_> = board
        .keys
        .iter()
        .filter(|k| k.id.contains("-left-"))
        .map(|k| doc.get(0, k.row, k.col))
        .collect();
    ops::mirror_layer(&mut doc, &board, 0, MirrorMode::LeftToRight).unwrap();
    for (i, key) in board
        .keys
        .iter()
        .filter(|k| k.id.contains("-left-"))
        .enumerate()
    {
        let m = board.mirror_of(key).unwrap();
        assert_eq!(
            doc.get(0, key.row, key.col),
            before_left[i],
            "left side changed at {}",
            key.id
        );
        assert_eq!(
            doc.get(0, m.row, m.col),
            doc.get(0, key.row, key.col),
            "{}",
            m.id
        );
    }
}

#[test]
fn mirror_swap_twice_is_identity_and_other_layers_are_untouched() {
    let board = Board::corne_v4_1();
    let original = common::load("corne_v32.vil");
    let mut doc = original.clone();
    ops::mirror_layer(&mut doc, &board, 7, MirrorMode::Swap).unwrap();
    assert_ne!(doc.layout[7], original.layout[7]);
    assert_eq!(doc.layout[6], original.layout[6]);
    assert_eq!(wire_at(&doc, 7, 3, 4), "MO(9)"); // v32 L7 had MO(9) on thumb-right-mid
    ops::mirror_layer(&mut doc, &board, 7, MirrorMode::Swap).unwrap();
    assert_eq!(doc, original);
}

#[test]
fn mirror_right_to_left() {
    let board = Board::corne_v4_1();
    let mut doc = common::load("corne_v32.vil");
    ops::mirror_layer(&mut doc, &board, 8, MirrorMode::RightToLeft).unwrap();
    assert_eq!(wire_at(&doc, 8, 0, 2), wire_at(&doc, 8, 4, 2));
    assert_eq!(wire_at(&doc, 8, 3, 4), wire_at(&doc, 8, 7, 4));
}
