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
    Feature {
        kind: FeatureSlotKind,
        value: FeatureSlotValue,
        name: Option<String>,
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
                    held: "a layer",
                    wanted: "keys",
                });
            }
            Some(Clipboard::Feature { kind, .. }) => {
                return Err(OpError::WrongClipboard {
                    held: slots::article_name(*kind),
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
                    held: "keys",
                    wanted: "a layer",
                });
            }
            Some(Clipboard::Feature { kind, .. }) => {
                return Err(OpError::WrongClipboard {
                    held: slots::article_name(*kind),
                    wanted: "a layer",
                });
            }
            Some(Clipboard::Layer { contents }) => contents.clone(),
        };
        self.apply(|doc| crate::ops::set_layer_contents(doc, layer, &contents))
    }

    pub fn copy_feature_slot(
        &mut self,
        kind: FeatureSlotKind,
        index: usize,
    ) -> Result<(), OpError> {
        let value = slots::get(&self.doc, kind, index)?;
        let name = match kind {
            FeatureSlotKind::Combo => self.names.combos.get(&index),
            FeatureSlotKind::Macro => self.names.macros.get(&index),
            _ => None,
        }
        .cloned();
        self.clipboard = Some(Clipboard::Feature { kind, value, name });
        Ok(())
    }

    pub fn paste_feature_slot(&mut self, kind: FeatureSlotKind, index: usize) -> OpResult {
        let (held, value, name) = match &self.clipboard {
            None => return Err(OpError::EmptyClipboard),
            Some(Clipboard::Feature {
                kind: held,
                value,
                name,
            }) => (*held, value.clone(), name.clone()),
            Some(Clipboard::Keys { .. }) => return Err(wrong_feature_clipboard("keys", kind)),
            Some(Clipboard::Layer { .. }) => return Err(wrong_feature_clipboard("a layer", kind)),
        };
        if held != kind {
            return Err(wrong_feature_clipboard(slots::article_name(held), kind));
        }
        self.apply_named(|doc, names| {
            slots::set(doc, kind, index, value)?;
            slots::set_name(names, kind, index, name);
            Ok(Vec::new())
        })
    }

    pub fn swap_feature_slots(&mut self, kind: FeatureSlotKind, a: usize, b: usize) -> OpResult {
        self.apply_named(|doc, names| {
            slots::swap_values(doc, kind, a, b)?;
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

fn wrong_feature_clipboard(held: &'static str, wanted: FeatureSlotKind) -> OpError {
    OpError::WrongClipboard {
        held,
        wanted: slots::article_name(wanted),
    }
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
