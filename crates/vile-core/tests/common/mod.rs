#![allow(dead_code)]

use std::path::PathBuf;

use vile_core::Document;

pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

pub fn fixture_bytes(name: &str) -> Vec<u8> {
    std::fs::read(fixture_path(name)).unwrap_or_else(|e| panic!("cannot read fixture {name}: {e}"))
}

pub fn fixture_text(name: &str) -> String {
    String::from_utf8(fixture_bytes(name)).expect("fixture is UTF-8")
}

pub fn load(name: &str) -> Document {
    let (doc, _report) = Document::from_vil_bytes(&fixture_bytes(name))
        .unwrap_or_else(|e| panic!("fixture {name} does not load: {e}"));
    doc
}

/// Wire name of the keycode at (layer, row, col), or "-1" for an absent position.
pub fn wire_at(doc: &Document, layer: usize, row: usize, col: usize) -> String {
    match doc.get(layer, row, col).expect("position exists") {
        vile_core::Slot::Absent => "-1".to_string(),
        vile_core::Slot::Key(kc) => doc.catalog().wire_name(kc),
    }
}

/// Parse a keycode with the document's catalog; panics on error.
pub fn kc(doc: &Document, s: &str) -> vile_core::Keycode {
    doc.catalog()
        .parse(s)
        .unwrap_or_else(|e| panic!("cannot parse {s}: {e}"))
}

pub fn board() -> vile_core::Board {
    serde_json::from_slice(&fixture_bytes("corne-geometry.json")).unwrap()
}
