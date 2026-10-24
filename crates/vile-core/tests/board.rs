mod common;

use vile_core::Board;

#[test]
fn corne_board_has_46_unique_keys() {
    let b = common::board();
    assert_eq!(b.rows, 8);
    assert_eq!(b.cols, 7);
    assert_eq!(b.keys.len(), 46);
    let mut ids: Vec<&str> = b.keys.iter().map(|k| k.id.as_str()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 46);
}

#[test]
fn key_lookup_and_names() {
    let b = common::board();
    let k = b.key_by_id("thumb-left-mid").unwrap();
    assert_eq!((k.row, k.col), (3, 4));
    assert_eq!(b.key_at(7, 4).unwrap().id, "thumb-right-mid");
    assert_eq!(b.position_name(0, 0), "top-left-outer");
    assert_eq!(b.position_name(3, 0), "r3c0");
    assert_eq!(b.index_of(0, 0), Some(0));
    assert_eq!(b.index_of(3, 0), None);
}

#[test]
fn every_key_has_a_mirror_on_the_other_half() {
    let b = common::board();
    for k in &b.keys {
        let m = b
            .mirror_of(k)
            .unwrap_or_else(|| panic!("{} has no mirror", k.id));
        assert_eq!(m.col, k.col, "{}", k.id);
        assert_eq!(m.row, (k.row + 4) % 8, "{}", k.id);
        assert_eq!(b.mirror_of(m).unwrap().id, k.id);
    }
}

#[test]
fn board_matches_both_fixtures() {
    for name in ["corne.vil", "corne_v32.vil"] {
        let doc = common::load(name);
        assert!(common::board().matches(&doc), "{name}");
        assert_eq!(
            Board::for_document(&doc).map(|b| b.id),
            Some("matrix".to_string())
        );
    }
}
