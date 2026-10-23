use vile_core::{Catalog, CatalogContext, Keycode, KeycodeKind, LayerOp};

fn fixture_bytes(name: &str) -> Vec<u8> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// Every keycode string in a raw .vil JSON value, in file order.
fn keycode_strings(v: &serde_json::Value) -> Vec<String> {
    let mut out = Vec::new();
    let mut push = |x: &serde_json::Value| {
        if let Some(s) = x.as_str() {
            out.push(s.to_string());
        }
    };
    for layer in v["layout"].as_array().unwrap() {
        for row in layer.as_array().unwrap() {
            for cell in row.as_array().unwrap() {
                push(cell);
            }
        }
    }
    for layer in v["encoder_layout"].as_array().unwrap() {
        for enc in layer.as_array().unwrap() {
            for cell in enc.as_array().unwrap() {
                push(cell);
            }
        }
    }
    for m in v["macro"].as_array().unwrap() {
        for action in m.as_array().unwrap() {
            let a = action.as_array().unwrap();
            if matches!(a[0].as_str(), Some("tap" | "down" | "up")) {
                for k in &a[1..] {
                    push(k);
                }
            }
        }
    }
    for td in v["tap_dance"].as_array().unwrap() {
        for k in &td.as_array().unwrap()[0..4] {
            push(k);
        }
    }
    for c in v["combo"].as_array().unwrap() {
        for k in c.as_array().unwrap() {
            push(k);
        }
    }
    for ko in v["key_override"].as_array().unwrap() {
        push(&ko["trigger"]);
        push(&ko["replacement"]);
    }
    for ar in v["alt_repeat_key"].as_array().unwrap() {
        push(&ar["keycode"]);
        push(&ar["alt_keycode"]);
    }
    out
}

fn cat() -> Catalog {
    Catalog::new(CatalogContext {
        layers: 10,
        macros: 16,
        tap_dances: 32,
    })
}

#[test]
fn every_keycode_in_the_fixtures_parses_and_round_trips() {
    for name in ["corne.vil", "corne_v32.vil"] {
        let v: serde_json::Value = serde_json::from_slice(&fixture_bytes(name)).unwrap();
        let strings = keycode_strings(&v);
        assert!(
            strings.len() > 500,
            "{name}: only {} keycodes found",
            strings.len()
        );
        let cat = cat();
        for s in &strings {
            let kc = cat
                .parse(s)
                .unwrap_or_else(|e| panic!("{name}: {s} failed to parse: {e}"));
            assert_eq!(cat.wire_name(kc), *s, "{name}: wire name differs for {s}");
        }
    }
}

#[test]
fn unknown_keycodes_are_rejected_not_turned_into_kc_no() {
    let cat = cat();
    for bad in [
        "KC_FOO",
        "kc_a",
        "LSFT(KC_FOO)",
        "MO(32)",
        "MO(01)",
        "MO()",
        "LT16(KC_A)",
        "LT(16,KC_A)",
        "LSFT(MO(1))",
        "LCTL(RSFT(KC_A))",
        "MT(MOD_LSFT|MOD_RCTL,KC_A)",
        "OSM(MOD_FOO)",
        "0x10000",
        "0x",
        "0xZZ",
        "",
        " KC_A",
        "KC_A ",
        "M256",
        "M01",
        "USER64",
        "USER1",
        "TD(256)",
        "LT(1,KC_A",
        "FOO(KC_A)",
        "LSFT(KC_A))",
    ] {
        assert!(cat.parse(bad).is_err(), "{bad:?} should be rejected");
    }
}

#[test]
fn wire_names_follow_vial_rules() {
    let cat = cat();
    for (input, wire) in [
        ("KC_TILD", "LSFT(KC_GRAVE)"),
        ("KC_ESC", "KC_ESCAPE"),
        ("KC_TRANSPARENT", "KC_TRNS"),
        ("QK_REP", "QK_REPEAT_KEY"),
        ("LCTL(LSFT(KC_A))", "C_S(KC_A)"),
        ("MT(MOD_LSFT,KC_A)", "LSFT_T(KC_A)"),
        ("LT(8,KC_ESC)", "LT8(KC_ESCAPE)"),
        ("RSA(KC_A)", "0x1604"),
        ("LT12(KC_A)", "0x4c04"),
        ("MO(12)", "0x522c"),
        ("M20", "0x7714"),
        ("OSM(MOD_LSFT|MOD_LCTL)", "OSM(MOD_LCTL|MOD_LSFT)"),
        ("LM(1,MOD_LSFT)", "0x5022"),
        ("TD(40)", "TD(40)"),
        ("USER05", "USER05"),
        ("USER20", "0x7e14"),
        ("0x4", "KC_A"),
        ("0x7C79", "QK_REPEAT_KEY"),
        ("FN_MO13", "FN_MO13"),
    ] {
        let kc = cat.parse(input).unwrap_or_else(|e| panic!("{input}: {e}"));
        assert_eq!(cat.wire_name(kc), wire, "{input}");
    }
}

#[test]
fn wire_names_depend_on_layer_count() {
    let small = Catalog::new(CatalogContext {
        layers: 4,
        macros: 16,
        tap_dances: 32,
    });
    let kc = small.parse("MO(5)").unwrap();
    assert_eq!(small.wire_name(kc), "0x5225");
    assert_eq!(cat().wire_name(kc), "MO(5)");
    let tri = Catalog::new(CatalogContext {
        layers: 3,
        macros: 16,
        tap_dances: 32,
    });
    assert_eq!(tri.wire_name(tri.parse("FN_MO13").unwrap()), "0x7c77");
}

#[test]
fn kinds_and_layer_references() {
    let cat = cat();
    let k = |s: &str| cat.parse(s).unwrap();
    assert_eq!(k("KC_A").kind(), KeycodeKind::Basic { key: 0x04 });
    assert_eq!(
        k("LT8(KC_ESCAPE)").kind(),
        KeycodeKind::LayerTap {
            layer: 8,
            key: 0x29
        }
    );
    assert_eq!(
        k("MO(9)").kind(),
        KeycodeKind::Layer {
            op: LayerOp::Mo,
            layer: 9
        }
    );
    assert_eq!(
        k("TO(1)").kind(),
        KeycodeKind::Layer {
            op: LayerOp::To,
            layer: 1
        }
    );
    assert_eq!(
        k("PDF(2)").kind(),
        KeycodeKind::Layer {
            op: LayerOp::Pdf,
            layer: 2
        }
    );
    assert_eq!(
        k("LSFT_T(KC_N)").kind(),
        KeycodeKind::ModTap {
            mods: 0x02,
            key: 0x11
        }
    );
    assert_eq!(
        k("RGUI_T(KC_H)").kind(),
        KeycodeKind::ModTap {
            mods: 0x18,
            key: 0x0B
        }
    );
    assert_eq!(
        k("LSFT(KC_8)").kind(),
        KeycodeKind::Mods {
            mods: 0x02,
            key: 0x25
        }
    );
    assert_eq!(
        k("OSM(MOD_LSFT)").kind(),
        KeycodeKind::OneShotMod { mods: 0x02 }
    );
    assert_eq!(
        k("LM(1,MOD_LSFT)").kind(),
        KeycodeKind::LayerMod {
            layer: 1,
            mods: 0x02
        }
    );
    assert_eq!(k("TD(3)").kind(), KeycodeKind::TapDance { index: 3 });
    assert_eq!(k("M0").kind(), KeycodeKind::Macro { index: 0 });
    assert_eq!(k("USER02").kind(), KeycodeKind::User { index: 2 });
    assert_eq!(k("QK_REPEAT_KEY").kind(), KeycodeKind::Other);
    assert_eq!(k("KC_A").layer_ref(), None);
    assert_eq!(k("LT8(KC_ESCAPE)").layer_ref(), Some(8));
    assert_eq!(k("LM(1,MOD_LSFT)").layer_ref(), Some(1));
    assert_eq!(k("TT(4)").layer_ref(), Some(4));
    assert_eq!(k("FN_MO13").fixed_layer_refs(), &[1, 3]);
    assert_eq!(k("FN_MO23").fixed_layer_refs(), &[2, 3]);
    assert!(k("KC_A").fixed_layer_refs().is_empty());
}

#[test]
fn with_layer_rewrites_only_the_layer() {
    let cat = cat();
    let k = |s: &str| cat.parse(s).unwrap();
    let w = |kc: Keycode| cat.wire_name(kc);
    assert_eq!(
        k("LT8(KC_ESCAPE)").with_layer(7).map(w),
        Ok("LT7(KC_ESCAPE)".to_string())
    );
    assert_eq!(k("MO(9)").with_layer(2).map(w), Ok("MO(2)".to_string()));
    assert_eq!(k("MO(9)").with_layer(31).map(|x| x.code()), Ok(0x523F));
    assert_eq!(
        k("LM(1,MOD_LSFT)").with_layer(3).map(|x| x.code()),
        Ok(0x5062)
    );
    assert!(k("LT8(KC_ESCAPE)").with_layer(16).is_err());
    assert!(k("LM(1,MOD_LSFT)").with_layer(16).is_err());
    assert!(k("MO(9)").with_layer(32).is_err());
    assert!(k("KC_A").with_layer(1).is_err());
}
