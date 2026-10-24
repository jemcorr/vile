//! vile core: the Vial .vil document model, keycodes, editing operations,
//! validation and semantic diff. This crate never touches HID devices.

pub mod board;
pub mod diff;
pub mod document;
pub mod editor;
pub mod git;
pub mod io;
pub mod keycode;
pub mod metadata;
pub mod ops;
pub mod pyjson;
pub mod validate;
pub use board::{Board, BoardKey};
pub use diff::{DiffEntry, DiffSection};
pub use document::{
    AltRepeatKey, Combo, Document, KeyOverride, LoadError, LoadIssue, LoadReport, Macro,
    MacroAction, Setting, Slot, TapDance, TopKey,
};
pub use editor::{Clipboard, DiffBase, Editor, SaveError};
pub use keycode::{
    Catalog, CatalogContext, HoldBehavior, KeyParts, KeyPartsPatch, Keycode, KeycodeError,
    KeycodeKind, LayerOp, SearchHit,
};
pub use ops::{KeyRef, MirrorMode, OpError, OpResult, Pos};
