use vile_core::{Catalog, CatalogContext};

fn cat() -> Catalog {
    Catalog::new(CatalogContext {
        layers: 10,
        macros: 16,
        tap_dances: 32,
    })
}

#[test]
fn lenient_input_accepts_user_spellings() {
    let cat = cat();
    for (input, wire) in [
        ("esc", "KC_ESCAPE"),
        ("a", "KC_A"),
        ("KC_A", "KC_A"),
        (";", "KC_SCOLON"),
        ("!", "LSFT(KC_1)"),
        ("lt8(esc)", "LT8(KC_ESCAPE)"),
        ("lsft(a", "LSFT(KC_A)"),
        ("  mo(9) ", "MO(9)"),
        ("lt(8, kc_esc)", "LT8(KC_ESCAPE)"),
        ("trns", "KC_TRNS"),
        ("osm(mod_lsft | mod_lctl)", "OSM(MOD_LCTL|MOD_LSFT)"),
        ("m3", "M3"),
    ] {
        let kc = cat
            .parse_input(input)
            .unwrap_or_else(|e| panic!("{input:?}: {e}"));
        assert_eq!(cat.wire_name(kc), wire, "{input:?}");
    }
}

#[test]
fn lenient_input_still_rejects_unknown_names() {
    let cat = cat();
    for bad in ["kc_foo", "lsft(mo(1))", "", "   ", "lt8(esc))", "mo(32)"] {
        assert!(cat.parse_input(bad).is_err(), "{bad:?}");
    }
}

#[test]
fn labels_are_short() {
    let cat = cat();
    let l = |s: &str| cat.label(cat.parse(s).unwrap());
    assert_eq!(l("KC_NO"), "");
    assert_eq!(l("KC_TRNS"), "▽");
    assert_eq!(l("KC_A"), "A");
    assert_eq!(l("KC_1"), "1");
    assert_eq!(l("KC_SCOLON"), ";");
    assert_eq!(l("KC_BSPACE"), "Bspc");
    assert_eq!(l("LSFT(KC_8)"), "*");
    assert_eq!(l("LSFT(KC_A)"), "⇧A");
    assert_eq!(l("C_S(KC_A)"), "⌃⇧A");
    assert_eq!(l("LSFT_T(KC_N)"), "N\n⇧");
    assert_eq!(l("RGUI_T(KC_H)"), "H\nR⌘");
    assert_eq!(l("LT8(KC_ESCAPE)"), "Esc\nL8");
    assert_eq!(l("MO(9)"), "MO9");
    assert_eq!(l("TO(1)"), "TO1");
    assert_eq!(l("OSM(MOD_LSFT)"), "OSM\n⇧");
    assert_eq!(l("TD(3)"), "TD3");
    assert_eq!(l("M0"), "M0");
    assert_eq!(l("USER02"), "U2");
    assert_eq!(l("QK_REPEAT_KEY"), "Rep");
    assert_eq!(l("KC_LSHIFT"), "⇧");
    assert_eq!(l("KC_RGUI"), "R⌘");
    assert_eq!(l("KC_PSCREEN"), "PrtSc");
}
