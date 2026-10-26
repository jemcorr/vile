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
use crate::editor::{FeatureSlotKind, FeatureSlotValue};
use crate::keycode::Keycode;
use crate::metadata::Names;
use crate::ops::{KeyRef, Pos};

/// A layer needs this many changed keys before a layer-wide story is considered.
const MIN_LAYER_KEYS: usize = 4;

/// The feature families with fixed slot indices, in the order the story checks them.
const SLOT_KINDS: [FeatureSlotKind; 5] = [
    FeatureSlotKind::Combo,
    FeatureSlotKind::Macro,
    FeatureSlotKind::TapDance,
    FeatureSlotKind::KeyOverride,
    FeatureSlotKind::AltRepeat,
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Intent {
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
        for kind in SLOT_KINDS {
            let count = slot_count(&self.doc, kind);
            if count != slot_count(self.current, kind) {
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
                if !slot_is_set(&value) {
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
        slot_value(&self.doc, kind, index)
    }

    fn current_slot(&self, kind: FeatureSlotKind, index: usize) -> Option<FeatureSlotValue> {
        slot_value(self.current, kind, index)
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
            if let (Some(va), Some(vb)) = (slot_value(doc, *kind, *a), slot_value(doc, *kind, *b)) {
                put_slot_value(doc, *kind, *a, vb);
                put_slot_value(doc, *kind, *b, va);
            }
            if *follow {
                swap_names(names, *kind, *a, *b);
            }
        }
        Intent::SlotCopied { kind, from, to } => {
            if let Some(value) = slot_value(doc, *kind, *from) {
                put_slot_value(doc, *kind, *to, value);
            }
        }
    }
}

fn swap_names(names: &mut Names, kind: FeatureSlotKind, a: usize, b: usize) {
    let map = match kind {
        FeatureSlotKind::Combo => &mut names.combos,
        FeatureSlotKind::Macro => &mut names.macros,
        _ => return,
    };
    let (first, second) = (map.remove(&a), map.remove(&b));
    if let Some(name) = first {
        map.insert(b, name);
    }
    if let Some(name) = second {
        map.insert(a, name);
    }
}

fn slot_count(doc: &Document, kind: FeatureSlotKind) -> usize {
    match kind {
        FeatureSlotKind::Combo => doc.combo.as_ref().map_or(0, Vec::len),
        FeatureSlotKind::Macro => doc.macros.len(),
        FeatureSlotKind::TapDance => doc.tap_dance.as_ref().map_or(0, Vec::len),
        FeatureSlotKind::KeyOverride => doc.key_override.as_ref().map_or(0, Vec::len),
        FeatureSlotKind::AltRepeat => doc.alt_repeat_key.as_ref().map_or(0, Vec::len),
    }
}

fn slot_value(doc: &Document, kind: FeatureSlotKind, index: usize) -> Option<FeatureSlotValue> {
    Some(match kind {
        FeatureSlotKind::Combo => FeatureSlotValue::Combo(*doc.combo.as_ref()?.get(index)?),
        FeatureSlotKind::Macro => FeatureSlotValue::Macro(doc.macros.get(index)?.clone()),
        FeatureSlotKind::TapDance => {
            FeatureSlotValue::TapDance(*doc.tap_dance.as_ref()?.get(index)?)
        }
        FeatureSlotKind::KeyOverride => {
            FeatureSlotValue::KeyOverride(*doc.key_override.as_ref()?.get(index)?)
        }
        FeatureSlotKind::AltRepeat => {
            FeatureSlotValue::AltRepeat(*doc.alt_repeat_key.as_ref()?.get(index)?)
        }
    })
}

/// Stores `value` in slot `index` when the kinds match and the slot exists.
fn put_slot_value(
    doc: &mut Document,
    kind: FeatureSlotKind,
    index: usize,
    value: FeatureSlotValue,
) {
    match (kind, value) {
        (FeatureSlotKind::Combo, FeatureSlotValue::Combo(v)) => {
            if let Some(cell) = doc.combo.as_mut().and_then(|list| list.get_mut(index)) {
                *cell = v;
            }
        }
        (FeatureSlotKind::Macro, FeatureSlotValue::Macro(v)) => {
            if let Some(cell) = doc.macros.get_mut(index) {
                *cell = v;
            }
        }
        (FeatureSlotKind::TapDance, FeatureSlotValue::TapDance(v)) => {
            if let Some(cell) = doc.tap_dance.as_mut().and_then(|list| list.get_mut(index)) {
                *cell = v;
            }
        }
        (FeatureSlotKind::KeyOverride, FeatureSlotValue::KeyOverride(v)) => {
            if let Some(cell) = doc
                .key_override
                .as_mut()
                .and_then(|list| list.get_mut(index))
            {
                *cell = v;
            }
        }
        (FeatureSlotKind::AltRepeat, FeatureSlotValue::AltRepeat(v)) => {
            if let Some(cell) = doc
                .alt_repeat_key
                .as_mut()
                .and_then(|list| list.get_mut(index))
            {
                *cell = v;
            }
        }
        _ => {}
    }
}

/// True when the slot holds something, by the rules of the slot editors (slots.ts).
fn slot_is_set(value: &FeatureSlotValue) -> bool {
    match value {
        FeatureSlotValue::Combo(combo) => {
            combo.keys.iter().any(|&kc| kc != Keycode::NO) || combo.output != Keycode::NO
        }
        FeatureSlotValue::Macro(m) => !m.actions.is_empty(),
        FeatureSlotValue::TapDance(td) => [td.on_tap, td.on_hold, td.on_double_tap, td.on_tap_hold]
            .iter()
            .any(|&kc| kc != Keycode::NO),
        FeatureSlotValue::KeyOverride(ko) => {
            ko.trigger != Keycode::NO || ko.replacement != Keycode::NO
        }
        FeatureSlotValue::AltRepeat(ar) => {
            ar.keycode != Keycode::NO
                || ar.alt_keycode != Keycode::NO
                || ar.allowed_mods != 0
                || ar.options != 0
        }
    }
}
