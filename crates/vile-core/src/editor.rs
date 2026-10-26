//! An editing session over one document: undo and redo, a clipboard, the
//! diff base (file on disk or git HEAD) and safe saving.

use std::path::{Path, PathBuf};

use crate::board::Board;
use crate::diff::DiffEntry;
use crate::document::{Document, LoadError, LoadReport, Slot};
use crate::keycode::Keycode;
use crate::metadata::{Names, Sidecar, fingerprint, sidecar_path};
use crate::ops::{OpError, OpResult, Pos};
use crate::slots;

/// What the user copied last.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Clipboard {
    Keys {
        positions: Vec<Pos>,
        values: Vec<Keycode>,
    },
    Layer {
        contents: Vec<Vec<Slot>>,
    },
    /// Slots of one family, in index order, each with its name.
    Features {
        kind: FeatureSlotKind,
        entries: Vec<(FeatureSlotValue, Option<String>)>,
    },
}

pub use crate::slots::{FeatureSlotKind, FeatureSlotValue};

/// What the diff compares the current document with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffBase {
    Disk,
    GitHead,
}

#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("no file path; use save as")]
    NoPath,
    #[error("{0} changed on disk since it was opened; reload it or save it under another name")]
    ChangedOnDisk(String),
    #[error("refusing to save: the serialized file does not load back identically (bug in vile)")]
    SelfCheck,
    #[error("cannot write {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
}

/// Undo history depth.
pub const UNDO_LIMIT: usize = 500;

#[derive(Debug)]
pub struct Editor {
    doc: Document,
    /// The last version loaded from or saved to disk.
    disk: Document,
    /// The bytes of that version, when it came from a file.
    disk_bytes: Option<Vec<u8>>,
    path: Option<PathBuf>,
    board: Option<Board>,
    base: DiffBase,
    git_doc: Option<Document>,
    undo: Vec<(Document, Names)>,
    redo: Vec<(Document, Names)>,
    names: Names,
    disk_names: Names,
    metadata_warnings: Vec<String>,
    sidecar_bytes: Option<Vec<u8>>,
    names_pending: bool,
    clipboard: Option<Clipboard>,
}

impl Editor {
    /// disk = doc.clone(), disk_bytes = None, base = Disk, empty history and clipboard.
    pub fn new(doc: Document, path: Option<PathBuf>, board: Option<Board>) -> Editor {
        Editor {
            disk: doc.clone(),
            doc,
            disk_bytes: None,
            path,
            board,
            base: DiffBase::Disk,
            git_doc: None,
            undo: Vec::new(),
            redo: Vec::new(),
            clipboard: None,
            names: Names::default(),
            disk_names: Names::default(),
            metadata_warnings: Vec::new(),
            sidecar_bytes: None,
            names_pending: false,
        }
    }

    /// Read and load the file; board = Board::for_document; disk_bytes = Some(the bytes read).
    pub fn open(path: &Path) -> Result<(Editor, LoadReport), LoadError> {
        let bytes = std::fs::read(path).map_err(|source| LoadError::Io {
            path: path.display().to_string(),
            source,
        })?;
        let (doc, report) = Document::from_vil_bytes(&bytes)?;
        let mut editor = Editor {
            board: Board::for_document(&doc),
            disk: doc.clone(),
            doc,
            disk_bytes: Some(bytes),
            path: Some(path.to_path_buf()),
            base: DiffBase::Disk,
            git_doc: None,
            undo: Vec::new(),
            redo: Vec::new(),
            clipboard: None,
            names: Names::default(),
            disk_names: Names::default(),
            metadata_warnings: Vec::new(),
            sidecar_bytes: None,
            names_pending: false,
        };
        let meta_path = sidecar_path(path);
        match std::fs::read(&meta_path) {
            Ok(bytes) => {
                match serde_json::from_slice::<Sidecar>(&bytes) {
                    Ok(meta) if meta.version == 1 && meta.uid == editor.doc.uid.to_string() && meta.fingerprint == fingerprint(editor.disk_bytes.as_ref().unwrap()) && meta.names.validate(editor.doc()).is_ok() => {
                        editor.names = meta.names;
                        editor.disk_names = editor.names.clone();
                    }
                    _ => editor.metadata_warnings.push("Names sidecar is invalid, stale, or belongs to another UID; names were not applied".into()),
                }
                editor.sidecar_bytes = Some(bytes);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => editor
                .metadata_warnings
                .push(format!("Cannot read names sidecar: {error}")),
        }
        Ok((editor, report))
    }

    pub fn doc(&self) -> &Document {
        &self.doc
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn board(&self) -> Option<&Board> {
        self.board.as_ref()
    }

    /// doc != disk
    pub fn is_dirty(&self) -> bool {
        self.doc != self.disk || self.names != self.disk_names || self.names_pending
    }

    /// Run f on a clone. On Err: nothing changes. On Ok: if the clone differs
    /// from doc, push doc to undo (drop the oldest beyond UNDO_LIMIT), clear
    /// redo, make the clone current. Return f's warnings.
    pub fn apply<F>(&mut self, f: F) -> OpResult
    where
        F: FnOnce(&mut Document) -> OpResult,
    {
        self.apply_named(|doc, _names| f(doc))
    }

    pub fn apply_named<F>(&mut self, f: F) -> OpResult
    where
        F: FnOnce(&mut Document, &mut Names) -> OpResult,
    {
        let mut next = self.doc.clone();
        let mut names = self.names.clone();
        let warnings = f(&mut next, &mut names)?;
        if next != self.doc || names != self.names {
            self.undo.push((
                std::mem::replace(&mut self.doc, next),
                std::mem::replace(&mut self.names, names),
            ));
            if self.undo.len() > UNDO_LIMIT {
                self.undo.remove(0);
            }
            self.redo.clear();
        }
        Ok(warnings)
    }

    pub fn names(&self) -> &Names {
        &self.names
    }
    pub fn disk_names(&self) -> &Names {
        &self.disk_names
    }
    pub fn disk_doc(&self) -> &Document {
        &self.disk
    }
    pub fn names_pending(&self) -> bool {
        self.names_pending
    }
    pub fn metadata_warnings(&self) -> &[String] {
        &self.metadata_warnings
    }
    pub fn set_board(&mut self, board: Board) -> Result<(), String> {
        if !board.matches(&self.doc) {
            return Err("The definition geometry does not match the layout matrix".into());
        }
        self.board = Some(board);
        Ok(())
    }

    /// Makes the last undo entry current. False when there is nothing to undo.
    pub fn undo(&mut self) -> bool {
        match self.undo.pop() {
            Some(previous) => {
                let current = (
                    std::mem::replace(&mut self.doc, previous.0),
                    std::mem::replace(&mut self.names, previous.1),
                );
                self.redo.push(current);
                true
            }
            None => false,
        }
    }

    /// Makes the last redo entry current. False when there is nothing to redo.
    pub fn redo(&mut self) -> bool {
        match self.redo.pop() {
            Some(next) => {
                let current = (
                    std::mem::replace(&mut self.doc, next.0),
                    std::mem::replace(&mut self.names, next.1),
                );
                self.undo.push(current);
                true
            }
            None => false,
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn clipboard(&self) -> Option<&Clipboard> {
        self.clipboard.as_ref()
    }

    pub fn copy_keys(&mut self, layer: usize, positions: &[Pos]) -> Result<(), OpError> {
        let values = crate::ops::read_keys(&self.doc, layer, positions)?;
        self.clipboard = Some(Clipboard::Keys {
            positions: positions.to_vec(),
            values,
        });
        Ok(())
    }

    pub fn paste_keys(&mut self, layer: usize, selection: &[Pos]) -> OpResult {
        let (positions, values) = match &self.clipboard {
            None => return Err(OpError::EmptyClipboard),
            Some(Clipboard::Layer { .. }) => {
                return Err(OpError::WrongClipboard {
                    held: "a layer".into(),
                    wanted: "keys",
                });
            }
            Some(Clipboard::Features { kind, entries }) => {
                return Err(OpError::WrongClipboard {
                    held: slots::count_phrase(*kind, entries.len()),
                    wanted: "keys",
                });
            }
            Some(Clipboard::Keys { positions, values }) => (positions.clone(), values.clone()),
        };
        let targets = crate::ops::paste_targets(&positions, selection)?;
        self.apply(|doc| crate::ops::write_keys(doc, layer, &targets, &values))
    }

    pub fn copy_layer(&mut self, layer: usize) -> Result<(), OpError> {
        let contents = crate::ops::layer_contents(&self.doc, layer)?;
        self.clipboard = Some(Clipboard::Layer { contents });
        Ok(())
    }

    pub fn paste_layer(&mut self, layer: usize) -> OpResult {
        let contents = match &self.clipboard {
            None => return Err(OpError::EmptyClipboard),
            Some(Clipboard::Keys { .. }) => {
                return Err(OpError::WrongClipboard {
                    held: "keys".into(),
                    wanted: "a layer",
                });
            }
            Some(Clipboard::Features { kind, entries }) => {
                return Err(OpError::WrongClipboard {
                    held: slots::count_phrase(*kind, entries.len()),
                    wanted: "a layer",
                });
            }
            Some(Clipboard::Layer { contents }) => contents.clone(),
        };
        self.apply(|doc| crate::ops::set_layer_contents(doc, layer, &contents))
    }

    /// Copies the slots into the clipboard, in index order. Duplicate indices are ignored.
    pub fn copy_feature_slots(
        &mut self,
        kind: FeatureSlotKind,
        indices: &[usize],
    ) -> Result<(), OpError> {
        let indices = sorted_indices(indices)?;
        let entries = self.feature_entries(kind, &indices)?;
        self.clipboard = Some(Clipboard::Features { kind, entries });
        Ok(())
    }

    /// Copies the slots into the clipboard, then clears them in one undo step. The clipboard
    /// changes only when the clear succeeds.
    pub fn cut_feature_slots(&mut self, kind: FeatureSlotKind, indices: &[usize]) -> OpResult {
        let indices = sorted_indices(indices)?;
        let entries = self.feature_entries(kind, &indices)?;
        let warnings = self.clear_feature_slots(kind, &indices)?;
        self.clipboard = Some(Clipboard::Features { kind, entries });
        Ok(warnings)
    }

    /// Writes the clipboard into the selected slots when the counts match, else into consecutive
    /// slots from the one selected slot. Names move with their slots. One undo step.
    pub fn paste_feature_slots(&mut self, kind: FeatureSlotKind, selection: &[usize]) -> OpResult {
        let (held, entries) = match &self.clipboard {
            None => return Err(OpError::EmptyClipboard),
            Some(Clipboard::Features {
                kind: held,
                entries,
            }) => (*held, entries.clone()),
            Some(Clipboard::Keys { .. }) => {
                return Err(OpError::WrongClipboard {
                    held: "keys".into(),
                    wanted: slots::article_name(kind),
                });
            }
            Some(Clipboard::Layer { .. }) => {
                return Err(OpError::WrongClipboard {
                    held: "a layer".into(),
                    wanted: slots::article_name(kind),
                });
            }
        };
        if held != kind {
            return Err(OpError::WrongClipboard {
                held: slots::count_phrase(held, entries.len()),
                wanted: slots::article_name(kind),
            });
        }
        let count = slots::count(&self.doc, kind);
        let targets = slots::paste_targets(entries.len(), selection, count)?;
        self.apply_named(|doc, names| {
            for (&target, (value, name)) in targets.iter().zip(entries) {
                slots::set(doc, kind, target, value)?;
                slots::set_name(names, kind, target, name);
            }
            Ok(Vec::new())
        })
    }

    /// Copies the slots at `from` onto the slots at `to`, pairwise. All values are read before any
    /// is written, so overlapping lists work. The clipboard is unchanged.
    pub fn copy_feature_slots_to(
        &mut self,
        kind: FeatureSlotKind,
        from: &[usize],
        to: &[usize],
    ) -> OpResult {
        if from.is_empty() {
            return Err(OpError::Invalid("no slot selected".into()));
        }
        if from.len() != to.len() {
            return Err(OpError::SizeMismatch {
                expected: from.len(),
                actual: to.len(),
            });
        }
        let mut distinct = to.to_vec();
        distinct.sort_unstable();
        distinct.dedup();
        if distinct.len() != to.len() {
            return Err(OpError::Invalid(
                "a slot appears twice as a copy target".into(),
            ));
        }
        let entries = self.feature_entries(kind, from)?;
        self.apply_named(|doc, names| {
            for (&target, (value, name)) in to.iter().zip(entries) {
                slots::set(doc, kind, target, value)?;
                slots::set_name(names, kind, target, name);
            }
            Ok(Vec::new())
        })
    }

    /// Empties the slots and removes their names. Keycodes that play a macro or tap dance keep
    /// their number.
    pub fn clear_feature_slots(&mut self, kind: FeatureSlotKind, indices: &[usize]) -> OpResult {
        let indices = sorted_indices(indices)?;
        self.apply_named(|doc, names| {
            for &index in &indices {
                slots::set(doc, kind, index, slots::empty(kind))?;
                slots::set_name(names, kind, index, None);
            }
            Ok(Vec::new())
        })
    }

    /// Turns the Enabled switch of key overrides or alt repeat keys on or off.
    pub fn set_feature_slots_enabled(
        &mut self,
        kind: FeatureSlotKind,
        indices: &[usize],
        on: bool,
    ) -> OpResult {
        let no_switch = || {
            OpError::Invalid(format!(
                "{} has no Enabled switch",
                slots::article_name(kind)
            ))
        };
        if slots::enabled(&slots::empty(kind)).is_none() {
            return Err(no_switch());
        }
        let indices = sorted_indices(indices)?;
        self.apply(|doc| {
            for &index in &indices {
                let value = slots::get(doc, kind, index)?;
                let updated = slots::with_enabled(value, on).ok_or_else(no_switch)?;
                slots::set(doc, kind, index, updated)?;
            }
            Ok(Vec::new())
        })
    }

    fn feature_entries(
        &self,
        kind: FeatureSlotKind,
        indices: &[usize],
    ) -> Result<Vec<(FeatureSlotValue, Option<String>)>, OpError> {
        indices
            .iter()
            .map(|&index| {
                Ok((
                    slots::get(&self.doc, kind, index)?,
                    slots::name_of(&self.names, kind, index),
                ))
            })
            .collect()
    }

    /// Swaps two slots with their names and renumbers every keycode that plays either one.
    pub fn swap_feature_slots(&mut self, kind: FeatureSlotKind, a: usize, b: usize) -> OpResult {
        self.apply_named(|doc, names| {
            slots::swap(doc, kind, a, b)?;
            slots::swap_names(names, kind, a, b);
            Ok(Vec::new())
        })
    }

    pub fn diff_base(&self) -> DiffBase {
        self.base
    }

    pub fn set_diff_base_disk(&mut self) {
        self.base = DiffBase::Disk;
    }

    /// Reads the git HEAD version of the file and makes it the diff base.
    pub fn set_diff_base_git(&mut self) -> Result<(), String> {
        let path = self
            .path
            .clone()
            .ok_or_else(|| "this document has no file path".to_string())?;
        let bytes = crate::git::read_head_version(&path)
            .map_err(|e| format!("cannot read the git HEAD version: {e}"))?;
        let (doc, _report) = Document::from_vil_bytes(&bytes)
            .map_err(|e| format!("the git HEAD version does not load: {e}"))?;
        self.git_doc = Some(doc);
        self.base = DiffBase::GitHead;
        Ok(())
    }

    /// disk, or the git HEAD document when base is GitHead.
    pub fn base_doc(&self) -> &Document {
        match self.base {
            DiffBase::Disk => &self.disk,
            DiffBase::GitHead => self.git_doc.as_ref().unwrap_or(&self.disk),
        }
    }

    /// diff(base_doc(), doc(), board())
    pub fn diff(&self) -> Vec<DiffEntry> {
        crate::diff::diff(self.base_doc(), &self.doc, self.board.as_ref())
    }

    pub fn save(&mut self) -> Result<(), SaveError> {
        let path = self.path.clone().ok_or(SaveError::NoPath)?;
        if let Some(expected) = &self.disk_bytes {
            let current = std::fs::read(&path).map_err(|source| SaveError::Io {
                path: path.display().to_string(),
                source,
            })?;
            if current != *expected {
                return Err(SaveError::ChangedOnDisk(path.display().to_string()));
            }
        }
        self.check_sidecar(&path)?;
        self.write_to(&path)
    }

    pub fn save_as(&mut self, path: &Path) -> Result<(), SaveError> {
        if self.path.as_deref() == Some(path) {
            return self.save();
        }
        // The caller must confirm replacing a Save As destination in the native dialog.
        self.write_to(path)
    }

    fn check_sidecar(&self, path: &Path) -> Result<(), SaveError> {
        let meta = sidecar_path(path);
        let current = match std::fs::read(&meta) {
            Ok(bytes) => Some(bytes),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(source) => {
                return Err(SaveError::Io {
                    path: meta.display().to_string(),
                    source,
                });
            }
        };
        if current != self.sidecar_bytes {
            return Err(SaveError::ChangedOnDisk(meta.display().to_string()));
        }
        Ok(())
    }

    /// Serializes, proves the bytes load back identically, then writes atomically.
    fn write_to(&mut self, path: &Path) -> Result<(), SaveError> {
        self.write_to_with(path, crate::io::write_atomic)
    }
    fn write_to_with(
        &mut self,
        path: &Path,
        mut write: impl FnMut(&Path, &[u8]) -> std::io::Result<()>,
    ) -> Result<(), SaveError> {
        let bytes = self.doc.to_vil_bytes();
        let reloaded = Document::from_vil_bytes(&bytes).map_err(|_| SaveError::SelfCheck)?;
        if reloaded.0 != self.doc {
            return Err(SaveError::SelfCheck);
        }
        let sidecar = serde_json::to_vec_pretty(&Sidecar {
            version: 1,
            uid: self.doc.uid.to_string(),
            fingerprint: fingerprint(&bytes),
            names: self.names.clone(),
        })
        .map_err(|_| SaveError::SelfCheck)?;
        let target_sidecar = sidecar_path(path);
        let expected_sidecar = match std::fs::read(&target_sidecar) {
            Ok(bytes) => Some(bytes),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(source) => {
                return Err(SaveError::Io {
                    path: target_sidecar.display().to_string(),
                    source,
                });
            }
        };
        let needs_sidecar = self.names != Names::default()
            || self.sidecar_bytes.is_some()
            || expected_sidecar.is_some();
        write(path, &bytes).map_err(|source| SaveError::Io {
            path: path.display().to_string(),
            source,
        })?;
        self.disk = self.doc.clone();
        self.disk_bytes = Some(bytes);
        self.path = Some(path.to_path_buf());
        self.sidecar_bytes = expected_sidecar;
        if needs_sidecar {
            self.names_pending = true;
            let meta = sidecar_path(path);
            if let Err(source) = write(&meta, &sidecar) {
                self.metadata_warnings = vec![format!(
                    "Layout saved; names not saved: {source}. Retry Save to write the names."
                )];
                return Err(SaveError::Io {
                    path: format!("layout saved; names not saved at {}", meta.display()),
                    source,
                });
            }
            self.sidecar_bytes = Some(sidecar);
        } else {
            self.sidecar_bytes = None;
        }
        self.disk_names = self.names.clone();
        self.names_pending = false;
        self.metadata_warnings.clear();
        Ok(())
    }
}

/// The distinct indices in ascending order. An empty selection is an error.
fn sorted_indices(indices: &[usize]) -> Result<Vec<usize>, OpError> {
    let mut sorted = indices.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    if sorted.is_empty() {
        return Err(OpError::Invalid("no slot selected".into()));
    }
    Ok(sorted)
}

#[cfg(test)]
mod save_tests {
    use super::*;
    #[test]
    fn partial_sidecar_failure_remains_dirty_and_can_retry() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("layout.vil");
        std::fs::copy(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../tests/fixtures/corne.vil"
            ),
            &path,
        )
        .unwrap();
        let (mut editor, _) = Editor::open(&path).unwrap();
        editor
            .apply_named(|_, names| {
                names.layers.insert(0, "Base".into());
                Ok(Vec::new())
            })
            .unwrap();
        let mut calls = 0;
        let error = editor
            .write_to_with(&path, |target, bytes| {
                calls += 1;
                if calls == 2 {
                    Err(std::io::Error::other("injected sidecar failure"))
                } else {
                    crate::io::write_atomic(target, bytes)
                }
            })
            .unwrap_err();
        assert!(error.to_string().contains("layout saved; names not saved"));
        assert!(editor.is_dirty());
        assert!(editor.names_pending());
        editor.save().unwrap();
        assert!(!editor.is_dirty());
        assert_eq!(
            Editor::open(&path)
                .unwrap()
                .0
                .names()
                .layers
                .get(&0)
                .map(String::as_str),
            Some("Base")
        );
    }
}
