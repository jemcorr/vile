//! Change stories: the difference between a baseline and the current document explained as
//! user-level actions, inferred from the two documents only, so it works against the last save
//! and the last commit alike. Stories are exact: `applied` is the reference with every intent
//! applied, and `rest == diff(&applied, current, board)`.
//!
//! Every intent is an outcome statement that is true for the documents ("these two keys now hold
//! each other's keycodes"), never a claim about what the user clicked. Every write stores the
//! current document's value at a position that differs, so an intent can remove differences but
//! never add one. Heuristics only choose a shorter story.

use crate::board::Board;
use crate::diff::{DiffEntry, diff};
use crate::document::{Document, Slot};
use crate::keycode::Keycode;
use crate::metadata::Names;
use crate::ops::{self, KeyRef, Pos};
use crate::slots::{self, FeatureSlotKind, FeatureSlotValue};

/// A layer needs this many changed keys before a layer-wide story is considered.
const MIN_LAYER_KEYS: usize = 4;

/// Two layers count as similar when this share of their keys is equal (after `norm`).
const SIMILAR: f64 = 0.8;

/// A reorder is told only when it removes at least this many changed positions.
const GAIN: usize = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Intent {
    /// The layers are in another order: `order[new] = old`. `renumbered` says the layer
    /// references followed their layers. `names` says the layer names followed too.
    LayersReordered {
        order: Vec<usize>,
        renumbered: bool,
        names: bool,
    },
    /// Every key on `layer` now holds `value` (KC_NO or KC_TRNS).
    LayerFilled {
        layer: usize,
        value: Keycode,
        keys: usize,
    },
    /// Layer `to` now equals layer `from`, which is unchanged.
    LayerCopied { from: usize, to: usize, keys: usize },
    /// The two keys exchanged keycodes.
    KeysSwapped { a: KeyRef, b: KeyRef },
    /// The keycode of `from` now sits at `to`, and `from` is KC_NO.
    KeyMoved { from: KeyRef, to: KeyRef },
    /// Two or more keys on `layer` now hold `value` (KC_NO or KC_TRNS).
    KeysFilled {
        layer: usize,
        value: Keycode,
        positions: Vec<Pos>,
    },
    /// The two slots exchanged contents. `names` is true when their names exchanged too.
    SlotsSwapped {
        kind: FeatureSlotKind,
        a: usize,
        b: usize,
        names: bool,
    },
    /// Slot `to` now equals the unchanged slot `from`. Names are not copied.
    SlotCopied {
        kind: FeatureSlotKind,
        from: usize,
        to: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Story {
    pub intents: Vec<Intent>,
    /// The reference with every intent applied.
    pub applied: Document,
    pub applied_names: Names,
    /// Document changes the intents do not explain, compared with `applied`.
    pub rest: Vec<DiffEntry>,
}

/// Explains how `current` differs from `base`. `board` decides which positions count, as in
/// `diff::diff`: the board keys when it matches both documents, else every matrix position.
pub fn explain(
    base: &Document,
    base_names: &Names,
    current: &Document,
    current_names: &Names,
    board: Option<&Board>,
) -> Story {
    let board = board.filter(|b| b.matches(base) && b.matches(current));
    let layout_matches = base.layer_count() == current.layer_count()
        && base.rows() == current.rows()
        && base.cols() == current.cols();
    let mut explainer = Explainer {
        current,
        current_names,
        board,
        positions: positions_of(base, board),
        doc: base.clone(),
        names: base_names.clone(),
        intents: Vec::new(),
    };
    if layout_matches {
        explainer.reorder();
        explainer.layer_intents();
        explainer.swap_keys();
        explainer.move_keys();
        explainer.filled_groups();
    }
    explainer.slot_intents();
    let Explainer {
        intents,
        doc,
        names,
        ..
    } = explainer;
    let rest = diff(&doc, current, board);
    Story {
        intents,
        applied: doc,
        applied_names: names,
        rest,
    }
}

/// `reference` with `intents` applied in order, and the names after them. `explain` returns
/// exactly this for the intents it chose.
pub fn apply_intents(
    reference: &Document,
    names: &Names,
    intents: &[Intent],
    board: Option<&Board>,
) -> (Document, Names) {
    let mut doc = reference.clone();
    let mut names = names.clone();
    for intent in intents {
        apply_one(&mut doc, &mut names, board, intent);
    }
    (doc, names)
}

struct Explainer<'a> {
    current: &'a Document,
    current_names: &'a Names,
    board: Option<&'a Board>,
    positions: Vec<Pos>,
    doc: Document,
    names: Names,
    intents: Vec<Intent>,
}

impl Explainer<'_> {
    fn push(&mut self, intent: Intent) {
        apply_one(&mut self.doc, &mut self.names, self.board, &intent);
        self.intents.push(intent);
    }

    fn layers(&self) -> usize {
        self.current.layer_count()
    }

    /// Step 1: the current layers are the base layers in another order. Accepted only when the
    /// best order removes at least GAIN changed positions.
    fn reorder(&mut self) {
        let base = &self.doc;
        if !reorder_gate(base, self.current, &self.positions) {
            return;
        }
        let raw = differing(base, self.current, &self.positions);
        let candidates = reorder_candidates(base, self.current, &self.positions);
        // Renumbered first, then the candidate order: a tie keeps the earlier one.
        let mut best: Option<(usize, Vec<usize>, bool)> = None;
        for renumbered in [true, false] {
            for order in &candidates {
                let mut doc = base.clone();
                if !reorder_layers(&mut doc, order, renumbered) {
                    continue;
                }
                let score = differing(&doc, self.current, &self.positions);
                if best.as_ref().is_none_or(|(known, _, _)| score < *known) {
                    best = Some((score, order.clone(), renumbered));
                }
            }
        }
        let Some((score, order, renumbered)) = best else {
            return;
        };
        if score + GAIN > raw {
            return;
        }
        let mut remapped = self.names.clone();
        remapped.remap_layers(&inverse(&order));
        let names = layer_name_differences(&remapped, self.current_names)
            < layer_name_differences(&self.names, self.current_names);
        self.push(Intent::LayersReordered {
            order,
            renumbered,
            names,
        });
    }

    /// True when the keycodes differ and both slots hold a keycode.
    fn is_changed(&self, key: KeyRef) -> bool {
        matches!((keycode(&self.doc, key), keycode(self.current, key)), (Some(a), Some(b)) if a != b)
    }

    /// Every changed key, as (layer, row, col) in order.
    fn changed_keys(&self) -> Vec<KeyRef> {
        (0..self.layers())
            .flat_map(|layer| self.positions.iter().map(move |&pos| KeyRef { layer, pos }))
            .filter(|&key| self.is_changed(key))
            .collect()
    }

    fn layer_intents(&mut self) {
        for layer in 0..self.layers() {
            let keys = self
                .changed_keys()
                .iter()
                .filter(|key| key.layer == layer)
                .count();
            if keys < MIN_LAYER_KEYS {
                continue;
            }
            if let Some(value) = self.filled_value(layer) {
                self.push(Intent::LayerFilled { layer, value, keys });
            } else if let Some(from) = self.copy_source(layer) {
                self.push(Intent::LayerCopied {
                    from,
                    to: layer,
                    keys,
                });
            }
        }
    }

    /// KC_NO or KC_TRNS when every key of the current layer holds it.
    fn filled_value(&self, layer: usize) -> Option<Keycode> {
        [Keycode::NO, Keycode::TRNS].into_iter().find(|&value| {
            self.positions.iter().all(|&pos| {
                keycode(self.current, KeyRef { layer, pos }).is_none_or(|kc| kc == value)
            })
        })
    }

    /// An unchanged layer that the current layer equals, when the current layer is not trivial
    /// (fewer than MIN_LAYER_KEYS keys that are neither KC_NO nor KC_TRNS).
    fn copy_source(&self, layer: usize) -> Option<usize> {
        let real_keys = self
            .positions
            .iter()
            .filter_map(|&pos| keycode(self.current, KeyRef { layer, pos }))
            .filter(|&kc| kc != Keycode::NO && kc != Keycode::TRNS)
            .count();
        if real_keys < MIN_LAYER_KEYS {
            return None;
        }
        (0..self.layers()).filter(|&m| m != layer).find(|&m| {
            self.positions.iter().all(|&pos| {
                let wanted = slot(self.current, m, pos);
                slot(&self.doc, m, pos) == wanted && slot(self.current, layer, pos) == wanted
            })
        })
    }

    /// Exchanges of two changed keys. Each first key takes the partner on its own layer first,
    /// then the nearest one, then the lowest position.
    fn swap_keys(&mut self) {
        let candidates = self.changed_keys();
        for &a in &candidates {
            if !self.is_changed(a) {
                continue;
            }
            let (Some(ra), Some(ca)) = (keycode(&self.doc, a), keycode(self.current, a)) else {
                continue;
            };
            let partner = candidates
                .iter()
                .copied()
                .filter(|&b| {
                    b != a
                        && self.is_changed(b)
                        && keycode(&self.doc, b) == Some(ca)
                        && keycode(self.current, b) == Some(ra)
                })
                .min_by_key(|&b| proximity(a, b));
            if let Some(b) = partner {
                self.push(Intent::KeysSwapped { a, b });
            }
        }
    }

    /// Moves of a changed key that is now KC_NO to a changed key that now holds its keycode.
    /// The target may hold another keycode before the move; the displaced keycode then shows
    /// in the baseline values of the target.
    fn move_keys(&mut self) {
        let candidates = self.changed_keys();
        for &from in &candidates {
            if !self.is_changed(from) {
                continue;
            }
            let (Some(rf), Some(cf)) = (keycode(&self.doc, from), keycode(self.current, from))
            else {
                continue;
            };
            if cf != Keycode::NO || rf == Keycode::NO {
                continue;
            }
            let target = candidates
                .iter()
                .copied()
                .filter(|&to| {
                    to != from && self.is_changed(to) && keycode(self.current, to) == Some(rf)
                })
                .min_by_key(|&to| proximity(from, to));
            if let Some(to) = target {
                self.push(Intent::KeyMoved { from, to });
            }
        }
    }

    /// Two or more remaining changed keys on one layer that now hold KC_NO, then KC_TRNS.
    fn filled_groups(&mut self) {
        for layer in 0..self.layers() {
            for value in [Keycode::NO, Keycode::TRNS] {
                let positions: Vec<Pos> = self
                    .positions
                    .iter()
                    .copied()
                    .filter(|&pos| {
                        let key = KeyRef { layer, pos };
                        self.is_changed(key) && keycode(self.current, key) == Some(value)
                    })
                    .collect();
                if positions.len() >= 2 {
                    self.push(Intent::KeysFilled {
                        layer,
                        value,
                        positions,
                    });
                }
            }
        }
    }

    fn slot_intents(&mut self) {
        for kind in slots::KINDS {
            let count = slots::count(&self.doc, kind);
            if count != slots::count(self.current, kind) {
                continue;
            }
            for a in 0..count {
                if !self.slot_changed(kind, a) {
                    continue;
                }
                let partner = (0..count).find(|&b| {
                    b != a
                        && self.slot_changed(kind, b)
                        && self.current_slot(kind, a) == self.working_slot(kind, b)
                        && self.current_slot(kind, b) == self.working_slot(kind, a)
                        && self.working_slot(kind, a) != self.working_slot(kind, b)
                });
                if let Some(b) = partner {
                    let names = self.names_follow(kind, a, b);
                    self.push(Intent::SlotsSwapped { kind, a, b, names });
                }
            }
            for to in 0..count {
                if !self.slot_changed(kind, to) {
                    continue;
                }
                let Some(value) = self.current_slot(kind, to) else {
                    continue;
                };
                if !slots::is_set(&value) {
                    continue;
                }
                let source = (0..count).find(|&from| {
                    from != to
                        && !self.slot_changed(kind, from)
                        && self.current_slot(kind, from) == Some(value.clone())
                });
                if let Some(from) = source {
                    self.push(Intent::SlotCopied { kind, from, to });
                }
            }
        }
    }

    fn slot_changed(&self, kind: FeatureSlotKind, index: usize) -> bool {
        self.working_slot(kind, index) != self.current_slot(kind, index)
    }

    fn working_slot(&self, kind: FeatureSlotKind, index: usize) -> Option<FeatureSlotValue> {
        slots::value(&self.doc, kind, index)
    }

    fn current_slot(&self, kind: FeatureSlotKind, index: usize) -> Option<FeatureSlotValue> {
        slots::value(self.current, kind, index)
    }

    /// Names follow a swap only when the current names are exactly the swapped names.
    /// Only combos and macros have names.
    fn names_follow(&self, kind: FeatureSlotKind, a: usize, b: usize) -> bool {
        let (before, after) = match kind {
            FeatureSlotKind::Combo => (&self.names.combos, &self.current_names.combos),
            FeatureSlotKind::Macro => (&self.names.macros, &self.current_names.macros),
            _ => return true,
        };
        before.get(&a) == after.get(&b) && before.get(&b) == after.get(&a)
    }
}

/// The positions the story reads: the board keys when the board matches `doc`, else every
/// matrix position. Sorted by (row, col).
fn positions_of(doc: &Document, board: Option<&Board>) -> Vec<Pos> {
    match board.filter(|b| b.matches(doc)) {
        Some(board) => {
            let mut positions: Vec<Pos> = board
                .keys
                .iter()
                .map(|key| Pos {
                    row: key.row,
                    col: key.col,
                })
                .collect();
            positions.sort();
            positions
        }
        None => (0..doc.rows())
            .flat_map(|row| (0..doc.cols()).map(move |col| Pos { row, col }))
            .collect(),
    }
}

fn slot(doc: &Document, layer: usize, pos: Pos) -> Option<Slot> {
    doc.get(layer, pos.row, pos.col)
}

/// A keycode with its layer number erased, so a layer reference compares by kind alone.
fn norm(kc: Keycode) -> Keycode {
    match kc.layer_ref() {
        Some(_) => kc.with_layer(0).unwrap_or(kc),
        None => kc,
    }
}

fn norm_slot(doc: &Document, layer: usize, pos: Pos) -> Option<Slot> {
    slot(doc, layer, pos).map(|cell| match cell {
        Slot::Key(kc) => Slot::Key(norm(kc)),
        Slot::Absent => Slot::Absent,
    })
}

/// Positions whose slots differ, summed over all layers.
fn differing(a: &Document, b: &Document, positions: &[Pos]) -> usize {
    (0..a.layer_count())
        .map(|layer| {
            positions
                .iter()
                .filter(|&&pos| slot(a, layer, pos) != slot(b, layer, pos))
                .count()
        })
        .sum()
}

/// A layer with fewer than MIN_LAYER_KEYS keys that are neither KC_NO nor KC_TRNS.
fn is_trivial(doc: &Document, layer: usize, positions: &[Pos]) -> bool {
    let real = positions
        .iter()
        .filter_map(|&pos| keycode(doc, KeyRef { layer, pos }))
        .filter(|&kc| kc != Keycode::NO && kc != Keycode::TRNS)
        .count();
    real < MIN_LAYER_KEYS
}

/// Gate: a current layer differs from the base layer of its number, and a non-trivial base layer
/// of another number is SIMILAR to it.
fn reorder_gate(base: &Document, current: &Document, positions: &[Pos]) -> bool {
    let n = base.layer_count();
    let total = positions.len() as f64;
    (0..n).any(|j| {
        let same_as_base = positions
            .iter()
            .all(|&pos| norm_slot(base, j, pos) == norm_slot(current, j, pos));
        !same_as_base
            && (0..n).any(|i| {
                i != j
                    && !is_trivial(base, i, positions)
                    && positions
                        .iter()
                        .filter(|&&pos| norm_slot(base, i, pos) == norm_slot(current, j, pos))
                        .count() as f64
                        >= SIMILAR * total
            })
    })
}

/// Every single swap, every single move, then the greedy matching by similarity. Each entry is
/// `order[new] = old`.
fn reorder_candidates(base: &Document, current: &Document, positions: &[Pos]) -> Vec<Vec<usize>> {
    let n = base.layer_count();
    let identity: Vec<usize> = (0..n).collect();
    let mut out = Vec::new();
    for a in 0..n {
        for b in a + 1..n {
            let mut order = identity.clone();
            order.swap(a, b);
            out.push(order);
        }
    }
    for from in 0..n {
        for to in 0..n {
            if from != to {
                let mut order = identity.clone();
                let moved = order.remove(from);
                order.insert(to, moved);
                out.push(order);
            }
        }
    }
    let similarity = |old: usize, new: usize| {
        positions
            .iter()
            .filter(|&&pos| norm_slot(base, old, pos) == norm_slot(current, new, pos))
            .count()
    };
    let mut pairs: Vec<(usize, usize, usize)> = (0..n)
        .flat_map(|old| (0..n).map(move |new| (old, new)))
        .map(|(old, new)| (similarity(old, new), old, new))
        .collect();
    pairs.sort_by(|x, y| {
        y.0.cmp(&x.0)
            .then(x.1.abs_diff(x.2).cmp(&y.1.abs_diff(y.2)))
            .then((x.1, x.2).cmp(&(y.1, y.2)))
    });
    let mut order = vec![usize::MAX; n];
    let mut taken = vec![false; n];
    for (_, old, new) in pairs {
        if !taken[old] && order[new] == usize::MAX {
            order[new] = old;
            taken[old] = true;
        }
    }
    out.push(order);
    out
}

/// `perm[old] = new` for `order[new] = old`.
fn inverse(order: &[usize]) -> Vec<usize> {
    let mut perm = vec![0; order.len()];
    for (new, &old) in order.iter().enumerate() {
        perm[old] = new;
    }
    perm
}

/// Puts the layers in `order`. Renumbered: references follow their layers, as
/// `ops::permute_layers` does. Otherwise only the contents move. False when the order is refused.
fn reorder_layers(doc: &mut Document, order: &[usize], renumbered: bool) -> bool {
    if order.len() != doc.layer_count() {
        return false;
    }
    if renumbered {
        let mut next = doc.clone();
        if ops::permute_layers(&mut next, &inverse(order)).is_err() {
            return false;
        }
        *doc = next;
        return true;
    }
    let layout = doc.layout.clone();
    for (new, &old) in order.iter().enumerate() {
        doc.layout[new] = layout[old].clone();
    }
    if let Some(encoders) = doc.encoder_layout.clone()
        && encoders.len() == order.len()
    {
        doc.encoder_layout = Some(order.iter().map(|&old| encoders[old].clone()).collect());
    }
    true
}

/// Number of layer indices whose names differ.
fn layer_name_differences(a: &Names, b: &Names) -> usize {
    let indices: std::collections::BTreeSet<usize> =
        a.layers.keys().chain(b.layers.keys()).copied().collect();
    indices
        .into_iter()
        .filter(|index| a.layers.get(index) != b.layers.get(index))
        .count()
}

fn keycode(doc: &Document, key: KeyRef) -> Option<Keycode> {
    match slot(doc, key.layer, key.pos)? {
        Slot::Key(kc) => Some(kc),
        Slot::Absent => None,
    }
}

/// Writes a keycode, only where the document already has a key.
fn set_keycode(doc: &mut Document, key: KeyRef, kc: Keycode) {
    if let Some(cell) = doc
        .layout
        .get_mut(key.layer)
        .and_then(|layer| layer.get_mut(key.pos.row))
        .and_then(|row| row.get_mut(key.pos.col))
        && *cell != Slot::Absent
    {
        *cell = Slot::Key(kc);
    }
}

/// Manhattan distance first, then layers: a same-layer partner wins before this is used.
fn proximity(a: KeyRef, b: KeyRef) -> (bool, usize, usize, Pos) {
    let distance = a.pos.row.abs_diff(b.pos.row) + a.pos.col.abs_diff(b.pos.col);
    (a.layer != b.layer, distance, b.layer, b.pos)
}

fn apply_one(doc: &mut Document, names: &mut Names, board: Option<&Board>, intent: &Intent) {
    match intent {
        Intent::LayersReordered {
            order,
            renumbered,
            names: remap,
        } => {
            if reorder_layers(doc, order, *renumbered) && *remap {
                names.remap_layers(&inverse(order));
            }
        }
        Intent::LayerFilled { layer, value, .. } => {
            for pos in positions_of(doc, board) {
                set_keycode(doc, KeyRef { layer: *layer, pos }, *value);
            }
        }
        Intent::LayerCopied { from, to, .. } => {
            for pos in positions_of(doc, board) {
                if let Some(kc) = keycode(doc, KeyRef { layer: *from, pos }) {
                    set_keycode(doc, KeyRef { layer: *to, pos }, kc);
                }
            }
        }
        Intent::KeysSwapped { a, b } => {
            if let (Some(ka), Some(kb)) = (keycode(doc, *a), keycode(doc, *b)) {
                set_keycode(doc, *a, kb);
                set_keycode(doc, *b, ka);
            }
        }
        Intent::KeyMoved { from, to } => {
            if let Some(kc) = keycode(doc, *from) {
                set_keycode(doc, *to, kc);
                set_keycode(doc, *from, Keycode::NO);
            }
        }
        Intent::KeysFilled {
            layer,
            value,
            positions,
        } => {
            for &pos in positions {
                set_keycode(doc, KeyRef { layer: *layer, pos }, *value);
            }
        }
        Intent::SlotsSwapped {
            kind,
            a,
            b,
            names: follow,
        } => {
            if let (Some(va), Some(vb)) =
                (slots::value(doc, *kind, *a), slots::value(doc, *kind, *b))
            {
                slots::put(doc, *kind, *a, vb);
                slots::put(doc, *kind, *b, va);
            }
            if *follow {
                slots::swap_names(names, *kind, *a, *b);
            }
        }
        Intent::SlotCopied { kind, from, to } => {
            if let Some(value) = slots::value(doc, *kind, *from) {
                slots::put(doc, *kind, *to, value);
            }
        }
    }
}
