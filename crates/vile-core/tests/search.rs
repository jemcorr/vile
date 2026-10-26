use vile_core::{Catalog, CatalogContext};

fn cat() -> Catalog {
    Catalog::new(CatalogContext {
        layers: 10,
        macros: 16,
        tap_dances: 32,
    })
}

fn first(q: &str) -> Option<String> {
    cat().search(q, 5).first().map(|h| h.name.clone())
}

#[test]
fn expected_hits_rank_first() {
    assert_eq!(first("esc").as_deref(), Some("KC_ESCAPE"));
    assert_eq!(first("ESC").as_deref(), Some("KC_ESCAPE"));
    assert_eq!(first("bspc").as_deref(), Some("KC_BSPACE"));
    assert_eq!(first("mo9").as_deref(), Some("MO(9)"));
    assert_eq!(first("lt8").as_deref(), Some("LT8(kc)"));
    assert_eq!(first("trns").as_deref(), Some("KC_TRNS"));
    assert_eq!(first("tild").as_deref(), Some("LSFT(KC_GRAVE)"));
    assert_eq!(first("rep").as_deref(), Some("QK_REPEAT_KEY"));
}

#[test]
fn templates_insert_an_open_parenthesis() {
    let hits = cat().search("lsft_t", 3);
    let hit = hits
        .iter()
        .find(|h| h.name == "LSFT_T(kc)")
        .expect("template hit");
    assert_eq!(hit.insert, "LSFT_T(");
    assert!(!hit.complete);
    let esc = &cat().search("esc", 1)[0];
    assert_eq!(esc.insert, "KC_ESCAPE");
    assert!(esc.complete);
}

#[test]
fn limits_and_empty_results() {
    assert!(cat().search("zzzzzz", 5).is_empty());
    assert_eq!(cat().search("", 7).len(), 7);
    assert!(cat().search("kc", 3).len() <= 3);
}

#[test]
fn generated_names_follow_the_context() {
    let small = Catalog::new(CatalogContext {
        layers: 4,
        macros: 2,
        tap_dances: 0,
    });
    assert!(small.search("mo9", 5).iter().all(|h| h.name != "MO(9)"));
    assert!(small.search("m1", 5).iter().any(|h| h.name == "M1"));
    assert!(small.search("m5", 5).iter().all(|h| h.name != "M5"));
}

#[test]
fn search_is_deterministic() {
    assert_eq!(cat().search("s", 20), cat().search("s", 20));
}

#[test]
fn exact_names_do_not_drag_in_letter_matches_from_prose() {
    let names = |q: &str| {
        cat()
            .search(q, 4096)
            .into_iter()
            .map(|h| h.name)
            .collect::<Vec<_>>()
    };
    let trns = names("kc_trns");
    assert_eq!(trns.first().map(String::as_str), Some("KC_TRNS"));
    assert!(
        trns.len() <= 3,
        "kc_trns matched {} keycodes: {trns:?}",
        trns.len()
    );
    let reboot = names("qk_reboot");
    assert_eq!(reboot.first().map(String::as_str), Some("QK_REBOOT"));
    assert!(!reboot.iter().any(|n| n == "KC_EQUAL"), "{reboot:?}");
    assert!(
        reboot.len() <= 3,
        "qk_reboot matched {} keycodes: {reboot:?}",
        reboot.len()
    );
}

#[test]
fn letter_matches_remain_a_fallback_for_names() {
    let hits = cat().search("bspce", 4096);
    assert!(hits.iter().any(|h| h.name == "KC_BSPACE"), "{hits:?}");
}
