mod common;

use vile_core::{Document, Slot};

const FIXTURES: [&str; 2] = ["corne.vil", "corne_v32.vil"];

#[test]
fn round_trip_is_byte_identical() {
    for name in FIXTURES {
        let raw = common::fixture_bytes(name);
        let (doc, report) = Document::from_vil_bytes(&raw).expect("fixture loads");
        assert!(
            report.canonical,
            "{name}: loader reports a non-canonical file"
        );
        assert!(
            report.warnings.is_empty(),
            "{name}: unexpected warnings {:?}",
            report.warnings
        );
        assert_eq!(
            doc.to_vil_bytes(),
            raw,
            "{name}: bytes differ after a round trip"
        );
    }
}

#[test]
fn uid_is_preserved_exactly() {
    for name in FIXTURES {
        let doc = common::load(name);
        assert_eq!(doc.uid, 5010774632021243529u64, "{name}");
        let out = String::from_utf8(doc.to_vil_bytes()).unwrap();
        assert!(
            out.starts_with("{\"version\": 1, \"uid\": 5010774632021243529, \"layout\": [[["),
            "{name}"
        );
    }
}

#[test]
fn top_level_key_order_is_preserved() {
    let doc = common::load("corne.vil");
    let names: Vec<&str> = doc.key_order.iter().map(|k| k.as_str()).collect();
    assert_eq!(
        names,
        [
            "version",
            "uid",
            "layout",
            "encoder_layout",
            "layout_options",
            "macro",
            "vial_protocol",
            "via_protocol",
            "tap_dance",
            "combo",
            "key_override",
            "alt_repeat_key",
            "settings"
        ]
    );
}

#[test]
fn fixture_shape() {
    let doc = common::load("corne.vil");
    assert_eq!(doc.layer_count(), 10);
    assert_eq!(doc.rows(), 8);
    assert_eq!(doc.cols(), 7);
    assert_eq!(doc.macros.len(), 16);
    assert_eq!(doc.tap_dance.as_ref().map(Vec::len), Some(32));
    assert_eq!(doc.combo.as_ref().map(Vec::len), Some(32));
    assert_eq!(doc.key_override.as_ref().map(Vec::len), Some(32));
    assert_eq!(doc.alt_repeat_key.as_ref().map(Vec::len), Some(32));
    assert_eq!(doc.encoder_layout.as_ref().map(Vec::len), Some(10));
    for layer in 0..10 {
        for (r, c) in [
            (2, 6),
            (3, 0),
            (3, 1),
            (3, 2),
            (3, 6),
            (6, 6),
            (7, 0),
            (7, 1),
            (7, 2),
            (7, 6),
        ] {
            assert_eq!(
                doc.get(layer, r, c),
                Some(Slot::Absent),
                "L{layer} r{r}c{c}"
            );
        }
        let present = (0..8)
            .flat_map(|r| (0..7).map(move |c| (r, c)))
            .filter(|&(r, c)| doc.get(layer, r, c) != Some(Slot::Absent))
            .count();
        assert_eq!(present, 46, "L{layer}");
    }
}

#[test]
fn macro_slots_are_preserved() {
    for name in FIXTURES {
        let raw: serde_json::Value = serde_json::from_slice(&common::fixture_bytes(name)).unwrap();
        let doc = common::load(name);
        assert_eq!(doc.macros.len(), 16, "{name}");
        let out: serde_json::Value = serde_json::from_slice(&doc.to_vil_bytes()).unwrap();
        assert_eq!(out["macro"], raw["macro"], "{name}");
        assert_eq!(out["macro"].as_array().map(Vec::len), Some(16), "{name}");
    }
}

#[test]
fn macro_contents_are_typed() {
    use vile_core::MacroAction;
    let doc = common::load("corne_v32.vil");
    let k = |s: &str| common::kc(&doc, s);
    assert_eq!(
        doc.macros[0].actions,
        vec![MacroAction::Tap(vec![k("KC_QUOTE"), k("KC_SPACE")])]
    );
    assert_eq!(
        doc.macros[1].actions,
        vec![MacroAction::Tap(vec![k("LSFT(KC_QUOTE)"), k("KC_SPACE")])]
    );
    assert_eq!(
        doc.macros[2].actions,
        vec![
            MacroAction::Text("# %%".to_string()),
            MacroAction::Tap(vec![k("KC_ENTER")])
        ]
    );
    assert_eq!(
        doc.macros[3].actions,
        vec![MacroAction::Text("//".to_string())]
    );
    assert!(doc.macros[4..].iter().all(|m| m.actions.is_empty()));
}
