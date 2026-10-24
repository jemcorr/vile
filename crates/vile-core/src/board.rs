//! Physical key geometry. Geometry is not stored in .vil files; vile bundles
//! one board file per supported keyboard.

use crate::document::{Document, Slot};

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct KeyRect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

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
    #[serde(default)]
    pub r: f64,
    #[serde(default)]
    pub rx: f64,
    #[serde(default)]
    pub ry: f64,
    #[serde(default)]
    pub secondary: Option<KeyRect>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Board {
    pub id: String,
    pub name: String,
    pub source: String,
    pub rows: usize,
    pub cols: usize,
    pub keys: Vec<BoardKey>,
    #[serde(default)]
    pub mirror_pairs: Vec<[String; 2]>,
}

impl Board {
    /// Without a definition, a usable matrix board is always available.
    pub fn for_document(doc: &Document) -> Option<Board> {
        Some(Board::matrix(doc))
    }

    /// Every present matrix position is editable even without physical geometry.
    pub fn matrix(doc: &Document) -> Board {
        Board {
            id: "matrix".into(),
            name: "Matrix grid".into(),
            source: "matrix".into(),
            mirror_pairs: Vec::new(),
            rows: doc.rows(),
            cols: doc.cols(),
            keys: (0..doc.rows())
                .flat_map(|row| {
                    (0..doc.cols())
                        .filter(move |&col| matches!(doc.get(0, row, col), Some(Slot::Key(_))))
                        .map(move |col| BoardKey {
                            id: format!("r{row}c{col}"),
                            row,
                            col,
                            x: col as f64,
                            y: row as f64,
                            w: 1.0,
                            h: 1.0,
                            r: 0.0,
                            rx: 0.0,
                            ry: 0.0,
                            secondary: None,
                            name: None,
                        })
                })
                .collect(),
        }
    }

    pub fn key_by_id(&self, id: &str) -> Option<&BoardKey> {
        self.keys
            .iter()
            .find(|key| key.id == id || key.name.as_deref() == Some(id))
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
        if let Some(pair) = self.mirror_pairs.iter().find(|pair| pair.contains(&key.id)) {
            return self.key_by_id(if pair[0] == key.id {
                &pair[1]
            } else {
                &pair[0]
            });
        }
        None
    }

    /// The key id at (row, col), or "r{row}c{col}" when no key is there.
    pub fn position_name(&self, row: usize, col: usize) -> String {
        match self.key_at(row, col) {
            Some(key) => key.name.clone().unwrap_or_else(|| key.id.clone()),
            None => format!("r{row}c{col}"),
        }
    }

    pub fn can_mirror(&self) -> bool {
        !self.mirror_pairs.is_empty()
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
