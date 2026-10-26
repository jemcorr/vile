//! Feature slots: the fixed-index families, their empty values, and (later) the keycodes that
//! reference macros and tap dances.

use std::collections::BTreeMap;

use crate::document::{AltRepeatKey, Combo, Document, KeyOverride, Macro, TapDance};
use crate::keycode::{Keycode, KeycodeKind, QK_MACRO, QK_TAP_DANCE};
use crate::metadata::Names;
use crate::ops::{self, OpError, OpResult};

/// A fixed-index feature family. Copying content never changes keycode references.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FeatureSlotKind {
    Combo,
    Macro,
    TapDance,
    KeyOverride,
    AltRepeat,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeatureSlotValue {
    Combo(Combo),
    Macro(Macro),
    TapDance(TapDance),
    KeyOverride(KeyOverride),
    AltRepeat(AltRepeatKey),
}

/// The feature families with fixed slot indices, in the order the story checks them.
pub const KINDS: [FeatureSlotKind; 5] = [
    FeatureSlotKind::Combo,
    FeatureSlotKind::Macro,
    FeatureSlotKind::TapDance,
    FeatureSlotKind::KeyOverride,
    FeatureSlotKind::AltRepeat,
];

/// The section name used in `OpError::NoSuchSlot` and in keycode paths.
pub fn section(kind: FeatureSlotKind) -> &'static str {
    match kind {
        FeatureSlotKind::Combo => "combo",
        FeatureSlotKind::Macro => "macro",
        FeatureSlotKind::TapDance => "tap_dance",
        FeatureSlotKind::KeyOverride => "key_override",
        FeatureSlotKind::AltRepeat => "alt_repeat_key",
    }
}

/// "a combo", "a macro", … for messages.
pub fn article_name(kind: FeatureSlotKind) -> &'static str {
    match kind {
        FeatureSlotKind::Combo => "a combo",
        FeatureSlotKind::Macro => "a macro",
        FeatureSlotKind::TapDance => "a tap dance",
        FeatureSlotKind::KeyOverride => "a key override",
        FeatureSlotKind::AltRepeat => "an alt repeat slot",
    }
}

pub fn count(doc: &Document, kind: FeatureSlotKind) -> usize {
    match kind {
        FeatureSlotKind::Combo => doc.combo.as_ref().map_or(0, Vec::len),
        FeatureSlotKind::Macro => doc.macros.len(),
        FeatureSlotKind::TapDance => doc.tap_dance.as_ref().map_or(0, Vec::len),
        FeatureSlotKind::KeyOverride => doc.key_override.as_ref().map_or(0, Vec::len),
        FeatureSlotKind::AltRepeat => doc.alt_repeat_key.as_ref().map_or(0, Vec::len),
    }
}

/// The value of slot `index`, or None when the slot does not exist.
pub fn value(doc: &Document, kind: FeatureSlotKind, index: usize) -> Option<FeatureSlotValue> {
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

/// The value of slot `index`; `NoSuchSlot` when it does not exist.
pub fn get(
    doc: &Document,
    kind: FeatureSlotKind,
    index: usize,
) -> Result<FeatureSlotValue, OpError> {
    value(doc, kind, index).ok_or_else(|| OpError::NoSuchSlot {
        section: section(kind),
        index,
        count: count(doc, kind),
    })
}

/// Writes `value` through the validating operation for its family (for example `ops::set_macro`).
/// A value of another family is an `Invalid` error.
pub fn set(
    doc: &mut Document,
    kind: FeatureSlotKind,
    index: usize,
    value: FeatureSlotValue,
) -> OpResult {
    match (kind, value) {
        (FeatureSlotKind::Combo, FeatureSlotValue::Combo(v)) => ops::set_combo(doc, index, v),
        (FeatureSlotKind::Macro, FeatureSlotValue::Macro(v)) => ops::set_macro(doc, index, v),
        (FeatureSlotKind::TapDance, FeatureSlotValue::TapDance(v)) => {
            ops::set_tap_dance(doc, index, v)
        }
        (FeatureSlotKind::KeyOverride, FeatureSlotValue::KeyOverride(v)) => {
            ops::set_key_override(doc, index, v)
        }
        (FeatureSlotKind::AltRepeat, FeatureSlotValue::AltRepeat(v)) => {
            ops::set_alt_repeat_key(doc, index, v)
        }
        _ => Err(OpError::Invalid("feature clipboard type mismatch".into())),
    }
}

/// Stores `value` in slot `index` without validation. Returns false, and changes nothing, when
/// the kinds do not match or the slot does not exist.
pub fn put(
    doc: &mut Document,
    kind: FeatureSlotKind,
    index: usize,
    value: FeatureSlotValue,
) -> bool {
    match (kind, value) {
        (FeatureSlotKind::Combo, FeatureSlotValue::Combo(v)) => {
            assign(doc.combo.as_mut(), index, v)
        }
        (FeatureSlotKind::Macro, FeatureSlotValue::Macro(v)) => {
            assign(Some(&mut doc.macros), index, v)
        }
        (FeatureSlotKind::TapDance, FeatureSlotValue::TapDance(v)) => {
            assign(doc.tap_dance.as_mut(), index, v)
        }
        (FeatureSlotKind::KeyOverride, FeatureSlotValue::KeyOverride(v)) => {
            assign(doc.key_override.as_mut(), index, v)
        }
        (FeatureSlotKind::AltRepeat, FeatureSlotValue::AltRepeat(v)) => {
            assign(doc.alt_repeat_key.as_mut(), index, v)
        }
        _ => false,
    }
}

fn assign<T>(list: Option<&mut Vec<T>>, index: usize, value: T) -> bool {
    match list.and_then(|list| list.get_mut(index)) {
        Some(cell) => {
            *cell = value;
            true
        }
        None => false,
    }
}

/// True when the slot holds something, by the rules of the slot editors (slots.ts).
pub fn is_set(value: &FeatureSlotValue) -> bool {
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

/// The name map of a family. Only combos and macros have names.
pub fn names_of(names: &mut Names, kind: FeatureSlotKind) -> Option<&mut BTreeMap<usize, String>> {
    match kind {
        FeatureSlotKind::Combo => Some(&mut names.combos),
        FeatureSlotKind::Macro => Some(&mut names.macros),
        _ => None,
    }
}

pub fn set_name(names: &mut Names, kind: FeatureSlotKind, index: usize, name: Option<String>) {
    if let Some(map) = names_of(names, kind) {
        match name {
            Some(name) => {
                map.insert(index, name);
            }
            None => {
                map.remove(&index);
            }
        }
    }
}

pub fn swap_names(names: &mut Names, kind: FeatureSlotKind, a: usize, b: usize) {
    if let Some(map) = names_of(names, kind) {
        let first = map.remove(&a);
        let second = map.remove(&b);
        if let Some(name) = first {
            map.insert(b, name);
        }
        if let Some(name) = second {
            map.insert(a, name);
        }
    }
}

/// Exchanges the contents of slots `a` and `b` through the validating operations. Names are not
/// touched. Copying never renumbers references; see `swap` in commit 3.
pub fn swap_values(doc: &mut Document, kind: FeatureSlotKind, a: usize, b: usize) -> OpResult {
    if a == b {
        get(doc, kind, a)?;
        return Ok(Vec::new());
    }
    let av = get(doc, kind, a)?;
    let bv = get(doc, kind, b)?;
    set(doc, kind, a, bv)?;
    set(doc, kind, b, av)
}

/// The slot a keycode plays: M(n) → (Macro, n), TD(n) → (TapDance, n). Nothing else references a
/// slot.
pub fn referenced_slot(kc: Keycode) -> Option<(FeatureSlotKind, usize)> {
    match kc.kind() {
        KeycodeKind::Macro { index } => Some((FeatureSlotKind::Macro, usize::from(index))),
        KeycodeKind::TapDance { index } => Some((FeatureSlotKind::TapDance, usize::from(index))),
        _ => None,
    }
}

/// The keycode of `kind` that plays slot `index`. None for combos, overrides, alt repeat keys
/// and indices above 255.
pub fn slot_keycode(kind: FeatureSlotKind, index: usize) -> Option<Keycode> {
    let index = u16::try_from(index).ok().filter(|&n| n <= 0xFF)?;
    match kind {
        FeatureSlotKind::Macro => Some(Keycode(QK_MACRO + index)),
        FeatureSlotKind::TapDance => Some(Keycode(QK_TAP_DANCE + index)),
        _ => None,
    }
}

/// True for the families that keycodes can reference.
pub fn has_references(kind: FeatureSlotKind) -> bool {
    matches!(kind, FeatureSlotKind::Macro | FeatureSlotKind::TapDance)
}

/// The field of a slot that holds a keycode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotField {
    ComboKey(usize),
    ComboOutput,
    MacroKey { action: usize, key: usize },
    TapDance(usize),
    OverrideTrigger,
    OverrideReplacement,
    AltKey,
    AltAlternate,
}

/// Where a keycode sits in the document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Key {
        layer: usize,
        row: usize,
        col: usize,
    },
    Knob {
        layer: usize,
        knob: usize,
        clockwise: bool,
    },
    Slot {
        kind: FeatureSlotKind,
        index: usize,
        field: SlotField,
    },
}

/// Parses a path of `Document::keycodes`, for example "layout[1][0][0]", "macro[2][0][1]" or
/// "key_override[3].trigger". None for any other path.
pub fn place_of(path: &str) -> Option<Place> {
    let (name, numbers, tail) = split_path(path)?;
    let place = match (name, numbers.as_slice(), tail) {
        ("layout", &[layer, row, col], "") => Place::Key { layer, row, col },
        ("encoder_layout", &[layer, knob, side], "") if side <= 1 => Place::Knob {
            layer,
            knob,
            clockwise: side == 0,
        },
        ("macro", &[index, action, key], "") if key >= 1 => Place::Slot {
            kind: FeatureSlotKind::Macro,
            index,
            field: SlotField::MacroKey {
                action,
                key: key - 1,
            },
        },
        ("tap_dance", &[index, field], "") if field < 4 => Place::Slot {
            kind: FeatureSlotKind::TapDance,
            index,
            field: SlotField::TapDance(field),
        },
        ("combo", &[index, key], "") if key < 4 => Place::Slot {
            kind: FeatureSlotKind::Combo,
            index,
            field: SlotField::ComboKey(key),
        },
        ("combo", &[index, 4], "") => Place::Slot {
            kind: FeatureSlotKind::Combo,
            index,
            field: SlotField::ComboOutput,
        },
        ("key_override", &[index], ".trigger") => Place::Slot {
            kind: FeatureSlotKind::KeyOverride,
            index,
            field: SlotField::OverrideTrigger,
        },
        ("key_override", &[index], ".replacement") => Place::Slot {
            kind: FeatureSlotKind::KeyOverride,
            index,
            field: SlotField::OverrideReplacement,
        },
        ("alt_repeat_key", &[index], ".keycode") => Place::Slot {
            kind: FeatureSlotKind::AltRepeat,
            index,
            field: SlotField::AltKey,
        },
        ("alt_repeat_key", &[index], ".alt_keycode") => Place::Slot {
            kind: FeatureSlotKind::AltRepeat,
            index,
            field: SlotField::AltAlternate,
        },
        _ => return None,
    };
    Some(place)
}

/// Splits "name[1][2].tail" into ("name", [1, 2], ".tail").
fn split_path(path: &str) -> Option<(&str, Vec<usize>, &str)> {
    let open = path.find('[')?;
    let name = &path[..open];
    let mut rest = &path[open..];
    let mut numbers = Vec::new();
    while let Some(after) = rest.strip_prefix('[') {
        let close = after.find(']')?;
        numbers.push(after[..close].parse().ok()?);
        rest = &after[close + 1..];
    }
    Some((name, numbers, rest))
}

/// One keycode that plays a slot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotUse {
    pub kind: FeatureSlotKind,
    pub index: usize,
    pub path: String,
    pub place: Place,
}

/// Every keycode that plays an existing macro or tap dance, in `Document::keycodes` order.
/// References past the slot count are skipped.
pub fn uses(doc: &Document) -> Vec<SlotUse> {
    doc.keycodes()
        .into_iter()
        .filter_map(|(path, kc)| {
            let (kind, index) = referenced_slot(kc)?;
            if index >= count(doc, kind) {
                return None;
            }
            let place = place_of(&path)?;
            Some(SlotUse {
                kind,
                index,
                path,
                place,
            })
        })
        .collect()
}
