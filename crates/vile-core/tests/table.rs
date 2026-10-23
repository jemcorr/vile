use vile_core::keycode::{Group, table};

#[test]
fn table_has_expected_rows() {
    let t = table();
    assert_eq!(t.len(), 403);
    assert_eq!(t[0].name, "KC_NO");
    assert_eq!(t[0].code, 0x0000);
    assert_eq!(t[0].group, Group::Special);
    assert_eq!(t[1].name, "KC_TRNS");
    assert_eq!(t[1].aliases, vec!["KC_TRANSPARENT"]);
}

#[test]
fn table_contains_names_used_by_the_fixture() {
    let t = table();
    for (name, code) in [
        ("KC_PSCREEN", 0x0046),
        ("KC_ACL0", 0x00DD),
        ("KC_ACL1", 0x00DE),
        ("KC_ACL2", 0x00DF),
        ("KC_GESC", 0x7C16),
        ("KC_BSPACE", 0x002A),
        ("KC_LSHIFT", 0x00E1),
        ("QK_REPEAT_KEY", 0x7C79),
        ("QK_ALT_REPEAT_KEY", 0x7C7A),
        ("KC_BRIU", 0x00BD),
        ("KC_MPLY", 0x00AE),
    ] {
        let e = t
            .iter()
            .find(|e| e.name == name)
            .unwrap_or_else(|| panic!("{name} missing"));
        assert_eq!(e.code, code, "{name}");
    }
}

#[test]
fn function_groups_are_marked() {
    let t = table();
    let lsft = t.iter().find(|e| e.name == "LSFT").unwrap();
    assert_eq!(lsft.group, Group::Mods);
    assert!(lsft.group.is_function());
    let lsft_t = t.iter().find(|e| e.name == "LSFT_T").unwrap();
    assert_eq!(lsft_t.group, Group::ModTap);
    assert_eq!(lsft_t.code, 0x2200);
    let rsa = t.iter().find(|e| e.name == "RSA").unwrap();
    assert_eq!(rsa.group, Group::ModsHex);
    assert!(
        !t.iter()
            .find(|e| e.name == "KC_A")
            .unwrap()
            .group
            .is_function()
    );
}

#[test]
fn names_and_aliases_are_unique_within_namespace() {
    let mut plain = std::collections::HashMap::new();
    let mut func = std::collections::HashMap::new();
    for e in table() {
        let map = if e.group.is_function() {
            &mut func
        } else {
            &mut plain
        };
        for n in std::iter::once(e.name).chain(e.aliases.iter().copied()) {
            if let Some(prev) = map.insert(n, e.code) {
                assert_eq!(prev, e.code, "{n} maps to two codes");
            }
        }
    }
}
