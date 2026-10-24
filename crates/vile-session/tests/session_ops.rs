mod common;

use vile_session::{KeyRefDto, OpDto, Session};

fn open(name: &str) -> (tempfile::TempDir, Session) {
    let (dir, path) = common::temp_copy(name);
    let mut s = Session::new();
    s.open_path(&path).unwrap();
    (dir, s)
}

fn keys(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

#[test]
fn set_keys_marks_dirty_and_shows_the_diff() {
    let (_dir, mut s) = open("corne_v32.vil");
    let v = s
        .apply_op(OpDto::SetKeys {
            layer: 0,
            keys: keys(&["r3c4"]),
            keycode: "lt8(esc)".to_string(),
        })
        .unwrap();
    assert!(v.dirty);
    assert!(v.can_undo);
    assert_eq!(v.diff.len(), 1);
    assert_eq!(v.diff[0].line, "L0 r3c4: MO(7) -> LT8(KC_ESCAPE)");
    assert_eq!(v.diff[0].section, "layout");
    assert_eq!(v.diff[0].layer, Some(0));
    assert_eq!(v.diff[0].key_id.as_deref(), Some("r3c4"));
    let cell = &v.layers[0][common::key_index(&v, "r3c4")];
    assert_eq!(cell.wire, "LT8(KC_ESCAPE)");
    assert_eq!(cell.label, "Esc\nL8");
    assert!(cell.changed);
    let v = s.undo().unwrap();
    assert!(!v.dirty);
    assert!(v.can_redo);
    let v = s.redo().unwrap();
    assert!(v.dirty);
}

#[test]
fn unknown_keycodes_are_rejected_and_nothing_changes() {
    let (_dir, mut s) = open("corne.vil");
    let before = s.view().unwrap();
    let err = s
        .apply_op(OpDto::SetKeys {
            layer: 0,
            keys: keys(&["r0c0"]),
            keycode: "KC_FOO".to_string(),
        })
        .unwrap_err();
    assert!(err.contains("unknown keycode"), "{err}");
    assert_eq!(s.view().unwrap(), before);
    let err = s
        .apply_op(OpDto::SetKeys {
            layer: 0,
            keys: keys(&["no-such-key"]),
            keycode: "KC_A".to_string(),
        })
        .unwrap_err();
    assert!(err.contains("no-such-key"), "{err}");
}

#[test]
fn clear_transparent_copy_paste_and_swap_keys() {
    let (_dir, mut s) = open("corne_v32.vil");
    let sel = keys(&["r1c1", "r1c2"]);
    let v = s
        .apply_op(OpDto::ClearKeys {
            layer: 0,
            keys: sel.clone(),
        })
        .unwrap();
    assert_eq!(common::wire(&v, 0, "r1c1"), "KC_NO");
    let v = s
        .apply_op(OpDto::TransparentKeys {
            layer: 0,
            keys: sel.clone(),
        })
        .unwrap();
    assert_eq!(common::wire(&v, 0, "r1c2"), "KC_TRNS");
    s.undo().unwrap();
    s.undo().unwrap();
    s.apply_op(OpDto::CopyKeys {
        layer: 0,
        keys: sel.clone(),
    })
    .unwrap();
    let v = s
        .apply_op(OpDto::PasteKeys {
            layer: 4,
            keys: vec![],
        })
        .unwrap();
    assert_eq!(common::wire(&v, 4, "r1c1"), common::wire(&v, 0, "r1c1"));
    let v = s
        .apply_op(OpDto::SwapKeys {
            a: vec![KeyRefDto {
                layer: 0,
                key: "r3c4".to_string(),
            }],
            b: vec![KeyRefDto {
                layer: 9,
                key: "r0c1".to_string(),
            }],
        })
        .unwrap();
    assert_eq!(common::wire(&v, 0, "r3c4"), "KC_F1");
    assert_eq!(common::wire(&v, 9, "r0c1"), "MO(7)");
}

#[test]
fn layer_operations() {
    let (_dir, mut s) = open("corne_v32.vil");
    let v = s.apply_op(OpDto::SwapLayers { a: 7, b: 8 }).unwrap();
    assert_eq!(common::wire(&v, 0, "r3c4"), "MO(8)");
    let v = s.apply_op(OpDto::SwapLayers { a: 7, b: 8 }).unwrap();
    assert!(!v.dirty);
    let v = s.apply_op(OpDto::MoveLayer { from: 9, to: 1 }).unwrap();
    assert_eq!(common::wire(&v, 0, "r7c4"), "MO(9)");
    let v = s
        .apply_op(OpDto::ClearLayer {
            layer: 2,
            transparent: true,
        })
        .unwrap();
    assert!(v.layers[2].iter().all(|c| c.wire == "KC_TRNS"));
    let v = s
        .apply_op(OpDto::ClearLayer {
            layer: 2,
            transparent: false,
        })
        .unwrap();
    assert!(v.layers[2].iter().all(|c| c.wire == "KC_NO"));
    s.apply_op(OpDto::CopyLayer { layer: 0 }).unwrap();
    let v = s.apply_op(OpDto::PasteLayer { layer: 5 }).unwrap();
    assert_eq!(
        v.layers[5].iter().map(|c| &c.wire).collect::<Vec<_>>(),
        v.layers[0].iter().map(|c| &c.wire).collect::<Vec<_>>()
    );
    let error = s
        .apply_op(OpDto::MirrorLayer {
            layer: 5,
            mode: "rightToLeft".into(),
        })
        .unwrap_err();
    assert!(error.contains("mirror pairs"));
    assert!(
        s.apply_op(OpDto::MirrorLayer {
            layer: 5,
            mode: "sideways".to_string()
        })
        .is_err()
    );
    assert!(s.apply_op(OpDto::SwapLayers { a: 0, b: 10 }).is_err());
}

#[test]
fn search_and_resolve() {
    let (_dir, s) = open("corne.vil");
    let hits = s.search_keycodes("esc", 3);
    assert_eq!(hits[0].name, "KC_ESCAPE");
    assert!(hits[0].complete);
    assert_eq!(s.resolve_keycode("lsft(a").unwrap(), "LSFT(KC_A)");
    assert!(s.resolve_keycode("KC_FOO").is_err());
}

#[test]
fn picker_tooltips_keep_full_behavior_without_expanding_inspector_names() {
    let (_dir, session) = open("corne.vil");
    let boot = session.search_keycodes("QK_BOOT", 1).remove(0);
    assert_eq!(boot.picker_label, "Bootloader");
    assert_eq!(boot.summary, "Flash firmware");
    assert_eq!(boot.picker_group, "Firmware and reset");
    assert!(boot.description.contains("flash firmware"));
    assert_eq!(
        session.keycode_view("QK_BOOT").unwrap().description,
        "Enter bootloader"
    );
    let reset = session.search_keycodes("QK_CLEAR_EEPROM", 1).remove(0);
    assert!(reset.description.contains("persistent settings"));
    assert!(reset.description.contains("depending on the firmware"));
}

#[test]
fn detailed_behavior_reaches_board_fields_and_search_without_losing_user_names() {
    let (_dir, mut session) = open("corne.vil");
    session
        .apply_op(OpDto::Rename {
            kind: "layer".into(),
            index: 2,
            name: "Nav".into(),
        })
        .unwrap();
    session
        .apply_op(OpDto::Rename {
            kind: "macro".into(),
            index: 0,
            name: "Launch".into(),
        })
        .unwrap();
    for wire in [
        "KC_ACL0",
        "KC_BTN1",
        "QK_CLEAR_EEPROM",
        "MO(2)",
        "PDF(2)",
        "M0",
        "TD(0)",
    ] {
        let view = session
            .apply_op(OpDto::SetKeys {
                layer: 0,
                keys: keys(&["r3c4"]),
                keycode: wire.into(),
            })
            .unwrap();
        let cell = &view.layers[0][common::key_index(&view, "r3c4")];
        let field = session.keycode_view(wire).unwrap();
        let hit = session
            .search_keycodes(wire, 4096)
            .into_iter()
            .find(|hit| hit.name == wire)
            .unwrap();
        assert_eq!(cell.tooltip, field.tooltip, "board vs field: {wire}");
        assert_eq!(hit.description, field.tooltip, "picker vs field: {wire}");
        assert_eq!(cell.label, field.label);
        assert_eq!(cell.wire, wire);
        assert_ne!(
            field.description, field.tooltip,
            "heading must remain compact: {wire}"
        );
        if wire == "MO(2)" {
            assert!(field.tooltip.contains("layer 2 (Nav)"));
        }
        if wire == "M0" {
            assert!(field.tooltip.contains("slot 0 (Launch)"));
        }
        if wire == "KC_ACL0" {
            assert_eq!(cell.label, "Speed\n0");
        }
    }
    let templates = session
        .search_keycodes("", 4096)
        .into_iter()
        .filter(|hit| !hit.complete)
        .collect::<Vec<_>>();
    assert!(!templates.is_empty());
    assert!(
        templates
            .iter()
            .all(|hit| hit.description.contains("Choose a basic"))
    );
}

#[test]
fn diff_base_switches_between_disk_and_git() {
    let (dir, path) = common::temp_copy("corne_v32.vil");
    common::git(dir.path(), &["init", "-q"]);
    common::git(dir.path(), &["add", "."]);
    common::git(
        dir.path(),
        &[
            "-c",
            "user.name=vile",
            "-c",
            "user.email=vile@example.invalid",
            "commit",
            "-q",
            "-m",
            "x",
        ],
    );
    let mut s = Session::new();
    s.open_path(&path).unwrap();
    s.apply_op(OpDto::SetKeys {
        layer: 0,
        keys: keys(&["r3c4"]),
        keycode: "KC_A".to_string(),
    })
    .unwrap();
    s.save().unwrap();
    let v = s.set_diff_base("git").unwrap();
    assert_eq!(v.diff_base, "git");
    assert_eq!(v.diff.len(), 1);
    let v = s.set_diff_base("disk").unwrap();
    assert!(v.diff.is_empty());
    assert!(s.set_diff_base("bogus").is_err());
}
