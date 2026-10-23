//! Physical key geometry. Geometry is not stored in .vil files; vile bundles
//! one board file per supported keyboard.

use crate::document::{Document, Slot};

/// One physical key: its id, its matrix position and its position in key
/// units (x to the right, y down), as in QMK info.json.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct BoardKey {
    pub id: String,
    pub row: usize,
    pub col: usize,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Board {
    pub id: String,
    pub name: String,
    pub source: String,
    pub rows: usize,
    pub cols: usize,
    pub keys: Vec<BoardKey>,
}

const CORNE_V4_1: &str = include_str!("../boards/corne_v4_1.json");

impl Board {
    /// The bundled Corne v4.1 board. Panics only if the bundled JSON is malformed (tested).
    pub fn corne_v4_1() -> Board {
        serde_json::from_str(CORNE_V4_1).expect("the bundled board file is valid JSON (tested)")
    }

    /// Some(corne_v4_1()) when it matches the document, else None.
    pub fn for_document(doc: &Document) -> Option<Board> {
        let board = Board::corne_v4_1();
        board.matches(doc).then_some(board)
    }

    pub fn key_by_id(&self, id: &str) -> Option<&BoardKey> {
        self.keys.iter().find(|key| key.id == id)
    }

    pub fn key_at(&self, row: usize, col: usize) -> Option<&BoardKey> {
        self.keys
            .iter()
            .find(|key| key.row == row && key.col == col)
    }

    /// Index of the key at (row, col) in `keys`.
    pub fn index_of(&self, row: usize, col: usize) -> Option<usize> {
        self.keys
            .iter()
            .position(|key| key.row == row && key.col == col)
    }

    /// The key whose id has "-left-" and "-right-" exchanged.
    pub fn mirror_of(&self, key: &BoardKey) -> Option<&BoardKey> {
        let target = if key.id.contains("-left-") {
            key.id.replacen("-left-", "-right-", 1)
        } else if key.id.contains("-right-") {
            key.id.replacen("-right-", "-left-", 1)
        } else {
            return None;
        };
        self.key_by_id(&target)
    }

    /// The key id at (row, col), or "r{row}c{col}" when no key is there.
    pub fn position_name(&self, row: usize, col: usize) -> String {
        match self.key_at(row, col) {
            Some(key) => key.id.clone(),
            None => format!("r{row}c{col}"),
        }
    }

    /// True when doc.rows() == rows, doc.cols() == cols, and for every
    /// position: layer 0 holds a key exactly where the board has a key.
    pub fn matches(&self, doc: &Document) -> bool {
        if doc.rows() != self.rows || doc.cols() != self.cols {
            return false;
        }
        (0..self.rows).all(|row| {
            (0..self.cols).all(|col| {
                let board_has_key = self.key_at(row, col).is_some();
                let doc_has_key = matches!(doc.get(0, row, col), Some(Slot::Key(_)));
                board_has_key == doc_has_key
            })
        })
    }
}
