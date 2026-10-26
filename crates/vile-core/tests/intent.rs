mod common;

use std::time::Instant;

use common::{board, kc};
use vile_core::diff::diff;
use vile_core::editor::FeatureSlotKind;
use vile_core::intent::{Intent, Story, apply_intents, explain};
use vile_core::metadata::Names;
use vile_core::ops::{self, KeyRef, Pos};
use vile_core::{Document, Keycode, Slot};

fn pos(row: usize, col: usize) -> Pos {
    Pos { row, col }
}

fn key(layer: usize, row: usize, col: usize) -> KeyRef {
    KeyRef {
        layer,
        pos: pos(row, col),
    }
}

/// Explains `current` against `base` on the corne board, then checks the story invariants:
/// the rest is the diff of the applied reference, intents never add differences, and
/// `apply_intents` reproduces the applied reference.
fn story_for(base: &Document, current: &Document) -> Story {
    story_with_names(base, &Names::default(), current, &Names::default())
}

fn story_with_names(
    base: &Document,
    base_names: &Names,
    current: &Document,
    current_names: &Names,
) -> Story {
    let board = board();
    let story = explain(base, base_names, current, current_names, Some(&board));
    assert_eq!(
        diff(&story.applied, current, Some(&board)),
        story.rest,
        "rest is the diff of the applied reference"
    );
    assert!(
        story.rest.len() <= diff(base, current, Some(&board)).len(),
        "intents never add differences"
    );
    let (applied, applied_names) = apply_intents(base, base_names, &story.intents, Some(&board));
    assert!(applied == story.applied, "apply_intents reproduces applied");
    assert!(applied_names == story.applied_names, "names reproduce too");
    story
}

fn keycode_at(doc: &Document, layer: usize, p: Pos) -> Keycode {
    match doc.get(layer, p.row, p.col).expect("position exists") {
        Slot::Key(kc) => kc,
        Slot::Absent => panic!("no key at {p:?} on layer {layer}"),
    }
}

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

#[test]
fn swapped_keys_pair_up() {
    let base = common::load("corne.vil");
    let mut current = base.clone();
    ops::swap_keys(&mut current, &[key(0, 0, 2)], &[key(0, 2, 4)]).unwrap();
    let story = story_for(&base, &current);
    assert_eq!(
        story.intents,
        vec![Intent::KeysSwapped {
            a: key(0, 0, 2),
            b: key(0, 2, 4)
        }]
    );
    assert!(story.rest.is_empty());
}

#[test]
fn equal_values_pick_the_nearest_partner() {
    let mut base = common::load("corne.vil");
    let w = kc(&base, "KC_W");
    ops::set_keys(&mut base, 0, &[pos(1, 0)], w).unwrap();
    let mut current = base.clone();
    let (w, l) = (kc(&base, "KC_W"), kc(&base, "KC_L"));
    ops::write_keys(
        &mut current,
        0,
        &[pos(0, 2), pos(2, 4), pos(1, 0)],
        &[w, l, l],
    )
    .unwrap();
    let story = story_for(&base, &current);
    assert_eq!(
        story.intents,
        vec![Intent::KeysSwapped {
            a: key(0, 0, 2),
            b: key(0, 1, 0)
        }]
    );
    assert_eq!(story.rest.len(), 1);
    assert_eq!(story.rest[0].layer, Some(0));
    assert_eq!(story.rest[0].row, Some(2));
    assert_eq!(story.rest[0].col, Some(4));
}

#[test]
fn moved_key() {
    let base = common::load("corne.vil");
    let mut current = base.clone();
    // r0c5 holds KC_V in the base, so the move replaces it.
    ops::set_keys(&mut current, 0, &[pos(0, 5)], kc(&base, "KC_B")).unwrap();
    ops::set_keys(&mut current, 0, &[pos(0, 1)], Keycode::NO).unwrap();
    let story = story_for(&base, &current);
    assert_eq!(
        story.intents,
        vec![Intent::KeyMoved {
            from: key(0, 0, 1),
            to: key(0, 0, 5)
        }]
    );
    assert!(story.rest.is_empty());
}

#[test]
fn cleared_keys_group() {
    let base = common::load("corne.vil");
    let mut current = base.clone();
    ops::clear_keys(&mut current, 0, &[pos(0, 3), pos(0, 4)]).unwrap();
    ops::clear_keys(&mut current, 0, &[pos(1, 5)]).unwrap();
    let story = story_for(&base, &current);
    assert_eq!(
        story.intents,
        vec![Intent::KeysFilled {
            layer: 0,
            value: Keycode::NO,
            positions: vec![pos(0, 3), pos(0, 4), pos(1, 5)]
        }]
    );
    assert!(story.rest.is_empty());

    let mut one = base.clone();
    ops::clear_keys(&mut one, 0, &[pos(0, 3)]).unwrap();
    let story = story_for(&base, &one);
    assert!(story.intents.is_empty());
    assert_eq!(story.rest.len(), 1);
}

#[test]
fn coincidental_values_are_not_copies() {
    let base = common::load("corne.vil");
    let mut current = base.clone();
    // KC_L is also at r0c2, so this is not a copy.
    ops::set_keys(&mut current, 0, &[pos(0, 1)], kc(&base, "KC_L")).unwrap();
    let story = story_for(&base, &current);
    assert!(story.intents.is_empty());
    assert_eq!(story.rest.len(), 1);
}

#[test]
fn cleared_layer() {
    let base = common::load("corne.vil");
    let mut current = base.clone();
    ops::fill_layer(&mut current, 0, Keycode::NO).unwrap();
    let story = story_for(&base, &current);
    assert_eq!(
        story.intents,
        vec![Intent::LayerFilled {
            layer: 0,
            value: Keycode::NO,
            keys: 41
        }]
    );
    assert!(story.rest.is_empty());
}

#[test]
fn transparent_layer() {
    let base = common::load("corne.vil");
    let mut current = base.clone();
    let trns = kc(&base, "KC_TRNS");
    ops::fill_layer(&mut current, 1, trns).unwrap();
    let not_trns = (0..8)
        .flat_map(|r| (0..7).map(move |c| pos(r, c)))
        .filter(|&p| matches!(base.get(1, p.row, p.col), Some(Slot::Key(_))))
        .filter(|&p| keycode_at(&base, 1, p) != trns)
        .count();
    assert!(not_trns > 0);
    let story = story_for(&base, &current);
    assert_eq!(
        story.intents,
        vec![Intent::LayerFilled {
            layer: 1,
            value: trns,
            keys: not_trns
        }]
    );
    assert!(story.rest.is_empty());
}

#[test]
fn copied_layer() {
    let base = common::load("corne.vil");
    let mut current = base.clone();
    ops::copy_layer(&mut current, 1, 3).unwrap();
    let story = story_for(&base, &current);
    assert_eq!(
        story.intents,
        vec![Intent::LayerCopied {
            from: 1,
            to: 3,
            keys: 26
        }]
    );
    assert!(story.rest.is_empty());

    // Once the source changes too, the copy is no longer a copy of an unchanged layer.
    ops::set_keys(&mut current, 1, &[pos(0, 1)], kc(&base, "KC_Z")).unwrap();
    let story = story_for(&base, &current);
    assert!(
        !story
            .intents
            .iter()
            .any(|intent| matches!(intent, Intent::LayerCopied { .. })),
        "{:?}",
        story.intents
    );
    assert!(!story.rest.is_empty());
}

#[test]
fn swapped_and_copied_slots() {
    let base = common::load("corne.vil");
    let combo0 = base.combo.as_ref().unwrap()[0];
    let combo1 = base.combo.as_ref().unwrap()[1];
    let mut swapped = base.clone();
    ops::set_combo(&mut swapped, 0, combo1).unwrap();
    ops::set_combo(&mut swapped, 1, combo0).unwrap();
    let story = story_for(&base, &swapped);
    assert_eq!(
        story.intents,
        vec![Intent::SlotsSwapped {
            kind: FeatureSlotKind::Combo,
            a: 0,
            b: 1,
            names: true
        }]
    );
    assert!(story.rest.is_empty());

    let mut copied = base.clone();
    ops::set_combo(&mut copied, 5, combo0).unwrap();
    let story = story_for(&base, &copied);
    assert_eq!(
        story.intents,
        vec![Intent::SlotCopied {
            kind: FeatureSlotKind::Combo,
            from: 0,
            to: 5
        }]
    );
    assert!(story.rest.is_empty());
}

#[test]
fn slot_names_follow_a_swap_only_when_exchanged() {
    let base = common::load("corne.vil");
    let combo0 = base.combo.as_ref().unwrap()[0];
    let combo1 = base.combo.as_ref().unwrap()[1];
    let mut base_names = Names::default();
    base_names.combos.insert(0, "Chord".into());
    let mut swapped = base.clone();
    ops::set_combo(&mut swapped, 0, combo1).unwrap();
    ops::set_combo(&mut swapped, 1, combo0).unwrap();

    // The names stay where they were: the swap is still told, and the names differ.
    let story = story_with_names(&base, &base_names, &swapped, &Names::default());
    assert_eq!(
        story.intents,
        vec![Intent::SlotsSwapped {
            kind: FeatureSlotKind::Combo,
            a: 0,
            b: 1,
            names: false
        }]
    );
    assert_eq!(story.applied_names, base_names);

    // The names moved with the contents: nothing is left to say about them.
    let mut moved_names = Names::default();
    moved_names.combos.insert(1, "Chord".into());
    let story = story_with_names(&base, &base_names, &swapped, &moved_names);
    assert_eq!(
        story.intents,
        vec![Intent::SlotsSwapped {
            kind: FeatureSlotKind::Combo,
            a: 0,
            b: 1,
            names: true
        }]
    );
    assert_eq!(story.applied_names, moved_names);
}

#[test]
fn shape_changes_skip_layout_intents() {
    let base = common::load("corne.vil");
    let mut current = base.clone();
    ops::fill_layer(&mut current, 0, Keycode::NO).unwrap();
    current.layout.pop();
    let story = story_for(&base, &current);
    assert!(story.intents.is_empty(), "{:?}", story.intents);
    assert!(!story.rest.is_empty());
}

#[test]
fn stray_key_breaks_the_layer_fill() {
    // Clearing layer 0 and setting r0c1 to KC_Z, which the cleared r2c5 held: no layer story.
    // The new KC_Z is a move from r2c5, and the other cleared keys form one group.
    let base = common::load("corne.vil");
    let mut current = base.clone();
    ops::fill_layer(&mut current, 0, Keycode::NO).unwrap();
    ops::set_keys(&mut current, 0, &[pos(0, 1)], kc(&base, "KC_Z")).unwrap();
    let story = story_for(&base, &current);
    let cleared: Vec<Pos> = (0..8)
        .flat_map(|r| (0..7).map(move |c| pos(r, c)))
        .filter(|&p| p != pos(0, 1) && p != pos(2, 5))
        .filter(|&p| matches!(base.get(0, p.row, p.col), Some(Slot::Key(kc)) if kc != Keycode::NO))
        .collect();
    assert_eq!(cleared.len(), 39);
    assert_eq!(
        story.intents,
        vec![
            Intent::KeyMoved {
                from: key(0, 2, 5),
                to: key(0, 0, 1)
            },
            Intent::KeysFilled {
                layer: 0,
                value: Keycode::NO,
                positions: cleared
            },
        ]
    );
    assert!(story.rest.is_empty());
}

#[test]
fn swap_on_one_layer_next_to_a_cleared_layer() {
    let base = common::load("corne.vil");
    let mut current = base.clone();
    ops::fill_layer(&mut current, 0, Keycode::NO).unwrap();
    ops::swap_keys(&mut current, &[key(1, 0, 1)], &[key(1, 1, 1)]).unwrap();
    let story = story_for(&base, &current);
    assert_eq!(
        story.intents,
        vec![
            Intent::LayerFilled {
                layer: 0,
                value: Keycode::NO,
                keys: 41
            },
            Intent::KeysSwapped {
                a: key(1, 0, 1),
                b: key(1, 1, 1)
            },
        ]
    );
    assert!(story.rest.is_empty());
}

#[test]
fn random_edits_stay_exact() {
    let base = common::load("corne.vil");
    let board = board();
    let positions: Vec<Pos> = board.keys.iter().map(|k| pos(k.row, k.col)).collect();
    let values = ["KC_A", "KC_B", "KC_NO", "KC_TRNS", "KC_Z"];
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    let mut explained = 0;
    for _ in 0..200 {
        let mut current = base.clone();
        let steps = 1 + rng.below(5);
        for _ in 0..steps {
            let layer = rng.below(current.layer_count());
            let other = rng.below(current.layer_count());
            let p = positions[rng.below(positions.len())];
            let q = positions[rng.below(positions.len())];
            let value = kc(&base, values[rng.below(values.len())]);
            // Errors leave the document unchanged; the sequence just does less.
            let _ = match rng.below(6) {
                0 => ops::swap_keys(
                    &mut current,
                    &[KeyRef { layer, pos: p }],
                    &[KeyRef {
                        layer: other,
                        pos: q,
                    }],
                ),
                1 => ops::clear_keys(&mut current, layer, &[p, q]),
                2 => ops::set_keys(&mut current, layer, &[p], value),
                3 => ops::fill_layer(&mut current, layer, value),
                4 => ops::copy_layer(&mut current, layer, other),
                _ => {
                    let (a, b) = (rng.below(8), rng.below(8));
                    let list = current.combo.clone().unwrap();
                    let _ = ops::set_combo(&mut current, a, list[b]);
                    ops::set_combo(&mut current, b, list[a])
                }
            };
        }
        let story = story_for(&base, &current);
        explained += story.intents.len();
    }
    assert!(explained > 0, "random edits should produce some stories");
}

#[test]
fn full_corne_with_all_layers_changed_is_explained_quickly() {
    let base = common::load("corne.vil");
    let values = [
        kc(&base, "KC_A"),
        kc(&base, "KC_NO"),
        kc(&base, "KC_TRNS"),
        kc(&base, "KC_Z"),
    ];
    let mut current = base.clone();
    let mut rng = Rng(0x2545_f491_4f6c_dd1d);
    for layer in current.layout.iter_mut() {
        for row in layer.iter_mut() {
            for cell in row.iter_mut() {
                if let Slot::Key(_) = cell {
                    *cell = Slot::Key(values[rng.below(values.len())]);
                }
            }
        }
    }
    let started = Instant::now();
    let story = story_for(&base, &current);
    let elapsed = started.elapsed();
    eprintln!(
        "explain on corne with all 10 layers changed: {:?}, {} intents, {} rest rows",
        elapsed,
        story.intents.len(),
        story.rest.len()
    );
    assert!(elapsed.as_secs() < 10, "took {elapsed:?}");
}
