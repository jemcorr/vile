//! vile core: the Vial .vil document model, keycodes, editing operations,
//! validation and semantic diff. This crate never touches HID devices.

pub mod board;
pub mod diff;
pub mod document;
pub mod editor;
pub mod git;
pub mod io;
pub mod keycode;
pub mod ops;
pub mod pyjson;
pub mod validate;
