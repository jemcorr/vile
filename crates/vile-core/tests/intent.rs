mod common;

use std::time::Instant;

use common::{board, kc};
use vile_core::DiffSection;
use vile_core::diff::diff;
use vile_core::editor::FeatureSlotKind;
use vile_core::intent::{Intent, Story, apply_intents, explain};
use vile_core::metadata::Names;
use vile_core::ops::{self, KeyRef, Pos};
use vile_core::slots;
use vile_core::{Document, Keycode, Macro, MacroAction, Slot};

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
            names: true,
            renumbered: false,
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
            names: false,
            renumbered: false,
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
            names: true,
            renumbered: false,
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
            let _ = match rng.below(8) {
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
                5 => {
                    let (a, b) = (rng.below(8), rng.below(8));
                    let list = current.combo.clone().unwrap();
                    let _ = ops::set_combo(&mut current, a, list[b]);
                    ops::set_combo(&mut current, b, list[a])
                }
                6 => {
                    // Plant a reference to a slot, then exchange two slots of its family.
                    let kind = if rng.below(2) == 0 {
                        FeatureSlotKind::Macro
                    } else {
                        FeatureSlotKind::TapDance
                    };
                    let count = slots::count(&current, kind);
                    let (a, b) = (rng.below(count), rng.below(count));
                    let code = slots::slot_keycode(kind, a).unwrap();
                    let _ = ops::set_keys(&mut current, layer, &[p], code);
                    slots::swap(&mut current, kind, a, b)
                }
                _ => {
                    let kind = slots::KINDS[rng.below(slots::KINDS.len())];
                    let index = rng.below(slots::count(&current, kind));
                    slots::set(&mut current, kind, index, slots::empty(kind))
                }
            };
        }
        let story = story_for(&base, &current);
        explained += story.intents.len();
    }
    assert!(explained > 0, "random edits should produce some stories");
}

#[test]
fn swapped_layers_are_one_reorder_with_renumbered_references() {
    let base = common::load("corne.vil");
    let mut current = base.clone();
    ops::swap_layers(&mut current, 7, 8).unwrap();
    let story = story_for(&base, &current);
    assert_eq!(
        story.intents,
        vec![Intent::LayersReordered {
            order: vec![0, 1, 2, 3, 4, 5, 6, 8, 7, 9],
            renumbered: true,
            names: false
        }]
    );
    assert!(story.rest.is_empty());
}

#[test]
fn moved_layer_is_a_move_not_a_greedy_order() {
    let base = common::load("corne.vil");
    let mut current = base.clone();
    ops::move_layer(&mut current, 9, 1).unwrap();
    let story = story_for(&base, &current);
    assert_eq!(
        story.intents,
        vec![Intent::LayersReordered {
            order: vec![0, 9, 1, 2, 3, 4, 5, 6, 7, 8],
            renumbered: true,
            names: false
        }]
    );
    assert!(story.rest.is_empty());
}

#[test]
fn two_swaps_give_a_general_order() {
    let base = common::load("corne.vil");
    let mut current = base.clone();
    ops::swap_layers(&mut current, 7, 8).unwrap();
    ops::swap_layers(&mut current, 1, 6).unwrap();
    let story = story_for(&base, &current);
    assert_eq!(
        story.intents,
        vec![Intent::LayersReordered {
            order: vec![0, 6, 2, 3, 4, 5, 1, 8, 7, 9],
            renumbered: true,
            names: false
        }]
    );
    assert!(story.rest.is_empty());
}

#[test]
fn reorder_with_an_edit_keeps_the_edit_in_the_rest() {
    let base = common::load("corne.vil");
    let mut current = base.clone();
    ops::swap_layers(&mut current, 7, 8).unwrap();
    ops::set_keys(&mut current, 7, &[pos(0, 1)], kc(&base, "KC_A")).unwrap();
    let story = story_for(&base, &current);
    assert_eq!(
        story.intents,
        vec![Intent::LayersReordered {
            order: vec![0, 1, 2, 3, 4, 5, 6, 8, 7, 9],
            renumbered: true,
            names: false
        }]
    );
    assert_eq!(story.rest.len(), 1);
    let entry = &story.rest[0];
    assert_eq!(
        (entry.layer, entry.row, entry.col),
        (Some(7), Some(0), Some(1))
    );
    // Base layer 8 holds KC_PGUP at r0c1, so the reference after the reorder does too.
    assert_eq!(entry.old, "KC_PGUP");
    assert_eq!(entry.new, "KC_A");
}

#[test]
fn layer_names_follow_a_reorder_only_when_they_moved_with_it() {
    let base = common::load("corne.vil");
    let mut base_names = Names::default();
    base_names.layers.insert(7, "Symbols".into());
    base_names.layers.insert(8, "Navigation".into());
    let mut current = base.clone();
    ops::swap_layers(&mut current, 7, 8).unwrap();

    // swap_layers leaves the names where they were, so the reorder does not carry them.
    let story = story_with_names(&base, &base_names, &current, &base_names);
    assert_eq!(
        story.intents,
        vec![Intent::LayersReordered {
            order: vec![0, 1, 2, 3, 4, 5, 6, 8, 7, 9],
            renumbered: true,
            names: false
        }]
    );
    assert_eq!(story.applied_names, base_names);

    // Names that moved with the layers are followed.
    let mut moved_names = Names::default();
    moved_names.layers.insert(8, "Symbols".into());
    moved_names.layers.insert(7, "Navigation".into());
    let story = story_with_names(&base, &base_names, &current, &moved_names);
    assert_eq!(
        story.intents,
        vec![Intent::LayersReordered {
            order: vec![0, 1, 2, 3, 4, 5, 6, 8, 7, 9],
            renumbered: true,
            names: true
        }]
    );
    assert_eq!(story.applied_names, moved_names);
}

#[test]
fn swapping_identical_layers_is_no_story() {
    let base = common::load("corne.vil");
    let mut current = base.clone();
    ops::swap_layers(&mut current, 2, 3).unwrap();
    let story = story_for(&base, &current);
    assert!(story.intents.is_empty());
    assert!(story.rest.is_empty());
}

#[test]
fn contents_moved_without_renumbering_are_not_renumbered() {
    let base = common::load("corne.vil");
    let mut current = base.clone();
    let layer7 = ops::layer_contents(&base, 7).unwrap();
    let layer8 = ops::layer_contents(&base, 8).unwrap();
    ops::set_layer_contents(&mut current, 7, &layer8).unwrap();
    ops::set_layer_contents(&mut current, 8, &layer7).unwrap();
    let story = story_for(&base, &current);
    assert_eq!(
        story.intents,
        vec![Intent::LayersReordered {
            order: vec![0, 1, 2, 3, 4, 5, 6, 8, 7, 9],
            renumbered: false,
            names: false
        }]
    );
    assert!(story.rest.is_empty());
}

#[test]
fn key_override_layer_masks_follow_a_reorder() {
    let mut base = common::load("corne.vil");
    base.key_override.as_mut().unwrap()[0].layers = 1 << 7;
    let mut current = base.clone();
    ops::swap_layers(&mut current, 7, 8).unwrap();
    assert_eq!(current.key_override.as_ref().unwrap()[0].layers, 1 << 8);
    let story = story_for(&base, &current);
    assert!(
        story.intents.iter().any(|intent| matches!(
            intent,
            Intent::LayersReordered {
                renumbered: true,
                ..
            }
        )),
        "{:?}",
        story.intents
    );
    assert!(
        story
            .rest
            .iter()
            .all(|entry| entry.section != DiffSection::KeyOverride),
        "{:?}",
        story.rest
    );
}

#[test]
fn reorder_search_over_all_layers_is_quick_and_exact() {
    // A random renumbered permutation of all 10 layers plus one edit: the gate passes and the
    // search tries every swap, move and the greedy matching.
    let base = common::load("corne.vil");
    let mut rng = Rng(0x0123_4567_89ab_cdef);
    let mut perm: Vec<usize> = (0..base.layer_count()).collect();
    for i in (1..perm.len()).rev() {
        perm.swap(i, rng.below(i + 1));
    }
    let mut current = base.clone();
    ops::permute_layers(&mut current, &perm).unwrap();
    ops::set_keys(&mut current, 4, &[pos(0, 1)], kc(&base, "KC_A")).unwrap();
    let started = Instant::now();
    let story = story_for(&base, &current);
    let elapsed = started.elapsed();
    eprintln!(
        "reorder search over 10 layers: {elapsed:?}, intents {:?}",
        story.intents
    );
    assert!(elapsed.as_secs() < 10, "took {elapsed:?}");
    assert!(
        story.intents.iter().any(|intent| matches!(
            intent,
            Intent::LayersReordered {
                renumbered: true,
                ..
            }
        )),
        "{:?}",
        story.intents
    );
    assert_eq!(
        story.rest.len(),
        1,
        "only the edit remains: {:?}",
        story.rest
    );
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

#[test]
fn renumbered_macro_swap_is_one_story() {
    let base = common::load("corne.vil");
    let mut current = base.clone();
    slots::swap(&mut current, FeatureSlotKind::Macro, 2, 3).unwrap();
    let story = story_for(&base, &current);
    assert_eq!(
        story.intents,
        vec![Intent::SlotsSwapped {
            kind: FeatureSlotKind::Macro,
            a: 2,
            b: 3,
            names: true,
            renumbered: true,
        }]
    );
    assert!(story.rest.is_empty());
}

#[test]
fn move_to_an_empty_macro_is_renumbered() {
    let base = common::load("corne.vil");
    let mut current = base.clone();
    slots::swap(&mut current, FeatureSlotKind::Macro, 3, 9).unwrap();
    let story = story_for(&base, &current);
    assert_eq!(
        story.intents,
        vec![Intent::SlotsSwapped {
            kind: FeatureSlotKind::Macro,
            a: 3,
            b: 9,
            names: true,
            renumbered: true,
        }]
    );
    assert!(story.rest.is_empty());
}

#[test]
fn swap_without_references_is_plain() {
    let mut base = common::load("corne.vil");
    let key = kc(&base, "KC_A");
    base.macros[10] = Macro {
        actions: vec![MacroAction::Tap(vec![key])],
    };
    base.macros[11] = Macro::default();
    let mut current = base.clone();
    slots::swap(&mut current, FeatureSlotKind::Macro, 10, 11).unwrap();
    let story = story_for(&base, &current);
    assert_eq!(
        story.intents,
        vec![Intent::SlotsSwapped {
            kind: FeatureSlotKind::Macro,
            a: 10,
            b: 11,
            names: true,
            renumbered: false,
        }]
    );
    assert!(story.rest.is_empty());
}

#[test]
fn self_referencing_macro_swap() {
    let mut base = common::load("corne.vil");
    let m3 = slots::slot_keycode(FeatureSlotKind::Macro, 3).unwrap();
    base.macros[2] = Macro {
        actions: vec![MacroAction::Tap(vec![m3])],
    };
    let mut current = base.clone();
    slots::swap(&mut current, FeatureSlotKind::Macro, 2, 3).unwrap();
    let story = story_for(&base, &current);
    assert_eq!(
        story.intents,
        vec![Intent::SlotsSwapped {
            kind: FeatureSlotKind::Macro,
            a: 2,
            b: 3,
            names: true,
            renumbered: true,
        }]
    );
    assert!(story.rest.is_empty());
}

#[test]
fn contents_exchanged_without_renumbering_stay_plain() {
    let base = common::load("corne.vil");
    let mut current = base.clone();
    let two = slots::value(&base, FeatureSlotKind::Macro, 2).unwrap();
    let three = slots::value(&base, FeatureSlotKind::Macro, 3).unwrap();
    assert!(slots::put(&mut current, FeatureSlotKind::Macro, 2, three));
    assert!(slots::put(&mut current, FeatureSlotKind::Macro, 3, two));
    let story = story_for(&base, &current);
    assert_eq!(
        story.intents,
        vec![Intent::SlotsSwapped {
            kind: FeatureSlotKind::Macro,
            a: 2,
            b: 3,
            names: true,
            renumbered: false,
        }]
    );
    assert!(story.rest.is_empty());
}

#[test]
fn tap_dance_swap_renumbers_other_features() {
    let mut base = common::load("corne.vil");
    let key = kc(&base, "KC_A");
    let td1 = slots::slot_keycode(FeatureSlotKind::TapDance, 1).unwrap();
    base.combo.as_mut().unwrap()[0].output = td1;
    base.macros[0] = Macro {
        actions: vec![MacroAction::Tap(vec![td1])],
    };
    base.tap_dance.as_mut().unwrap()[1].on_tap = key;
    let mut current = base.clone();
    slots::swap(&mut current, FeatureSlotKind::TapDance, 1, 4).unwrap();
    let story = story_for(&base, &current);
    assert_eq!(
        story.intents,
        vec![Intent::SlotsSwapped {
            kind: FeatureSlotKind::TapDance,
            a: 1,
            b: 4,
            names: true,
            renumbered: true,
        }]
    );
    assert!(story.rest.is_empty());
}

#[test]
fn renumbered_swap_with_an_edit() {
    let base = common::load("corne.vil");
    let mut current = base.clone();
    slots::swap(&mut current, FeatureSlotKind::Macro, 2, 3).unwrap();
    let z = kc(&base, "KC_Z");
    ops::set_keys(&mut current, 0, &[pos(0, 1)], z).unwrap();
    let story = story_for(&base, &current);
    assert_eq!(
        story.intents,
        vec![Intent::SlotsSwapped {
            kind: FeatureSlotKind::Macro,
            a: 2,
            b: 3,
            names: true,
            renumbered: true,
        }]
    );
    assert_eq!(story.rest.len(), 1);
}
