//! Editing operations on a Document. Every operation validates its inputs
//! first and leaves the document unchanged when it returns an error.

use crate::document::{Document, Slot};
use crate::keycode::{Keycode, KeycodeError};
use std::collections::HashSet;

/// A matrix position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Pos {
    pub row: usize,
    pub col: usize,
}

/// A matrix position on one layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KeyRef {
    pub layer: usize,
    pub pos: Pos,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MirrorMode {
    /// Exchange the left and right halves.
    Swap,
    /// Copy the left half onto the right half.
    LeftToRight,
    /// Copy the right half onto the left half.
    RightToLeft,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OpError {
    #[error("layer {layer} does not exist (the file has {count} layers)")]
    NoSuchLayer { layer: usize, count: usize },
    #[error("r{row}c{col} is not a key position")]
    NoKey { row: usize, col: usize },
    #[error("selection sizes differ: {expected} copied, {actual} selected")]
    SizeMismatch { expected: usize, actual: usize },
    #[error("the same key appears in both selections")]
    Overlap,
    #[error("clipboard is empty")]
    EmptyClipboard,
    #[error("clipboard holds {held}, not {wanted}")]
    WrongClipboard {
        held: &'static str,
        wanted: &'static str,
    },
    #[error("the board does not match this file")]
    BoardMismatch,
    #[error("{section} slot {index} does not exist (the file has {count})")]
    NoSuchSlot {
        section: &'static str,
        index: usize,
        count: usize,
    },
    #[error(transparent)]
    Keycode(#[from] KeycodeError),
    #[error("{0}")]
    Invalid(String),
}

/// Ok carries warnings for the user (usually empty).
pub type OpResult = Result<Vec<String>, OpError>;

fn check_layer(doc: &Document, layer: usize) -> Result<(), OpError> {
    let count = doc.layer_count();
    if layer < count {
        Ok(())
    } else {
        Err(OpError::NoSuchLayer { layer, count })
    }
}

/// True when `perm` holds each of 0..n exactly once.
fn is_permutation(perm: &[usize], n: usize) -> bool {
    if perm.len() != n {
        return false;
    }
    let mut seen = vec![false; n];
    for &p in perm {
        if p >= n || seen[p] {
            return false;
        }
        seen[p] = true;
    }
    true
}

/// Every key on the layer becomes `kc`; empty positions stay empty.
pub fn fill_layer(doc: &mut Document, layer: usize, kc: Keycode) -> OpResult {
    check_layer(doc, layer)?;
    for row in doc.layout[layer].iter_mut() {
        for slot in row.iter_mut() {
            if matches!(slot, Slot::Key(_)) {
                *slot = Slot::Key(kc);
            }
        }
    }
    Ok(vec![])
}

pub fn layer_contents(doc: &Document, layer: usize) -> Result<Vec<Vec<Slot>>, OpError> {
    check_layer(doc, layer)?;
    Ok(doc.layout[layer].clone())
}

pub fn set_layer_contents(doc: &mut Document, layer: usize, contents: &[Vec<Slot>]) -> OpResult {
    check_layer(doc, layer)?;
    let current = &doc.layout[layer];
    let same_shape = current.len() == contents.len()
        && current.iter().zip(contents).all(|(a, b)| {
            a.len() == b.len()
                && a.iter()
                    .zip(b)
                    .all(|(x, y)| matches!(x, Slot::Absent) == matches!(y, Slot::Absent))
        });
    if !same_shape {
        return Err(OpError::Invalid("layer shape differs".to_string()));
    }
    doc.layout[layer] = contents.to_vec();
    Ok(vec![])
}

/// Copies the keys of `src` onto `dst`. Keycodes are copied verbatim.
pub fn copy_layer(doc: &mut Document, src: usize, dst: usize) -> OpResult {
    check_layer(doc, src)?;
    check_layer(doc, dst)?;
    let contents = layer_contents(doc, src)?;
    set_layer_contents(doc, dst, &contents)
}

/// `perm[old] = new` moves each layer to its new index and renumbers every
/// layer reference in the file.
pub fn permute_layers(doc: &mut Document, perm: &[usize]) -> OpResult {
    let n = doc.layer_count();
    if !is_permutation(perm, n) {
        return Err(OpError::Invalid(format!(
            "not a permutation of the {n} layers"
        )));
    }
    let mut new = doc.clone();

    // 1. Move the layers (and their encoders).
    for (old, layer) in doc.layout.iter().enumerate() {
        new.layout[perm[old]] = layer.clone();
    }
    if let Some(encoders) = &doc.encoder_layout {
        let mut moved = encoders.clone();
        for (old, layer) in encoders.iter().enumerate() {
            moved[perm[old]] = layer.clone();
        }
        new.encoder_layout = Some(moved);
    }

    // 2. Renumber every layer reference that points at a layer of this file.
    let mut failure: Option<KeycodeError> = None;
    new.for_each_keycode_mut(&mut |kc| {
        if failure.is_some() {
            return;
        }
        let current = *kc;
        if let Some(l) = current.layer_ref()
            && usize::from(l) < n
        {
            let target = u8::try_from(perm[usize::from(l)]).unwrap_or(u8::MAX);
            match current.with_layer(target) {
                Ok(renumbered) => *kc = renumbered,
                Err(e) => failure = Some(e),
            }
        }
    });
    if let Some(e) = failure {
        return Err(OpError::Keycode(e));
    }

    // 3. Key override layer masks: bit i < min(n, 16) moves to bit perm[i].
    let m = n.min(16);
    if let Some(overrides) = &mut new.key_override {
        for ko in overrides.iter_mut() {
            let mut mask = u32::from(ko.layers) & !((1u32 << m) - 1);
            for (i, &target) in perm.iter().enumerate().take(m) {
                if ko.layers >> i & 1 == 1 {
                    if target >= 16 {
                        return Err(OpError::Invalid(
                            "key override layer mask cannot hold layer 16 or higher".to_string(),
                        ));
                    }
                    mask |= 1 << target;
                }
            }
            ko.layers = mask as u16;
        }
    }

    // 4. FN_MO13 and FN_MO23 keep addressing layers 1, 2 and 3.
    let mut warnings = Vec::new();
    let moves_fixed_layer = (1..=3).any(|l| l < n && perm[l] != l);
    if moves_fixed_layer
        && new
            .keycodes()
            .iter()
            .any(|(_, kc)| !kc.fixed_layer_refs().is_empty())
    {
        warnings.push(
            "FN_MO13 and FN_MO23 always use layers 1, 2 and 3; vile did not renumber them"
                .to_string(),
        );
    }

    *doc = new;
    Ok(warnings)
}

/// Exchanges two layers. The same index changes nothing.
pub fn swap_layers(doc: &mut Document, a: usize, b: usize) -> OpResult {
    check_layer(doc, a)?;
    check_layer(doc, b)?;
    if a == b {
        return Ok(vec![]);
    }
    let mut perm: Vec<usize> = (0..doc.layer_count()).collect();
    perm.swap(a, b);
    permute_layers(doc, &perm)
}

/// Checks the layer, then that every position holds a key.
fn check_positions(doc: &Document, layer: usize, positions: &[Pos]) -> Result<(), OpError> {
    check_layer(doc, layer)?;
    for pos in positions {
        if !matches!(doc.get(layer, pos.row, pos.col), Some(Slot::Key(_))) {
            return Err(OpError::NoKey {
                row: pos.row,
                col: pos.col,
            });
        }
    }
    Ok(())
}

fn write_slot(doc: &mut Document, layer: usize, pos: Pos, kc: Keycode) {
    doc.layout[layer][pos.row][pos.col] = Slot::Key(kc);
}

pub fn set_keys(doc: &mut Document, layer: usize, positions: &[Pos], kc: Keycode) -> OpResult {
    check_positions(doc, layer, positions)?;
    for &pos in positions {
        write_slot(doc, layer, pos, kc);
    }
    Ok(vec![])
}

pub fn clear_keys(doc: &mut Document, layer: usize, positions: &[Pos]) -> OpResult {
    set_keys(doc, layer, positions, Keycode::NO)
}

pub fn transparent_keys(doc: &mut Document, layer: usize, positions: &[Pos]) -> OpResult {
    set_keys(doc, layer, positions, Keycode::TRNS)
}

/// The keycodes at `positions`, in the order of `positions`.
pub fn read_keys(doc: &Document, layer: usize, positions: &[Pos]) -> Result<Vec<Keycode>, OpError> {
    check_positions(doc, layer, positions)?;
    Ok(positions
        .iter()
        .map(|pos| match doc.get(layer, pos.row, pos.col) {
            Some(Slot::Key(kc)) => kc,
            _ => Keycode::NO,
        })
        .collect())
}

/// `positions[i]` gets `values[i]`.
pub fn write_keys(
    doc: &mut Document,
    layer: usize,
    positions: &[Pos],
    values: &[Keycode],
) -> OpResult {
    check_layer(doc, layer)?;
    if positions.len() != values.len() {
        return Err(OpError::SizeMismatch {
            expected: values.len(),
            actual: positions.len(),
        });
    }
    check_positions(doc, layer, positions)?;
    for (&pos, &kc) in positions.iter().zip(values) {
        write_slot(doc, layer, pos, kc);
    }
    Ok(vec![])
}

/// Exchanges the keys of `a` and `b` pairwise. The two lists may use different layers.
pub fn swap_keys(doc: &mut Document, a: &[KeyRef], b: &[KeyRef]) -> OpResult {
    if a.len() != b.len() {
        return Err(OpError::SizeMismatch {
            expected: a.len(),
            actual: b.len(),
        });
    }
    if a.iter().any(|r| b.contains(r)) {
        return Err(OpError::Overlap);
    }
    for r in a.iter().chain(b) {
        check_positions(doc, r.layer, &[r.pos])?;
    }
    let read = |doc: &Document, refs: &[KeyRef]| -> Vec<Keycode> {
        refs.iter()
            .map(|r| match doc.get(r.layer, r.pos.row, r.pos.col) {
                Some(Slot::Key(kc)) => kc,
                _ => Keycode::NO,
            })
            .collect()
    };
    let values_a = read(doc, a);
    let values_b = read(doc, b);
    for (r, kc) in a.iter().zip(values_b) {
        write_slot(doc, r.layer, r.pos, kc);
    }
    for (r, kc) in b.iter().zip(values_a) {
        write_slot(doc, r.layer, r.pos, kc);
    }
    Ok(vec![])
}

/// Where pasted keys go: the copied positions, or the selection when it is
/// given and fits.
pub fn paste_targets(copied: &[Pos], selection: &[Pos]) -> Result<Vec<Pos>, OpError> {
    if selection.is_empty() {
        return Ok(copied.to_vec());
    }
    let copied_set: HashSet<Pos> = copied.iter().copied().collect();
    let selection_set: HashSet<Pos> = selection.iter().copied().collect();
    if copied_set == selection_set {
        return Ok(copied.to_vec());
    }
    if selection.len() == copied.len() {
        return Ok(selection.to_vec());
    }
    Err(OpError::SizeMismatch {
        expected: copied.len(),
        actual: selection.len(),
    })
}
