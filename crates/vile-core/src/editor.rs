//! An editing session over one document: undo and redo, a clipboard, the
//! diff base (file on disk or git HEAD) and safe saving.

use std::path::{Path, PathBuf};

use crate::board::Board;
use crate::diff::DiffEntry;
use crate::document::{Document, LoadError, LoadReport, Slot};
use crate::keycode::Keycode;
use crate::ops::{OpError, OpResult, Pos};

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
}

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
    undo: Vec<Document>,
    redo: Vec<Document>,
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
        }
    }

    /// Read and load the file; board = Board::for_document; disk_bytes = Some(the bytes read).
    pub fn open(path: &Path) -> Result<(Editor, LoadReport), LoadError> {
        let bytes = std::fs::read(path).map_err(|source| LoadError::Io {
            path: path.display().to_string(),
            source,
        })?;
        let (doc, report) = Document::from_vil_bytes(&bytes)?;
        let editor = Editor {
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
        };
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
        self.doc != self.disk
    }

    /// Run f on a clone. On Err: nothing changes. On Ok: if the clone differs
    /// from doc, push doc to undo (drop the oldest beyond UNDO_LIMIT), clear
    /// redo, make the clone current. Return f's warnings.
    pub fn apply<F>(&mut self, f: F) -> OpResult
    where
        F: FnOnce(&mut Document) -> OpResult,
    {
        let mut next = self.doc.clone();
        let warnings = f(&mut next)?;
        if next != self.doc {
            let previous = std::mem::replace(&mut self.doc, next);
            self.undo.push(previous);
            if self.undo.len() > UNDO_LIMIT {
                self.undo.remove(0);
            }
            self.redo.clear();
        }
        Ok(warnings)
    }

    /// Makes the last undo entry current. False when there is nothing to undo.
    pub fn undo(&mut self) -> bool {
        match self.undo.pop() {
            Some(previous) => {
                let current = std::mem::replace(&mut self.doc, previous);
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
                let current = std::mem::replace(&mut self.doc, next);
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
            Some(Clipboard::Layer { contents }) => contents.clone(),
        };
        self.apply(|doc| crate::ops::set_layer_contents(doc, layer, &contents))
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
        if let Some(expected) = &self.disk_bytes
            && path.exists()
            && let Ok(current) = std::fs::read(&path)
            && current != *expected
        {
            return Err(SaveError::ChangedOnDisk(path.display().to_string()));
        }
        self.write_to(&path)
    }

    pub fn save_as(&mut self, path: &Path) -> Result<(), SaveError> {
        self.write_to(path)?;
        self.path = Some(path.to_path_buf());
        Ok(())
    }

    /// Serializes, proves the bytes load back identically, then writes atomically.
    fn write_to(&mut self, path: &Path) -> Result<(), SaveError> {
        let bytes = self.doc.to_vil_bytes();
        let reloaded = Document::from_vil_bytes(&bytes).map_err(|_| SaveError::SelfCheck)?;
        if reloaded.0 != self.doc {
            return Err(SaveError::SelfCheck);
        }
        crate::io::write_atomic(path, &bytes).map_err(|source| SaveError::Io {
            path: path.display().to_string(),
            source,
        })?;
        self.disk = self.doc.clone();
        self.disk_bytes = Some(bytes);
        Ok(())
    }
}
