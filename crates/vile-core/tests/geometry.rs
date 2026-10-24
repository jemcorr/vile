mod common;
use serde_json::json;
use vile_core::geometry::from_vial_definition;
use vile_core::{Board, Slot};
fn document() -> vile_core::Document {
    let mut doc = common::load("corne.vil");
    doc.layout = vec![vec![vec![Slot::Key(vile_core::Keycode(4)); 2]]];
    doc.encoder_layout = Some(vec![vec![[Slot::Key(vile_core::Keycode(4)); 2]]]);
    doc.layout_options = 0;
    doc
}
#[test]
fn matrix_keeps_holes_and_rejects_implicit_mirroring() {
    let doc = common::load("corne.vil");
    let board = Board::matrix(&doc);
    assert_eq!(board.keys.len(), 46);
    assert!(board.key_by_id("r3c0").is_none());
    assert!(!board.can_mirror());
}
#[test]
fn kle_rotations_secondary_rectangles_and_nonkeys() {
    let doc = document();
    let definition = json!({"matrix":{"rows":1,"cols":2},"layouts":{"keymap":[[{"r":15,"rx":2,"ry":3,"w":1.5,"w2":1.0,"x2":0.5,"h2":2},"0,0",{"d":true},"decal","0,1"]]}});
    let board = from_vial_definition(&definition, &doc).unwrap();
    assert_eq!(board.keys.len(), 2);
    assert_eq!(board.keys[0].r, 15.0);
    assert_eq!(board.keys[0].rx, 2.0);
    assert_eq!(board.keys[0].secondary.as_ref().unwrap().h, 2.0);
    assert_eq!(board.keys[1].w, 1.0);
}
#[test]
fn layout_choices_use_reverse_bit_packing_and_align_alternatives() {
    let mut doc = document();
    doc.layout_options = 1;
    let def = json!({"matrix":{"rows":1,"cols":2},"layouts":{"labels":["Split"],"keymap":[["0,0\n\n\n0,0","0,1"],[{"x":5},"0,0\n\n\n0,1"]]}});
    let board = from_vial_definition(&def, &doc).unwrap();
    let key = board.key_by_id("r0c0").unwrap();
    assert_eq!(key.x, 0.0);
    assert_eq!(key.y, 0.0);
    assert_eq!(board.keys.len(), 2);
}

#[test]
fn encoder_visibility_follows_the_selected_layout_option() {
    let mut doc = document();
    let definition = json!({
        "matrix":{"rows":1,"cols":2},
        "layouts":{
            "labels":[["EX1", "Key", "Encoder", "None"]],
            "keymap":[["0,0", "0,0\n\n\n0,1\n\n\n\n\n\ne"]]
        }
    });
    for (option, expected) in [(0, vec![]), (1, vec![0]), (2, vec![])] {
        doc.layout_options = option;
        assert_eq!(
            from_vial_definition(&definition, &doc)
                .unwrap()
                .encoder_indices,
            Some(expected)
        );
    }
}

#[test]
fn unconditional_encoders_are_visible_and_invalid_legends_fall_back() {
    let doc = document();
    let definition = json!({
        "matrix":{"rows":1,"cols":2},
        "layouts":{"keymap":[["0,0", "0,0\n\n\n\n\n\n\n\n\ne"]]}
    });
    assert_eq!(
        from_vial_definition(&definition, &doc)
            .unwrap()
            .encoder_indices,
        Some(vec![0])
    );
    let invalid = json!({
        "matrix":{"rows":1,"cols":2},
        "layouts":{"keymap":[["0,0", "0,2\n\n\n\n\n\n\n\n\ne"]]}
    });
    assert!(from_vial_definition(&invalid, &doc).is_err());
}
#[test]
fn explicit_pairing_and_semantic_names_are_validated() {
    let doc = document();
    let mut def = json!({"matrix":{"rows":1,"cols":2},"layouts":{"keymap":[["0,0","0,1"]]},"vile":{"key_names":{"r0c0":"left thumb"},"mirror_pairs":[["r0c0","r0c1"]]}});
    let board = from_vial_definition(&def, &doc).unwrap();
    assert!(board.can_mirror());
    assert_eq!(board.key_by_id("left thumb").unwrap().id, "r0c0");
    def["vile"]["mirror_pairs"] = json!([["r0c0", "r9c9"]]);
    assert!(from_vial_definition(&def, &doc).is_err());
}
#[test]
fn invalid_definitions_return_errors_for_a_visible_matrix_fallback() {
    let doc = document();
    for def in [
        json!({}),
        json!({"matrix":{"rows":1,"cols":2},"layouts":{"keymap":[["9,9"]]}}),
        json!({"matrix":{"rows":1,"cols":2},"layouts":{"keymap":[[{"w":-1},"0,0"]]}}),
    ] {
        assert!(from_vial_definition(&def, &doc).is_err());
    }
}
