mod common;
use vile_core::{HoldBehavior, KeyPartsPatch};
use vile_session::{OpDto, Session};

#[test]
fn patches_preserve_bases_and_form_one_undo_step() {
    let mut session = Session::new();
    session.open_path(&common::fixture("corne.vil")).unwrap();
    session
        .apply_op(OpDto::SetKeys {
            layer: 0,
            keys: vec!["r0c1".into()],
            keycode: "KC_S".into(),
        })
        .unwrap();
    let before = session
        .apply_op(OpDto::SetKeys {
            layer: 0,
            keys: vec!["r0c2".into()],
            keycode: "KC_D".into(),
        })
        .unwrap();
    let paths = vec!["layout[0][0][1]".into(), "layout[0][0][2]".into()];
    let view = session
        .apply_op_at(
            before.revision,
            OpDto::PatchFields {
                paths,
                patch: KeyPartsPatch {
                    hold: Some(HoldBehavior::Mod { mods: 8 }),
                    ..Default::default()
                },
            },
        )
        .unwrap();
    assert_eq!(view.keycodes["layout[0][0][1]"].wire, "LGUI_T(KC_S)");
    assert_eq!(view.keycodes["layout[0][0][2]"].wire, "LGUI_T(KC_D)");
    assert_eq!(session.undo().unwrap().layers, before.layers);
}
#[test]
fn invalid_batch_and_stale_revision_leave_the_document_unchanged() {
    let mut session = Session::new();
    session.open_path(&common::fixture("corne.vil")).unwrap();
    let before = session
        .apply_op(OpDto::SetKeys {
            layer: 0,
            keys: vec!["r0c2".into()],
            keycode: "M0".into(),
        })
        .unwrap();
    let err = session
        .apply_op_at(
            before.revision,
            OpDto::PatchFields {
                paths: vec!["layout[0][0][1]".into(), "layout[0][0][2]".into()],
                patch: KeyPartsPatch {
                    hold: Some(HoldBehavior::Layer { layer: 8 }),
                    ..Default::default()
                },
            },
        )
        .unwrap_err();
    assert!(err.contains("layout[0][0][2]"));
    assert_eq!(session.view().unwrap(), before);
    assert!(
        session
            .apply_op_at(
                before.revision - 1,
                OpDto::ClearLayer {
                    layer: 0,
                    transparent: false
                }
            )
            .is_err()
    );
    assert_eq!(session.view().unwrap(), before);
}
#[test]
fn fields_work_across_all_document_sections() {
    let mut session = Session::new();
    let initial = session.open_path(&common::fixture("corne.vil")).unwrap();
    let paths: Vec<_> = initial
        .keycodes
        .keys()
        .filter(|path| !path.starts_with("layout"))
        .cloned()
        .collect();
    assert!(paths.iter().any(|p| p.starts_with("combo")));
    assert!(paths.iter().any(|p| p.starts_with("macro")));
    let view = session
        .apply_op(OpDto::ReplaceFields {
            paths: paths.clone(),
            keycode: "KC_ESCAPE".into(),
        })
        .unwrap();
    for path in paths {
        assert_eq!(view.keycodes[&path].wire, "KC_ESCAPE");
    }
    assert_eq!(session.undo().unwrap().keycodes, initial.keycodes);
}
#[test]
fn names_follow_layer_moves_and_survive_save_reopen_and_undo() {
    let (_dir, path) = common::temp_copy("corne.vil");
    let mut session = Session::new();
    session.open_path(&path).unwrap();
    session
        .apply_op(OpDto::Rename {
            kind: "layer".into(),
            index: 7,
            name: "Symbols".into(),
        })
        .unwrap();
    let view = session.apply_op(OpDto::SwapLayers { a: 7, b: 8 }).unwrap();
    assert_eq!(view.names.layers.get(&8).unwrap(), "Symbols");
    let view = session.undo().unwrap();
    assert_eq!(view.names.layers.get(&7).unwrap(), "Symbols");
    session.save().unwrap();
    let mut other = Session::new();
    let view = other.open_path(&path).unwrap();
    assert_eq!(view.names.layers.get(&7).unwrap(), "Symbols");
    assert!(
        view.keycodes
            .values()
            .any(|key| key.description.contains("Symbols"))
    );
    session
        .apply_op(OpDto::Rename {
            kind: "macro".into(),
            index: 0,
            name: "Jupyter cell".into(),
        })
        .unwrap();
    assert!(!session.search_keycodes("Jupyter", 10).is_empty());
}
#[test]
fn wrong_uid_sidecar_is_ignored_and_external_changes_are_protected() {
    let (_dir, path) = common::temp_copy("corne.vil");
    let mut session = Session::new();
    session.open_path(&path).unwrap();
    session
        .apply_op(OpDto::Rename {
            kind: "layer".into(),
            index: 0,
            name: "Base".into(),
        })
        .unwrap();
    session.save().unwrap();
    let meta = vile_core::metadata::sidecar_path(&path);
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&meta).unwrap()).unwrap();
    value["uid"] = serde_json::json!("0");
    std::fs::write(&meta, serde_json::to_vec(&value).unwrap()).unwrap();
    let view = Session::new().open_path(&path).unwrap();
    assert!(view.names.layers.is_empty());
    assert!(view.warnings.iter().any(|w| w.contains("sidecar")));
    assert!(session.save().unwrap_err().contains("changed on disk"));
    std::fs::remove_file(&path).unwrap();
    assert!(session.save().is_err());
}
#[test]
fn git_comparison_does_not_change_pending_markers() {
    let (dir, path) = common::temp_copy("corne.vil");
    common::git(dir.path(), &["init", "-q"]);
    common::git(dir.path(), &["add", "."]);
    common::git(
        dir.path(),
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.test",
            "commit",
            "-qm",
            "baseline",
        ],
    );
    let mut session = Session::new();
    session.open_path(&path).unwrap();
    session
        .apply_op(OpDto::SetKeys {
            layer: 0,
            keys: vec!["r0c0".into()],
            keycode: "KC_ESCAPE".into(),
        })
        .unwrap();
    session.save().unwrap();
    let view = session.set_diff_base("git").unwrap();
    assert!(!view.diff.is_empty());
    assert!(view.pending_diff.is_empty());
    assert!(view.layers.iter().flatten().all(|cell| !cell.changed));
}

#[test]
fn save_as_replaces_destination_names_with_the_source_names() {
    let (dir, path) = common::temp_copy("corne.vil");
    let target = dir.path().join("named.vil");
    std::fs::copy(&path, &target).unwrap();
    let mut named = Session::new();
    named.open_path(&target).unwrap();
    named
        .apply_op(OpDto::Rename {
            kind: "layer".into(),
            index: 0,
            name: "Destination".into(),
        })
        .unwrap();
    named.save().unwrap();
    let mut source = Session::new();
    source.open_path(&path).unwrap();
    source.save_as(&target).unwrap();
    let view = Session::new().open_path(&target).unwrap();
    assert!(view.names.layers.is_empty());
    assert!(view.warnings.is_empty());
}
