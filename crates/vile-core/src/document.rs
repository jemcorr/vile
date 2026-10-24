//! The .vil document: a typed model of Vial's saved layout file, a strict
//! loader, and a writer that reproduces Vial's bytes exactly.

use std::path::Path;

use crate::keycode::{Catalog, CatalogContext, Keycode};
use crate::pyjson::{write_key, write_list, write_py_string};
use serde_json::{Map, Value};

/// One matrix position: -1 in the file (no key there) or a keycode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Slot {
    Absent,
    Key(Keycode),
}

/// Top-level keys of a .vil file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TopKey {
    Version,
    Uid,
    Layout,
    EncoderLayout,
    LayoutOptions,
    Macro,
    VialProtocol,
    ViaProtocol,
    TapDance,
    Combo,
    KeyOverride,
    AltRepeatKey,
    Settings,
}

impl TopKey {
    /// The order in which Vial writes the keys.
    pub const CANONICAL: [TopKey; 13] = [
        TopKey::Version,
        TopKey::Uid,
        TopKey::Layout,
        TopKey::EncoderLayout,
        TopKey::LayoutOptions,
        TopKey::Macro,
        TopKey::VialProtocol,
        TopKey::ViaProtocol,
        TopKey::TapDance,
        TopKey::Combo,
        TopKey::KeyOverride,
        TopKey::AltRepeatKey,
        TopKey::Settings,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            TopKey::Version => "version",
            TopKey::Uid => "uid",
            TopKey::Layout => "layout",
            TopKey::EncoderLayout => "encoder_layout",
            TopKey::LayoutOptions => "layout_options",
            TopKey::Macro => "macro",
            TopKey::VialProtocol => "vial_protocol",
            TopKey::ViaProtocol => "via_protocol",
            TopKey::TapDance => "tap_dance",
            TopKey::Combo => "combo",
            TopKey::KeyOverride => "key_override",
            TopKey::AltRepeatKey => "alt_repeat_key",
            TopKey::Settings => "settings",
        }
    }

    pub fn from_name(s: &str) -> Option<TopKey> {
        TopKey::CANONICAL.into_iter().find(|k| k.as_str() == s)
    }
}

/// One macro action, written in the file as ["tap", kc...], ["down", kc...],
/// ["up", kc...], ["text", "..."] or ["delay", ms].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MacroAction {
    Tap(Vec<Keycode>),
    Down(Vec<Keycode>),
    Up(Vec<Keycode>),
    Text(String),
    Delay(u32),
}

/// One macro slot. An empty slot has no actions and is written as [].
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Macro {
    pub actions: Vec<MacroAction>,
}

/// [on_tap, on_hold, on_double_tap, on_tap_hold, tapping_term_ms]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TapDance {
    pub on_tap: Keycode,
    pub on_hold: Keycode,
    pub on_double_tap: Keycode,
    pub on_tap_hold: Keycode,
    pub tapping_term: u16,
}

/// [k1, k2, k3, k4, output]; unused inputs are KC_NO.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Combo {
    pub keys: [Keycode; 4],
    pub output: Keycode,
}

/// Object with keys trigger, replacement, layers, trigger_mods,
/// negative_mod_mask, suppressed_mods, options (written in this order).
/// `layers` is a bit mask: bit n set means active on layer n.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyOverride {
    pub trigger: Keycode,
    pub replacement: Keycode,
    pub layers: u16,
    pub trigger_mods: u8,
    pub negative_mod_mask: u8,
    pub suppressed_mods: u8,
    pub options: u8,
}

/// Object with keys keycode, alt_keycode, allowed_mods, options.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AltRepeatKey {
    pub keycode: Keycode,
    pub alt_keycode: Keycode,
    pub allowed_mods: u8,
    pub options: u8,
}

/// One entry of the "settings" object: "qsid": value, kept in file order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Setting {
    pub qsid: String,
    pub value: u32,
}

/// A loaded .vil file. Optional sections are None when the file lacks the key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    /// Top-level keys in file order; the writer uses this order.
    pub key_order: Vec<TopKey>,
    pub version: u32,
    pub uid: u64,
    /// [layer][row][col]
    pub layout: Vec<Vec<Vec<Slot>>>,
    /// [layer][encoder] = [clockwise, counter-clockwise]
    pub encoder_layout: Option<Vec<Vec<[Slot; 2]>>>,
    pub layout_options: i64,
    /// The "macro" key. Always at least one slot (see LoadError for "macro": []).
    pub macros: Vec<Macro>,
    pub vial_protocol: u32,
    pub via_protocol: u32,
    pub tap_dance: Option<Vec<TapDance>>,
    pub combo: Option<Vec<Combo>>,
    pub key_override: Option<Vec<KeyOverride>>,
    pub alt_repeat_key: Option<Vec<AltRepeatKey>>,
    pub settings: Option<Vec<Setting>>,
}

/// A problem found while loading, with a JSON path such as "layout[0][3][4]".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadIssue {
    pub path: String,
    pub message: String,
}

/// Non-fatal findings of a successful load.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LoadReport {
    pub warnings: Vec<LoadIssue>,
    /// True when saving the loaded document reproduces the input bytes exactly.
    pub canonical: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("cannot read {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("not valid JSON: {0}")]
    Json(String),
    #[error("{}", format_issues(.0))]
    Invalid(Vec<LoadIssue>),
}

/// "N problem(s) in the file:" followed by one indented "path: message" line per issue.
pub fn format_issues(issues: &[LoadIssue]) -> String {
    let mut s = format!("{} problem(s) in the file:", issues.len());
    for issue in issues {
        s.push_str("\n  ");
        s.push_str(&issue.path);
        s.push_str(": ");
        s.push_str(&issue.message);
    }
    s
}

fn all_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

impl Document {
    /// Loads and validates the bytes of a .vil file. Every problem is reported
    /// at once; nothing is dropped.
    pub fn from_vil_bytes(bytes: &[u8]) -> Result<(Document, LoadReport), LoadError> {
        let value: Value =
            serde_json::from_slice(bytes).map_err(|e| LoadError::Json(e.to_string()))?;
        let Some(object) = value.as_object() else {
            return Err(LoadError::Invalid(vec![LoadIssue {
                path: "<root>".to_string(),
                message: "the file must be a JSON object".to_string(),
            }]));
        };

        let mut loader = Loader {
            catalog: Catalog::new(CatalogContext {
                layers: u8::try_from(
                    object
                        .get("layout")
                        .and_then(Value::as_array)
                        .map_or(0, Vec::len)
                        .min(255),
                )
                .unwrap_or(u8::MAX),
                macros: u16::try_from(
                    object
                        .get("macro")
                        .and_then(Value::as_array)
                        .map_or(0, Vec::len),
                )
                .unwrap_or(u16::MAX),
                tap_dances: u16::try_from(
                    object
                        .get("tap_dance")
                        .and_then(Value::as_array)
                        .map_or(0, Vec::len),
                )
                .unwrap_or(u16::MAX),
            }),
            issues: Vec::new(),
            seen: Vec::new(),
        };

        let mut key_order = Vec::new();
        for key in object.keys() {
            match TopKey::from_name(key) {
                Some(known) => key_order.push(known),
                None => loader.issues.push(LoadIssue {
                    path: key.clone(),
                    message: "unknown top-level key; vile refuses to load the file so that nothing is dropped"
                        .to_string(),
                }),
            }
        }
        for required in [
            TopKey::Version,
            TopKey::Uid,
            TopKey::Layout,
            TopKey::LayoutOptions,
            TopKey::Macro,
            TopKey::VialProtocol,
            TopKey::ViaProtocol,
        ] {
            if !object.contains_key(required.as_str()) {
                loader.issues.push(LoadIssue {
                    path: required.as_str().to_string(),
                    message: "missing required key".to_string(),
                });
            }
        }

        let field = |key: TopKey| object.get(key.as_str());
        let version = match field(TopKey::Version) {
            Some(v) => loader
                .uint(
                    v,
                    "version",
                    u64::from(u32::MAX),
                    "must be a non-negative integer",
                )
                .unwrap_or(0) as u32,
            None => 0,
        };
        let uid = match field(TopKey::Uid) {
            Some(v) => loader
                .uint(
                    v,
                    "uid",
                    u64::MAX,
                    "must be a non-negative integer below 2^64",
                )
                .unwrap_or(0),
            None => 0,
        };
        let layout = match field(TopKey::Layout) {
            Some(v) => loader.layout(v),
            None => Vec::new(),
        };
        let layer_count = layout.len();
        let encoder_layout = field(TopKey::EncoderLayout).map(|v| loader.encoders(v, layer_count));
        let layout_options = match field(TopKey::LayoutOptions) {
            Some(v) => match v.as_i64() {
                Some(n) => n,
                None => {
                    loader.issue("layout_options", "must be an integer");
                    0
                }
            },
            None => 0,
        };
        let macros = match field(TopKey::Macro) {
            Some(v) => loader.macros(v),
            None => Vec::new(),
        };
        let vial_protocol = match field(TopKey::VialProtocol) {
            Some(v) => loader
                .uint(
                    v,
                    "vial_protocol",
                    u64::from(u32::MAX),
                    "must be a non-negative integer",
                )
                .unwrap_or(0) as u32,
            None => 0,
        };
        let via_protocol = match field(TopKey::ViaProtocol) {
            Some(v) => loader
                .uint(
                    v,
                    "via_protocol",
                    u64::from(u32::MAX),
                    "must be a non-negative integer",
                )
                .unwrap_or(0) as u32,
            None => 0,
        };
        let tap_dance = field(TopKey::TapDance).map(|v| loader.tap_dances(v));
        let combo = field(TopKey::Combo).map(|v| loader.combos(v));
        let key_override = field(TopKey::KeyOverride).map(|v| loader.key_overrides(v));
        let alt_repeat_key = field(TopKey::AltRepeatKey).map(|v| loader.alt_repeat_keys(v));
        let settings = field(TopKey::Settings).map(|v| loader.settings(v));

        if !loader.issues.is_empty() {
            return Err(LoadError::Invalid(loader.issues));
        }

        let doc = Document {
            key_order,
            version,
            uid,
            layout,
            encoder_layout,
            layout_options,
            macros,
            vial_protocol,
            via_protocol,
            tap_dance,
            combo,
            key_override,
            alt_repeat_key,
            settings,
        };
        let catalog = loader.catalog;
        let mut warnings: Vec<LoadIssue> = loader
            .seen
            .iter()
            .filter_map(|(path, original, code)| {
                let wire = catalog.wire_name(*code);
                (wire != *original).then(|| LoadIssue {
                    path: path.clone(),
                    message: format!("\"{original}\" will be saved as \"{wire}\""),
                })
            })
            .collect();
        warnings.extend(crate::validate::validate(&doc));
        let canonical = doc.to_vil_bytes() == bytes;
        Ok((
            doc,
            LoadReport {
                warnings,
                canonical,
            },
        ))
    }

    /// Reads and loads a .vil file.
    pub fn load(path: &Path) -> Result<(Document, LoadReport), LoadError> {
        let bytes = std::fs::read(path).map_err(|source| LoadError::Io {
            path: path.display().to_string(),
            source,
        })?;
        Document::from_vil_bytes(&bytes)
    }

    /// The bytes Vial writes for this document (no trailing newline).
    pub fn to_vil_bytes(&self) -> Vec<u8> {
        let catalog = self.catalog();
        let mut out = String::from("{");
        let mut first = true;
        for key in &self.key_order {
            let mut body = String::new();
            if !self.write_section(*key, &catalog, &mut body) {
                continue;
            }
            if !first {
                out.push_str(", ");
            }
            first = false;
            write_key(&mut out, key.as_str());
            out.push_str(&body);
        }
        out.push('}');
        out.into_bytes()
    }

    pub fn layer_count(&self) -> usize {
        self.layout.len()
    }

    pub fn rows(&self) -> usize {
        self.layout.first().map_or(0, Vec::len)
    }

    pub fn cols(&self) -> usize {
        self.layout
            .first()
            .and_then(|layer| layer.first())
            .map_or(0, Vec::len)
    }

    /// The catalog for this document: names depend on its layers, macros and tap dances.
    pub fn catalog(&self) -> Catalog {
        Catalog::new(CatalogContext {
            layers: u8::try_from(self.layer_count().min(255)).unwrap_or(u8::MAX),
            macros: u16::try_from(self.macros.len()).unwrap_or(u16::MAX),
            tap_dances: self
                .tap_dance
                .as_ref()
                .map_or(0, |list| u16::try_from(list.len()).unwrap_or(u16::MAX)),
        })
    }

    /// The slot at (layer, row, col), or None when an index is out of range.
    pub fn get(&self, layer: usize, row: usize, col: usize) -> Option<Slot> {
        self.layout.get(layer)?.get(row)?.get(col).copied()
    }

    /// Writes the value of one top-level key. Returns false when the optional
    /// section is absent, so the key is skipped.
    fn write_section(&self, key: TopKey, catalog: &Catalog, out: &mut String) -> bool {
        match key {
            TopKey::Version => out.push_str(&self.version.to_string()),
            TopKey::Uid => out.push_str(&self.uid.to_string()),
            TopKey::Layout => write_layers(out, catalog, &self.layout),
            TopKey::EncoderLayout => match &self.encoder_layout {
                Some(encoders) => write_list(out, encoders, |out, layer| {
                    write_list(out, layer, |out, pair| {
                        write_list(out, pair, |out, slot| write_slot(out, catalog, *slot));
                    });
                }),
                None => return false,
            },
            TopKey::LayoutOptions => out.push_str(&self.layout_options.to_string()),
            TopKey::Macro => write_list(out, &self.macros, |out, slot| {
                write_list(out, &slot.actions, |out, action| {
                    write_action(out, catalog, action);
                });
            }),
            TopKey::VialProtocol => out.push_str(&self.vial_protocol.to_string()),
            TopKey::ViaProtocol => out.push_str(&self.via_protocol.to_string()),
            TopKey::TapDance => match &self.tap_dance {
                Some(list) => write_list(out, list, |out, td| {
                    out.push('[');
                    write_kc(out, catalog, td.on_tap);
                    out.push_str(", ");
                    write_kc(out, catalog, td.on_hold);
                    out.push_str(", ");
                    write_kc(out, catalog, td.on_double_tap);
                    out.push_str(", ");
                    write_kc(out, catalog, td.on_tap_hold);
                    out.push_str(", ");
                    out.push_str(&td.tapping_term.to_string());
                    out.push(']');
                }),
                None => return false,
            },
            TopKey::Combo => match &self.combo {
                Some(list) => write_list(out, list, |out, combo| {
                    out.push('[');
                    for kc in combo.keys {
                        write_kc(out, catalog, kc);
                        out.push_str(", ");
                    }
                    write_kc(out, catalog, combo.output);
                    out.push(']');
                }),
                None => return false,
            },
            TopKey::KeyOverride => match &self.key_override {
                Some(list) => write_list(out, list, |out, ko| {
                    out.push('{');
                    write_key(out, "trigger");
                    write_kc(out, catalog, ko.trigger);
                    out.push_str(", ");
                    write_key(out, "replacement");
                    write_kc(out, catalog, ko.replacement);
                    out.push_str(", ");
                    write_key(out, "layers");
                    out.push_str(&ko.layers.to_string());
                    out.push_str(", ");
                    write_key(out, "trigger_mods");
                    out.push_str(&ko.trigger_mods.to_string());
                    out.push_str(", ");
                    write_key(out, "negative_mod_mask");
                    out.push_str(&ko.negative_mod_mask.to_string());
                    out.push_str(", ");
                    write_key(out, "suppressed_mods");
                    out.push_str(&ko.suppressed_mods.to_string());
                    out.push_str(", ");
                    write_key(out, "options");
                    out.push_str(&ko.options.to_string());
                    out.push('}');
                }),
                None => return false,
            },
            TopKey::AltRepeatKey => match &self.alt_repeat_key {
                Some(list) => write_list(out, list, |out, ar| {
                    out.push('{');
                    write_key(out, "keycode");
                    write_kc(out, catalog, ar.keycode);
                    out.push_str(", ");
                    write_key(out, "alt_keycode");
                    write_kc(out, catalog, ar.alt_keycode);
                    out.push_str(", ");
                    write_key(out, "allowed_mods");
                    out.push_str(&ar.allowed_mods.to_string());
                    out.push_str(", ");
                    write_key(out, "options");
                    out.push_str(&ar.options.to_string());
                    out.push('}');
                }),
                None => return false,
            },
            TopKey::Settings => match &self.settings {
                Some(list) => {
                    out.push('{');
                    for (i, setting) in list.iter().enumerate() {
                        if i > 0 {
                            out.push_str(", ");
                        }
                        write_key(out, &setting.qsid);
                        out.push_str(&setting.value.to_string());
                    }
                    out.push('}');
                }
                None => return false,
            },
        }
        true
    }
}

fn write_kc(out: &mut String, catalog: &Catalog, kc: Keycode) {
    write_py_string(out, &catalog.wire_name(kc));
}

fn write_slot(out: &mut String, catalog: &Catalog, slot: Slot) {
    match slot {
        Slot::Absent => out.push_str("-1"),
        Slot::Key(kc) => write_kc(out, catalog, kc),
    }
}

fn write_layers(out: &mut String, catalog: &Catalog, layout: &[Vec<Vec<Slot>>]) {
    write_list(out, layout, |out, layer| {
        write_list(out, layer, |out, row| {
            write_list(out, row, |out, slot| write_slot(out, catalog, *slot));
        });
    });
}

fn write_action(out: &mut String, catalog: &Catalog, action: &MacroAction) {
    match action {
        MacroAction::Tap(keys) => write_key_action(out, catalog, "tap", keys),
        MacroAction::Down(keys) => write_key_action(out, catalog, "down", keys),
        MacroAction::Up(keys) => write_key_action(out, catalog, "up", keys),
        MacroAction::Text(text) => {
            out.push('[');
            write_py_string(out, "text");
            out.push_str(", ");
            write_py_string(out, text);
            out.push(']');
        }
        MacroAction::Delay(ms) => {
            out.push('[');
            write_py_string(out, "delay");
            out.push_str(", ");
            out.push_str(&ms.to_string());
            out.push(']');
        }
    }
}

fn write_key_action(out: &mut String, catalog: &Catalog, name: &str, keys: &[Keycode]) {
    out.push('[');
    write_py_string(out, name);
    for kc in keys {
        out.push_str(", ");
        write_kc(out, catalog, *kc);
    }
    out.push(']');
}

/// Collects load problems while converting the JSON value to the document.
struct Loader {
    catalog: Catalog,
    issues: Vec<LoadIssue>,
    /// Every keycode that parsed: (path, original spelling, code), in file order.
    seen: Vec<(String, String, Keycode)>,
}

impl Loader {
    fn issue(&mut self, path: &str, message: impl Into<String>) {
        self.issues.push(LoadIssue {
            path: path.to_string(),
            message: message.into(),
        });
    }

    fn uint(&mut self, v: &Value, path: &str, max: u64, message: &str) -> Option<u64> {
        match v.as_u64().filter(|n| *n <= max) {
            Some(n) => Some(n),
            None => {
                self.issue(path, message);
                None
            }
        }
    }

    fn keycode(&mut self, v: &Value, path: &str) -> Option<Keycode> {
        let Some(text) = v.as_str() else {
            self.issue(path, "must be a keycode string");
            return None;
        };
        match self.catalog.parse(text) {
            Ok(code) => {
                self.seen.push((path.to_string(), text.to_string(), code));
                Some(code)
            }
            Err(e) => {
                self.issue(path, e.to_string());
                None
            }
        }
    }

    /// A keycode string or -1. None when the value is neither (already reported).
    fn cell(&mut self, v: &Value, path: &str) -> Option<Slot> {
        if v.as_i64() == Some(-1) {
            return Some(Slot::Absent);
        }
        if v.is_string() {
            return self.keycode(v, path).map(Slot::Key);
        }
        self.issue(path, "must be a keycode string or -1");
        None
    }

    fn layout(&mut self, v: &Value) -> Vec<Vec<Vec<Slot>>> {
        let Some(layers) = v.as_array().filter(|layers| !layers.is_empty()) else {
            self.issue("layout", "must be a non-empty list of layers");
            return Vec::new();
        };
        let mut cells: Vec<Vec<Vec<Option<Slot>>>> = Vec::new();
        for (l, layer) in layers.iter().enumerate() {
            let mut rows_out = Vec::new();
            if let Some(rows) = layer.as_array() {
                for (r, row) in rows.iter().enumerate() {
                    let mut cols_out = Vec::new();
                    if let Some(cols) = row.as_array() {
                        for (c, cell) in cols.iter().enumerate() {
                            cols_out.push(self.cell(cell, &format!("layout[{l}][{r}][{c}]")));
                        }
                    } else {
                        self.issue(&format!("layout[{l}][{r}]"), "must be a list of cells");
                    }
                    rows_out.push(cols_out);
                }
            } else {
                self.issue(&format!("layout[{l}]"), "must be a list of rows");
            }
            cells.push(rows_out);
        }

        let rows = cells[0].len();
        let cols = cells[0].first().map_or(0, Vec::len);
        let same_shape = cells
            .iter()
            .all(|layer| layer.len() == rows && layer.iter().all(|row| row.len() == cols));
        if !same_shape {
            self.issue("layout", "all layers must have the same rows and columns");
        }

        for l in 1..cells.len() {
            for (r, row) in cells[l].iter().enumerate() {
                for (c, cell) in row.iter().enumerate() {
                    let reference = cells[0]
                        .get(r)
                        .and_then(|row0| row0.get(c))
                        .copied()
                        .flatten();
                    if let (Some(here), Some(reference)) = (*cell, reference)
                        && matches!(here, Slot::Absent) != matches!(reference, Slot::Absent)
                    {
                        self.issue(
                            &format!("layout[{l}][{r}][{c}]"),
                            "-1 here does not match layer 0",
                        );
                    }
                }
            }
        }

        cells
            .into_iter()
            .map(|layer| {
                layer
                    .into_iter()
                    .map(|row| {
                        row.into_iter()
                            .map(|cell| cell.unwrap_or(Slot::Absent))
                            .collect()
                    })
                    .collect()
            })
            .collect()
    }

    fn encoders(&mut self, v: &Value, layer_count: usize) -> Vec<Vec<[Slot; 2]>> {
        let Some(layers) = v.as_array() else {
            self.issue("encoder_layout", "must be a list of layers");
            return Vec::new();
        };
        if layers.len() != layer_count {
            self.issue("encoder_layout", "must have one entry per layer");
        }
        let mut out = Vec::new();
        let mut expected: Option<usize> = None;
        for (l, layer) in layers.iter().enumerate() {
            let Some(encoders) = layer.as_array() else {
                self.issue(
                    &format!("encoder_layout[{l}]"),
                    "must be a list of encoders",
                );
                out.push(Vec::new());
                continue;
            };
            match expected {
                None => expected = Some(encoders.len()),
                Some(count) if count != encoders.len() => {
                    self.issue(
                        "encoder_layout",
                        "every layer must have the same number of encoders",
                    );
                }
                Some(_) => {}
            }
            let mut pairs = Vec::new();
            for (e, encoder) in encoders.iter().enumerate() {
                let path = format!("encoder_layout[{l}][{e}]");
                let Some(items) = encoder.as_array().filter(|items| items.len() == 2) else {
                    self.issue(&path, "must be a list of two keycodes or -1");
                    continue;
                };
                let clockwise = self.cell(&items[0], &format!("{path}[0]"));
                let counter = self.cell(&items[1], &format!("{path}[1]"));
                pairs.push([
                    clockwise.unwrap_or(Slot::Absent),
                    counter.unwrap_or(Slot::Absent),
                ]);
            }
            out.push(pairs);
        }
        out
    }

    fn macros(&mut self, v: &Value) -> Vec<Macro> {
        let Some(slots) = v.as_array() else {
            self.issue("macro", "must be a list of macro slots");
            return Vec::new();
        };
        if slots.is_empty() {
            self.issue(
                "macro",
                "is an empty list; loading this file in Vial would erase every macro",
            );
            return Vec::new();
        }
        let mut out = Vec::new();
        for (m, slot) in slots.iter().enumerate() {
            let mut actions = Vec::new();
            if let Some(list) = slot.as_array() {
                for (a, action) in list.iter().enumerate() {
                    if let Some(parsed) = self.macro_action(action, &format!("macro[{m}][{a}]")) {
                        actions.push(parsed);
                    }
                }
            } else {
                self.issue(&format!("macro[{m}]"), "must be a list of actions");
            }
            out.push(Macro { actions });
        }
        out
    }

    fn macro_action(&mut self, v: &Value, path: &str) -> Option<MacroAction> {
        let Some(items) = v.as_array().filter(|items| !items.is_empty()) else {
            self.issue(path, "must be a list whose first item is an action name");
            return None;
        };
        let Some(name) = items[0].as_str() else {
            self.issue(path, "must be a list whose first item is an action name");
            return None;
        };
        match name {
            "tap" | "down" | "up" => {
                let mut keys = Vec::new();
                let mut ok = true;
                for (k, item) in items.iter().enumerate().skip(1) {
                    match self.keycode(item, &format!("{path}[{k}]")) {
                        Some(code) => keys.push(code),
                        None => ok = false,
                    }
                }
                if !ok {
                    return None;
                }
                Some(match name {
                    "tap" => MacroAction::Tap(keys),
                    "down" => MacroAction::Down(keys),
                    _ => MacroAction::Up(keys),
                })
            }
            "text" => match items.as_slice() {
                [_, Value::String(text)] => {
                    if text.chars().any(|c| (c as u32) <= 7) {
                        self.issue(
                            &format!("{path}[1]"),
                            "text must not contain characters U+0000 to U+0007",
                        );
                        None
                    } else {
                        Some(MacroAction::Text(text.clone()))
                    }
                }
                _ => {
                    self.issue(path, "text takes exactly one string");
                    None
                }
            },
            "delay" => match items.as_slice() {
                [_, value] => match value.as_u64().filter(|ms| *ms <= 65024) {
                    Some(ms) => Some(MacroAction::Delay(ms as u32)),
                    None => {
                        self.issue(
                            &format!("{path}[1]"),
                            "delay must be an integer from 0 to 65024",
                        );
                        None
                    }
                },
                _ => {
                    self.issue(path, "delay takes exactly one integer");
                    None
                }
            },
            other => {
                self.issue(path, format!("unknown macro action \"{other}\""));
                None
            }
        }
    }

    fn tap_dances(&mut self, v: &Value) -> Vec<TapDance> {
        let Some(items) = v.as_array() else {
            self.issue("tap_dance", "must be a list of tap dances");
            return Vec::new();
        };
        let mut out = Vec::new();
        for (i, item) in items.iter().enumerate() {
            let path = format!("tap_dance[{i}]");
            let Some(fields) = item.as_array().filter(|fields| fields.len() == 5) else {
                self.issue(&path, "must be a list of five values");
                continue;
            };
            let mut codes = [Keycode::NO; 4];
            let mut ok = true;
            for (f, field) in fields.iter().take(4).enumerate() {
                match self.keycode(field, &format!("{path}[{f}]")) {
                    Some(code) => codes[f] = code,
                    None => ok = false,
                }
            }
            let term = self.uint(
                &fields[4],
                &format!("{path}[4]"),
                65535,
                "must be an integer from 0 to 65535",
            );
            if let (true, Some(term)) = (ok, term) {
                out.push(TapDance {
                    on_tap: codes[0],
                    on_hold: codes[1],
                    on_double_tap: codes[2],
                    on_tap_hold: codes[3],
                    tapping_term: term as u16,
                });
            }
        }
        out
    }

    fn combos(&mut self, v: &Value) -> Vec<Combo> {
        let Some(items) = v.as_array() else {
            self.issue("combo", "must be a list of combos");
            return Vec::new();
        };
        let mut out = Vec::new();
        for (i, item) in items.iter().enumerate() {
            let path = format!("combo[{i}]");
            let Some(fields) = item.as_array().filter(|fields| fields.len() == 5) else {
                self.issue(&path, "must be a list of five keycodes");
                continue;
            };
            let mut codes = [Keycode::NO; 5];
            let mut ok = true;
            for (f, field) in fields.iter().enumerate() {
                match self.keycode(field, &format!("{path}[{f}]")) {
                    Some(code) => codes[f] = code,
                    None => ok = false,
                }
            }
            if ok {
                out.push(Combo {
                    keys: [codes[0], codes[1], codes[2], codes[3]],
                    output: codes[4],
                });
            }
        }
        out
    }

    fn keycode_field(
        &mut self,
        obj: &Map<String, Value>,
        key: &str,
        path: &str,
    ) -> Option<Keycode> {
        self.keycode(&obj[key], &format!("{path}.{key}"))
    }

    fn uint_field(
        &mut self,
        obj: &Map<String, Value>,
        key: &str,
        path: &str,
        max: u64,
    ) -> Option<u64> {
        let message = format!("must be an integer from 0 to {max}");
        self.uint(&obj[key], &format!("{path}.{key}"), max, &message)
    }

    fn key_overrides(&mut self, v: &Value) -> Vec<KeyOverride> {
        const KEYS: [&str; 7] = [
            "trigger",
            "replacement",
            "layers",
            "trigger_mods",
            "negative_mod_mask",
            "suppressed_mods",
            "options",
        ];
        let Some(items) = v.as_array() else {
            self.issue("key_override", "must be a list of key overrides");
            return Vec::new();
        };
        let mut out = Vec::new();
        for (i, item) in items.iter().enumerate() {
            let path = format!("key_override[{i}]");
            let Some(obj) = item
                .as_object()
                .filter(|obj| obj.len() == KEYS.len() && KEYS.iter().all(|k| obj.contains_key(*k)))
            else {
                self.issue(
                    &path,
                    format!(
                        "must be an object with exactly the keys {}",
                        KEYS.join(", ")
                    ),
                );
                continue;
            };
            let trigger = self.keycode_field(obj, "trigger", &path);
            let replacement = self.keycode_field(obj, "replacement", &path);
            let layers = self.uint_field(obj, "layers", &path, 65535);
            let trigger_mods = self.uint_field(obj, "trigger_mods", &path, 255);
            let negative_mod_mask = self.uint_field(obj, "negative_mod_mask", &path, 255);
            let suppressed_mods = self.uint_field(obj, "suppressed_mods", &path, 255);
            let options = self.uint_field(obj, "options", &path, 255);
            if let (
                Some(trigger),
                Some(replacement),
                Some(layers),
                Some(trigger_mods),
                Some(negative_mod_mask),
                Some(suppressed_mods),
                Some(options),
            ) = (
                trigger,
                replacement,
                layers,
                trigger_mods,
                negative_mod_mask,
                suppressed_mods,
                options,
            ) {
                out.push(KeyOverride {
                    trigger,
                    replacement,
                    layers: layers as u16,
                    trigger_mods: trigger_mods as u8,
                    negative_mod_mask: negative_mod_mask as u8,
                    suppressed_mods: suppressed_mods as u8,
                    options: options as u8,
                });
            }
        }
        out
    }

    fn alt_repeat_keys(&mut self, v: &Value) -> Vec<AltRepeatKey> {
        const KEYS: [&str; 4] = ["keycode", "alt_keycode", "allowed_mods", "options"];
        let Some(items) = v.as_array() else {
            self.issue("alt_repeat_key", "must be a list of alt repeat keys");
            return Vec::new();
        };
        let mut out = Vec::new();
        for (i, item) in items.iter().enumerate() {
            let path = format!("alt_repeat_key[{i}]");
            let Some(obj) = item
                .as_object()
                .filter(|obj| obj.len() == KEYS.len() && KEYS.iter().all(|k| obj.contains_key(*k)))
            else {
                self.issue(
                    &path,
                    format!(
                        "must be an object with exactly the keys {}",
                        KEYS.join(", ")
                    ),
                );
                continue;
            };
            let keycode = self.keycode_field(obj, "keycode", &path);
            let alt_keycode = self.keycode_field(obj, "alt_keycode", &path);
            let allowed_mods = self.uint_field(obj, "allowed_mods", &path, 255);
            let options = self.uint_field(obj, "options", &path, 255);
            if let (Some(keycode), Some(alt_keycode), Some(allowed_mods), Some(options)) =
                (keycode, alt_keycode, allowed_mods, options)
            {
                out.push(AltRepeatKey {
                    keycode,
                    alt_keycode,
                    allowed_mods: allowed_mods as u8,
                    options: options as u8,
                });
            }
        }
        out
    }

    fn settings(&mut self, v: &Value) -> Vec<Setting> {
        let Some(object) = v.as_object() else {
            self.issue("settings", "must be an object");
            return Vec::new();
        };
        let mut out = Vec::new();
        for (qsid, value) in object {
            let path = format!("settings.{qsid}");
            if !all_digits(qsid) {
                self.issue(&path, "must be ASCII digits only");
                continue;
            }
            if let Some(n) = self.uint(
                value,
                &path,
                u64::from(u32::MAX),
                "must be an integer from 0 to 4294967295",
            ) {
                out.push(Setting {
                    qsid: qsid.clone(),
                    value: n as u32,
                });
            }
        }
        out
    }
}

impl Document {
    /// Set a key. Errors: NoSuchLayer, NoKey (out of range or an Absent position).
    pub fn set(
        &mut self,
        layer: usize,
        row: usize,
        col: usize,
        kc: Keycode,
    ) -> Result<(), crate::ops::OpError> {
        let count = self.layer_count();
        let slot = self
            .layout
            .get_mut(layer)
            .ok_or(crate::ops::OpError::NoSuchLayer { layer, count })?
            .get_mut(row)
            .and_then(|cells| cells.get_mut(col))
            .ok_or(crate::ops::OpError::NoKey { row, col })?;
        if *slot == Slot::Absent {
            return Err(crate::ops::OpError::NoKey { row, col });
        }
        *slot = Slot::Key(kc);
        Ok(())
    }

    /// Call f on every keycode in every section: layout, encoder_layout, macros
    /// (tap/down/up keycodes), tap_dance (4 keycodes), combo (5), key_override
    /// (trigger, replacement), alt_repeat_key (keycode, alt_keycode).
    pub fn for_each_keycode_mut(&mut self, f: &mut dyn FnMut(&mut Keycode)) {
        self.visit_keycodes(&mut |_, kc| f(kc));
    }

    /// Every keycode with its JSON path ("layout[1][0][0]", "combo[3][4]",
    /// "key_override[2].trigger", ...), in the order for_each_keycode_mut visits them.
    pub fn keycodes(&self) -> Vec<(String, Keycode)> {
        let mut copy = self.clone();
        let mut out = Vec::new();
        copy.visit_keycodes(&mut |path, kc| out.push((path, *kc)));
        out
    }

    /// Replace a validated set of existing keycode fields, without partial edits.
    pub fn replace_keycodes(
        &mut self,
        replacements: &std::collections::BTreeMap<String, Keycode>,
    ) -> crate::ops::OpResult {
        let existing: std::collections::BTreeSet<_> =
            self.keycodes().into_iter().map(|(path, _)| path).collect();
        for path in replacements.keys() {
            if !existing.contains(path) {
                return Err(crate::ops::OpError::Invalid(format!(
                    "Keycode field no longer exists: {path}"
                )));
            }
        }
        self.visit_keycodes(&mut |path, key| {
            if let Some(value) = replacements.get(&path) {
                *key = *value;
            }
        });
        Ok(crate::validate::validate(self)
            .into_iter()
            .map(|issue| format!("{}: {}", issue.path, issue.message))
            .collect())
    }

    /// The single traversal behind for_each_keycode_mut and keycodes.
    fn visit_keycodes(&mut self, f: &mut dyn FnMut(String, &mut Keycode)) {
        for (l, layer) in self.layout.iter_mut().enumerate() {
            for (r, row) in layer.iter_mut().enumerate() {
                for (c, slot) in row.iter_mut().enumerate() {
                    if let Slot::Key(kc) = slot {
                        f(format!("layout[{l}][{r}][{c}]"), kc);
                    }
                }
            }
        }
        if let Some(encoders) = &mut self.encoder_layout {
            for (l, layer) in encoders.iter_mut().enumerate() {
                for (e, pair) in layer.iter_mut().enumerate() {
                    for (i, slot) in pair.iter_mut().enumerate() {
                        if let Slot::Key(kc) = slot {
                            f(format!("encoder_layout[{l}][{e}][{i}]"), kc);
                        }
                    }
                }
            }
        }
        for (m, slot) in self.macros.iter_mut().enumerate() {
            for (a, action) in slot.actions.iter_mut().enumerate() {
                match action {
                    MacroAction::Tap(keys) | MacroAction::Down(keys) | MacroAction::Up(keys) => {
                        for (k, kc) in keys.iter_mut().enumerate() {
                            // The JSON array holds the action name at index 0.
                            f(format!("macro[{m}][{a}][{}]", k + 1), kc);
                        }
                    }
                    MacroAction::Text(_) | MacroAction::Delay(_) => {}
                }
            }
        }
        if let Some(list) = &mut self.tap_dance {
            for (i, td) in list.iter_mut().enumerate() {
                f(format!("tap_dance[{i}][0]"), &mut td.on_tap);
                f(format!("tap_dance[{i}][1]"), &mut td.on_hold);
                f(format!("tap_dance[{i}][2]"), &mut td.on_double_tap);
                f(format!("tap_dance[{i}][3]"), &mut td.on_tap_hold);
            }
        }
        if let Some(list) = &mut self.combo {
            for (i, combo) in list.iter_mut().enumerate() {
                for (k, kc) in combo.keys.iter_mut().enumerate() {
                    f(format!("combo[{i}][{k}]"), kc);
                }
                f(format!("combo[{i}][4]"), &mut combo.output);
            }
        }
        if let Some(list) = &mut self.key_override {
            for (i, ko) in list.iter_mut().enumerate() {
                f(format!("key_override[{i}].trigger"), &mut ko.trigger);
                f(
                    format!("key_override[{i}].replacement"),
                    &mut ko.replacement,
                );
            }
        }
        if let Some(list) = &mut self.alt_repeat_key {
            for (i, ar) in list.iter_mut().enumerate() {
                f(format!("alt_repeat_key[{i}].keycode"), &mut ar.keycode);
                f(
                    format!("alt_repeat_key[{i}].alt_keycode"),
                    &mut ar.alt_keycode,
                );
            }
        }
    }
}
