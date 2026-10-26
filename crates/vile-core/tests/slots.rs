mod common;

use std::collections::BTreeMap;

use vile_core::metadata::Names;
use vile_core::ops::OpError;
use vile_core::slots::{self, FeatureSlotKind, FeatureSlotValue, Place, SlotField};
use vile_core::validate;
use vile_core::{AltRepeatKey, Document, Keycode, Macro, MacroAction};

/// Small xorshift generator, so random sequences are reproducible without a dependency.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

/// The value of the keycode at one path of `Document::keycodes`.
fn value_at(doc: &Document, path: &str) -> Keycode {
    doc.keycodes()
        .into_iter()
        .find(|(p, _)| p == path)
        .unwrap_or_else(|| panic!("no keycode at {path}"))
        .1
}

/// The path a keycode moves to when the contents of slots `a` and `b` of `section` are exchanged.
fn follow(path: &str, section: &str, a: usize, b: usize) -> String {
    if let Some(rest) = path.strip_prefix(&format!("{section}[{a}]")) {
        format!("{section}[{b}]{rest}")
    } else if let Some(rest) = path.strip_prefix(&format!("{section}[{b}]")) {
        format!("{section}[{a}]{rest}")
    } else {
        path.to_string()
    }
}

fn in_slot(path: &str, section: &str, a: usize, b: usize) -> bool {
    path.starts_with(&format!("{section}[{a}]")) || path.starts_with(&format!("{section}[{b}]"))
}

fn all_paths(doc: &Document) -> Vec<String> {
    doc.keycodes().into_iter().map(|(p, _)| p).collect()
}

/// What a keycode means, with references resolved. Depth 3 stops the recursion.
#[derive(Debug, PartialEq)]
enum Meaning {
    Code(u16),
    Macro(Vec<ActionMeaning>),
    TapDance(Vec<Meaning>, u16),
    Deep,
}

#[derive(Debug, PartialEq)]
enum ActionMeaning {
    Tap(Vec<Meaning>),
    Down(Vec<Meaning>),
    Up(Vec<Meaning>),
    Text(String),
    Delay(u32),
}

fn meaning(doc: &Document, kc: Keycode, depth: usize) -> Meaning {
    if depth == 3 {
        return Meaning::Deep;
    }
    match slots::referenced_slot(kc) {
        Some((FeatureSlotKind::Macro, index)) if index < doc.macros.len() => Meaning::Macro(
            doc.macros[index]
                .actions
                .iter()
                .map(|action| action_meaning(doc, action, depth + 1))
                .collect(),
        ),
        Some((FeatureSlotKind::TapDance, index)) => {
            match doc.tap_dance.as_ref().and_then(|list| list.get(index)) {
                Some(td) => Meaning::TapDance(
                    [td.on_tap, td.on_hold, td.on_double_tap, td.on_tap_hold]
                        .into_iter()
                        .map(|k| meaning(doc, k, depth + 1))
                        .collect(),
                    td.tapping_term,
                ),
                None => Meaning::Code(kc.code()),
            }
        }
        _ => Meaning::Code(kc.code()),
    }
}

fn action_meaning(doc: &Document, action: &MacroAction, depth: usize) -> ActionMeaning {
    let keys = |keys: &[Keycode]| {
        keys.iter()
            .map(|&k| meaning(doc, k, depth))
            .collect::<Vec<_>>()
    };
    match action {
        MacroAction::Tap(k) => ActionMeaning::Tap(keys(k)),
        MacroAction::Down(k) => ActionMeaning::Down(keys(k)),
        MacroAction::Up(k) => ActionMeaning::Up(keys(k)),
        MacroAction::Text(text) => ActionMeaning::Text(text.clone()),
        MacroAction::Delay(ms) => ActionMeaning::Delay(*ms),
    }
}

/// Writes `kc` at one existing keycode path.
fn plant(doc: &mut Document, path: &str, kc: Keycode) {
    let map = BTreeMap::from([(path.to_string(), kc)]);
    doc.replace_keycodes(&map)
        .unwrap_or_else(|e| panic!("cannot plant at {path}: {e:?}"));
}

#[test]
fn is_set_matches_the_slot_editors() {
    let doc = common::load("corne.vil");
    let get = |kind, index| slots::get(&doc, kind, index).unwrap();
    assert!(slots::is_set(&get(FeatureSlotKind::Combo, 0)));
    assert!(!slots::is_set(&get(FeatureSlotKind::Combo, 10)));
    assert!(slots::is_set(&get(FeatureSlotKind::Macro, 0)));
    assert!(!slots::is_set(&get(FeatureSlotKind::Macro, 1)));
    assert!(!slots::is_set(&get(FeatureSlotKind::TapDance, 0)));
    assert!(!slots::is_set(&get(FeatureSlotKind::KeyOverride, 0)));
    assert!(!slots::is_set(&get(FeatureSlotKind::AltRepeat, 0)));

    // An alt repeat key counts as set when its options alone are non-zero.
    let options_only = FeatureSlotValue::AltRepeat(AltRepeatKey {
        keycode: Keycode::NO,
        alt_keycode: Keycode::NO,
        allowed_mods: 0,
        options: 8,
    });
    assert!(slots::is_set(&options_only));
}

#[test]
fn get_reports_the_slot_count() {
    let doc = common::load("corne.vil");
    assert_eq!(
        slots::get(&doc, FeatureSlotKind::Combo, 999),
        Err(OpError::NoSuchSlot {
            section: "combo",
            index: 999,
            count: 32,
        })
    );
}

#[test]
fn every_keycode_path_has_a_place() {
    let doc = common::load("corne.vil");
    for (path, _) in doc.keycodes() {
        assert!(slots::place_of(&path).is_some(), "no place for {path}");
    }
}

#[test]
fn fixture_uses() {
    let doc = common::load("corne.vil");
    let found: Vec<_> = slots::uses(&doc)
        .into_iter()
        .map(|u| (u.kind, u.index, u.path, u.place))
        .collect();
    let key = |layer, row, col| Place::Key { layer, row, col };
    assert_eq!(
        found,
        vec![
            (
                FeatureSlotKind::Macro,
                0,
                "layout[0][6][1]".to_string(),
                key(0, 6, 1)
            ),
            (
                FeatureSlotKind::Macro,
                2,
                "layout[7][4][0]".to_string(),
                key(7, 4, 0)
            ),
            (
                FeatureSlotKind::Macro,
                3,
                "layout[7][5][0]".to_string(),
                key(7, 5, 0)
            ),
        ]
    );
}

#[test]
fn every_field_form_is_found() {
    let forms: [(&str, Place); 10] = [
        (
            "layout[3][0][0]",
            Place::Key {
                layer: 3,
                row: 0,
                col: 0,
            },
        ),
        (
            "encoder_layout[2][1][1]",
            Place::Knob {
                layer: 2,
                knob: 1,
                clockwise: false,
            },
        ),
        (
            "macro[0][0][1]",
            Place::Slot {
                kind: FeatureSlotKind::Macro,
                index: 0,
                field: SlotField::MacroKey { action: 0, key: 0 },
            },
        ),
        (
            "tap_dance[4][2]",
            Place::Slot {
                kind: FeatureSlotKind::TapDance,
                index: 4,
                field: SlotField::TapDance(2),
            },
        ),
        (
            "combo[5][1]",
            Place::Slot {
                kind: FeatureSlotKind::Combo,
                index: 5,
                field: SlotField::ComboKey(1),
            },
        ),
        (
            "combo[5][4]",
            Place::Slot {
                kind: FeatureSlotKind::Combo,
                index: 5,
                field: SlotField::ComboOutput,
            },
        ),
        (
            "key_override[1].replacement",
            Place::Slot {
                kind: FeatureSlotKind::KeyOverride,
                index: 1,
                field: SlotField::OverrideReplacement,
            },
        ),
        (
            "key_override[1].trigger",
            Place::Slot {
                kind: FeatureSlotKind::KeyOverride,
                index: 1,
                field: SlotField::OverrideTrigger,
            },
        ),
        (
            "alt_repeat_key[2].keycode",
            Place::Slot {
                kind: FeatureSlotKind::AltRepeat,
                index: 2,
                field: SlotField::AltKey,
            },
        ),
        (
            "alt_repeat_key[2].alt_keycode",
            Place::Slot {
                kind: FeatureSlotKind::AltRepeat,
                index: 2,
                field: SlotField::AltAlternate,
            },
        ),
    ];
    for (kind, index) in [(FeatureSlotKind::Macro, 2), (FeatureSlotKind::TapDance, 2)] {
        let code = slots::slot_keycode(kind, index).unwrap();
        for (path, expected) in &forms {
            let mut doc = common::load("corne.vil");
            plant(&mut doc, path, code);
            let hits: Vec<_> = slots::uses(&doc)
                .into_iter()
                .filter(|u| u.path == *path)
                .collect();
            assert_eq!(hits.len(), 1, "{path}");
            assert_eq!(
                (hits[0].kind, hits[0].index, hits[0].place),
                (kind, index, *expected),
                "{path}"
            );
        }
    }
}

#[test]
fn dangling_references_are_not_uses() {
    let mut doc = common::load("corne.vil");
    let m20 = common::kc(&doc, "M20");
    let td40 = common::kc(&doc, "TD(40)");
    plant(&mut doc, "layout[0][0][0]", m20);
    plant(&mut doc, "layout[0][0][1]", td40);
    assert!(
        slots::uses(&doc)
            .iter()
            .all(|u| !u.path.starts_with("layout[0][0]"))
    );
}

#[test]
fn other_codes_are_not_uses() {
    let mut doc = common::load("corne.vil");
    for (i, name) in ["DYN_MACRO_PLAY1", "USER00", "FN_MO13", "LT1(KC_A)"]
        .iter()
        .enumerate()
    {
        let kc = common::kc(&doc, name);
        plant(&mut doc, &format!("layout[0][1][{i}]"), kc);
    }
    assert!(
        slots::uses(&doc)
            .iter()
            .all(|u| !u.path.starts_with("layout[0][1]"))
    );
    assert_eq!(
        slots::referenced_slot(Keycode(0x7700)),
        Some((FeatureSlotKind::Macro, 0))
    );
    assert_eq!(
        slots::referenced_slot(Keycode(0x77FF)),
        Some((FeatureSlotKind::Macro, 255))
    );
    assert_eq!(
        slots::referenced_slot(Keycode(0x5700)),
        Some((FeatureSlotKind::TapDance, 0))
    );
    assert_eq!(slots::referenced_slot(Keycode(0x7C56)), None);
}

#[test]
fn fixture_swap_renumbers_layer_keys() {
    let mut doc = common::load("corne.vil");
    let before = doc.clone();
    slots::swap(&mut doc, FeatureSlotKind::Macro, 2, 3).unwrap();
    assert_eq!(common::wire_at(&doc, 7, 4, 0), "M3");
    assert_eq!(common::wire_at(&doc, 7, 5, 0), "M2");
    assert_eq!(doc.macros[2], before.macros[3]);
    assert_eq!(doc.macros[3], before.macros[2]);

    let mut doc = common::load("corne.vil");
    let before = doc.clone();
    slots::swap(&mut doc, FeatureSlotKind::Macro, 0, 9).unwrap();
    assert_eq!(common::wire_at(&doc, 0, 6, 1), "M9");
    assert_eq!(doc.macros[9], before.macros[0]);
    assert_eq!(doc.macros[0], Macro::default());
}

#[test]
fn every_path_follows_a_swap() {
    let base = common::load("corne.vil");
    let paths = all_paths(&base);
    for (kind, section) in [
        (FeatureSlotKind::Macro, "macro"),
        (FeatureSlotKind::TapDance, "tap_dance"),
    ] {
        let code2 = slots::slot_keycode(kind, 2).unwrap();
        let code5 = slots::slot_keycode(kind, 5).unwrap();
        for path in &paths {
            let moved = follow(path, section, 2, 5);
            for (planted, expected) in [(code2, code5), (code5, code2)] {
                let mut doc = base.clone();
                plant(&mut doc, path, planted);
                slots::swap(&mut doc, kind, 2, 5).unwrap();
                assert_eq!(value_at(&doc, &moved), expected, "{path}");
            }
        }
    }
}

#[test]
fn other_family_references_do_not_follow() {
    let base = common::load("corne.vil");
    let paths = all_paths(&base);
    for (kind, section, other) in [
        (FeatureSlotKind::Macro, "macro", FeatureSlotKind::TapDance),
        (
            FeatureSlotKind::TapDance,
            "tap_dance",
            FeatureSlotKind::Macro,
        ),
    ] {
        let code = slots::slot_keycode(other, 2).unwrap();
        for path in &paths {
            let mut doc = base.clone();
            plant(&mut doc, path, code);
            slots::swap(&mut doc, kind, 2, 5).unwrap();
            assert_eq!(value_at(&doc, &follow(path, section, 2, 5)), code, "{path}");
        }
    }
}

#[test]
fn self_references_follow() {
    let mut doc = common::load("corne.vil");
    let m2 = slots::slot_keycode(FeatureSlotKind::Macro, 2).unwrap();
    let m5 = slots::slot_keycode(FeatureSlotKind::Macro, 5).unwrap();
    doc.macros[2] = Macro {
        actions: vec![MacroAction::Tap(vec![m2, m5])],
    };
    slots::swap(&mut doc, FeatureSlotKind::Macro, 2, 5).unwrap();
    assert_eq!(
        doc.macros[5],
        Macro {
            actions: vec![MacroAction::Tap(vec![m5, m2])],
        }
    );

    let mut doc = common::load("corne.vil");
    let td1 = slots::slot_keycode(FeatureSlotKind::TapDance, 1).unwrap();
    let td4 = slots::slot_keycode(FeatureSlotKind::TapDance, 4).unwrap();
    doc.tap_dance.as_mut().unwrap()[1].on_hold = td1;
    slots::swap(&mut doc, FeatureSlotKind::TapDance, 1, 4).unwrap();
    assert_eq!(doc.tap_dance.as_ref().unwrap()[4].on_hold, td4);
}

#[test]
fn dangling_and_other_kinds_are_untouched() {
    let mut doc = common::load("corne.vil");
    let names = ["M20", "TD(40)", "DYN_MACRO_PLAY1", "LT2(KC_A)", "MO(2)"];
    let paths: Vec<String> = (0..names.len())
        .map(|i| format!("layout[1][1][{i}]"))
        .collect();
    for (path, name) in paths.iter().zip(names) {
        let kc = common::kc(&doc, name);
        plant(&mut doc, path, kc);
    }
    let before: Vec<Keycode> = paths.iter().map(|p| value_at(&doc, p)).collect();
    slots::swap(&mut doc, FeatureSlotKind::Macro, 2, 5).unwrap();
    slots::swap(&mut doc, FeatureSlotKind::TapDance, 2, 5).unwrap();
    let after: Vec<Keycode> = paths.iter().map(|p| value_at(&doc, p)).collect();
    assert_eq!(after, before);
}

#[test]
fn index_limit() {
    let mut doc = common::load("corne.vil");
    doc.macros.resize(300, Macro::default());
    let m3 = slots::slot_keycode(FeatureSlotKind::Macro, 3).unwrap();
    plant(&mut doc, "layout[0][0][0]", m3);
    let before = doc.clone();
    let err = slots::swap(&mut doc, FeatureSlotKind::Macro, 3, 280).unwrap_err();
    assert_eq!(
        err,
        OpError::Invalid("macro 3 is used by keys that cannot point at macro 280".into())
    );
    assert_eq!(doc, before);
    slots::swap(&mut doc, FeatureSlotKind::Macro, 270, 280).unwrap();
}

#[test]
fn kinds_without_references_only_swap() {
    for kind in [
        FeatureSlotKind::Combo,
        FeatureSlotKind::KeyOverride,
        FeatureSlotKind::AltRepeat,
    ] {
        let mut doc = common::load("corne.vil");
        // Give slot 0 a keycode so that the exchange moves something visible.
        let path = match kind {
            FeatureSlotKind::Combo => "combo[0][0]",
            FeatureSlotKind::KeyOverride => "key_override[0].trigger",
            _ => "alt_repeat_key[0].keycode",
        };
        let key_a = common::kc(&doc, "KC_A");
        plant(&mut doc, path, key_a);
        let before = doc.clone();
        slots::swap(&mut doc, kind, 0, 7).unwrap();
        assert_eq!(slots::value(&doc, kind, 0), slots::value(&before, kind, 7));
        assert_eq!(slots::value(&doc, kind, 7), slots::value(&before, kind, 0));
        let section = slots::section(kind);
        let outside = |d: &Document| {
            d.keycodes()
                .into_iter()
                .filter(|(p, _)| !in_slot(p, section, 0, 7))
                .collect::<Vec<_>>()
        };
        assert_eq!(outside(&doc), outside(&before));
    }
}

#[test]
fn random_swaps_preserve_meaning() {
    let base = common::load("corne.vil");
    let paths = all_paths(&base);
    let mut codes: Vec<Keycode> = (0..16)
        .map(|n| slots::slot_keycode(FeatureSlotKind::Macro, n).unwrap())
        .collect();
    codes.extend((0..32).map(|n| slots::slot_keycode(FeatureSlotKind::TapDance, n).unwrap()));
    for name in ["M20", "TD(40)", "KC_A", "LT1(KC_B)"] {
        codes.push(common::kc(&base, name));
    }
    let mut rng = Rng(0x2545_F491_4F6C_DD1D);
    let mut renumbering_cases = 0;
    for case in 0..500 {
        let mut doc = base.clone();
        let mut plants = BTreeMap::new();
        for _ in 0..40 {
            let path = paths[rng.below(paths.len())].clone();
            plants.insert(path, codes[rng.below(codes.len())]);
        }
        doc.replace_keycodes(&plants).unwrap();
        let mut names = Names::default();
        for i in 0..16 {
            if rng.below(3) == 0 {
                names.macros.insert(i, format!("m{i}"));
            }
        }
        for i in 0..32 {
            if rng.below(3) == 0 {
                names.combos.insert(i, format!("c{i}"));
            }
        }
        let kind = slots::KINDS[rng.below(slots::KINDS.len())];
        let count = slots::count(&doc, kind);
        let (a, b) = (rng.below(count), rng.below(count));
        let section = slots::section(kind);
        let pi = |i: usize| {
            if i == a {
                b
            } else if i == b {
                a
            } else {
                i
            }
        };

        let mut new = doc.clone();
        slots::swap(&mut new, kind, a, b).unwrap();

        // 1. Every keycode outside the two swapped slots keeps its meaning.
        let old_map: BTreeMap<String, Keycode> = doc.keycodes().into_iter().collect();
        let new_map: BTreeMap<String, Keycode> = new.keycodes().into_iter().collect();
        let outside = |m: &BTreeMap<String, Keycode>| {
            m.keys()
                .filter(|p| !in_slot(p, section, a, b))
                .cloned()
                .collect::<Vec<_>>()
        };
        assert_eq!(outside(&old_map), outside(&new_map), "case {case}");
        for path in outside(&old_map) {
            assert_eq!(
                meaning(&new, new_map[&path], 0),
                meaning(&doc, old_map[&path], 0),
                "case {case}: {path}"
            );
        }

        // 2. Each slot keeps its contents under the exchange, with references resolved.
        for i in 0..count {
            if slots::has_references(kind) {
                let old = meaning(&doc, slots::slot_keycode(kind, i).unwrap(), 0);
                let moved = meaning(&new, slots::slot_keycode(kind, pi(i)).unwrap(), 0);
                assert_eq!(moved, old, "case {case}: slot {i}");
            } else {
                assert_eq!(
                    slots::value(&new, kind, pi(i)),
                    slots::value(&doc, kind, i),
                    "case {case}: slot {i}"
                );
            }
        }

        // 3. Swapping again restores the document; the order of the two indices does not matter.
        let mut back = new.clone();
        slots::swap(&mut back, kind, a, b).unwrap();
        assert_eq!(back, doc, "case {case}: involution");
        let mut mirrored = doc.clone();
        slots::swap(&mut mirrored, kind, b, a).unwrap();
        assert_eq!(mirrored, new, "case {case}: symmetry");

        // 4. Macro bytes and validation issues are unchanged.
        assert_eq!(
            validate::macro_buffer_bytes(&new),
            validate::macro_buffer_bytes(&doc),
            "case {case}"
        );
        assert_eq!(
            validate::validate(&new).len(),
            validate::validate(&doc).len(),
            "case {case}"
        );

        // 5. The result survives a save and reload.
        let (reloaded, _) = Document::from_vil_bytes(&new.to_vil_bytes()).unwrap();
        assert_eq!(reloaded, new, "case {case}");

        // Names exchange the same way and return on a second exchange.
        let mut once = names.clone();
        slots::swap_names(&mut once, kind, a, b);
        slots::swap_names(&mut once, kind, a, b);
        assert_eq!(once, names, "case {case}: names");
        if slots::uses(&doc)
            .iter()
            .any(|u| u.kind == kind && (u.index == a || u.index == b))
        {
            renumbering_cases += 1;
        }
    }
    // The property must exercise renumbering, not only the cases where nothing is referenced.
    assert!(
        renumbering_cases > 100,
        "only {renumbering_cases} cases renumbered"
    );
}

#[test]
fn slot_keycode_round_trips() {
    assert_eq!(
        slots::slot_keycode(FeatureSlotKind::Macro, 255),
        Some(Keycode(0x77FF))
    );
    assert_eq!(
        slots::slot_keycode(FeatureSlotKind::TapDance, 0),
        Some(Keycode(0x5700))
    );
    assert_eq!(slots::slot_keycode(FeatureSlotKind::Macro, 256), None);
    assert_eq!(slots::slot_keycode(FeatureSlotKind::Combo, 0), None);
    for index in [0, 7, 255] {
        let kc = slots::slot_keycode(FeatureSlotKind::TapDance, index).unwrap();
        assert_eq!(
            slots::referenced_slot(kc),
            Some((FeatureSlotKind::TapDance, index))
        );
    }
}
