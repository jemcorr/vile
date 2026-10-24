//! Data transfer objects exchanged with the web view as JSON (camelCase).
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vile_core::metadata::Names;
use vile_core::{KeyParts, KeyPartsPatch};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardKeyDto {
    pub id: String,
    pub row: usize,
    pub col: usize,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub r: f64,
    pub rx: f64,
    pub ry: f64,
    pub secondary: Option<vile_core::board::KeyRect>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardDto {
    pub id: String,
    pub name: String,
    pub keys: Vec<BoardKeyDto>,
    pub can_mirror: bool,
    pub encoder_indices: Option<Vec<usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyCellDto {
    pub id: String,
    pub wire: String,
    pub label: String,
    pub kind: String,
    pub changed: bool,
    pub path: String,
    pub code: u16,
    pub description: String,
    pub previous: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffLineDto {
    pub section: String,
    pub line: String,
    pub layer: Option<usize>,
    pub key_id: Option<String>,
    pub index: Option<usize>,
    pub field: Option<String>,
    pub old: String,
    pub new: String,
    pub old_description: String,
    pub new_description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComboDto {
    pub index: usize,
    pub keys: Vec<String>,
    pub output: String,
    pub output_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MacroActionDto {
    pub kind: String,
    pub keys: Vec<String>,
    pub text: String,
    pub delay: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MacroDto {
    pub index: usize,
    pub actions: Vec<MacroActionDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TapDanceDto {
    pub index: usize,
    pub on_tap: String,
    pub on_hold: String,
    pub on_double_tap: String,
    pub on_tap_hold: String,
    pub tapping_term: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyOverrideDto {
    pub index: usize,
    pub trigger: String,
    pub replacement: String,
    pub layers: u16,
    pub trigger_mods: u8,
    pub negative_mod_mask: u8,
    pub suppressed_mods: u8,
    pub options: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AltRepeatDto {
    pub index: usize,
    pub keycode: String,
    pub alt_keycode: String,
    pub allowed_mods: u8,
    pub options: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewModel {
    pub revision: u64,
    pub names: Names,
    pub keycodes: BTreeMap<String, KeycodeView>,
    pub pending_diff: Vec<DiffLineDto>,
    pub macro_bytes: usize,
    pub macro_capacity: Option<usize>,
    pub geometry_source: String,
    pub path: Option<String>,
    pub uid: String,
    pub dirty: bool,
    pub can_undo: bool,
    pub can_redo: bool,
    pub layer_count: usize,
    pub board: Option<BoardDto>,
    pub layers: Vec<Vec<KeyCellDto>>,
    pub diff: Vec<DiffLineDto>,
    pub diff_base: String,
    pub warnings: Vec<String>,
    pub combos: Vec<ComboDto>,
    pub macros: Vec<MacroDto>,
    pub tap_dances: Vec<TapDanceDto>,
    pub key_overrides: Vec<KeyOverrideDto>,
    pub alt_repeat_keys: Vec<AltRepeatDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHitDto {
    pub picker_label: String,
    pub summary: String,
    pub picker_group: String,
    pub code: Option<u16>,
    pub description: String,
    pub category: String,
    pub name: String,
    pub insert: String,
    pub complete: bool,
    pub label: String,
    pub group: String,
    pub matched: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyRefDto {
    pub layer: usize,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum OpDto {
    PatchFields {
        paths: Vec<String>,
        patch: KeyPartsPatch,
    },
    ReplaceFields {
        paths: Vec<String>,
        keycode: String,
    },
    Rename {
        kind: String,
        index: usize,
        name: String,
    },
    SetKeys {
        layer: usize,
        keys: Vec<String>,
        keycode: String,
    },
    ClearKeys {
        layer: usize,
        keys: Vec<String>,
    },
    TransparentKeys {
        layer: usize,
        keys: Vec<String>,
    },
    CopyKeys {
        layer: usize,
        keys: Vec<String>,
    },
    PasteKeys {
        layer: usize,
        keys: Vec<String>,
    },
    SwapKeys {
        a: Vec<KeyRefDto>,
        b: Vec<KeyRefDto>,
    },
    ClearLayer {
        layer: usize,
        transparent: bool,
    },
    CopyLayer {
        layer: usize,
    },
    PasteLayer {
        layer: usize,
    },
    SwapLayers {
        a: usize,
        b: usize,
    },
    MoveLayer {
        from: usize,
        to: usize,
    },
    MirrorLayer {
        layer: usize,
        mode: String,
    },
    SetCombo {
        index: usize,
        keys: Vec<String>,
        output: String,
    },
    SetMacro {
        index: usize,
        actions: Vec<MacroActionDto>,
    },
    SetTapDance {
        index: usize,
        on_tap: String,
        on_hold: String,
        on_double_tap: String,
        on_tap_hold: String,
        tapping_term: u16,
    },
    SetKeyOverride {
        index: usize,
        trigger: String,
        replacement: String,
        layers: u16,
        trigger_mods: u8,
        negative_mod_mask: u8,
        suppressed_mods: u8,
        options: u8,
    },
    SetAltRepeat {
        index: usize,
        keycode: String,
        alt_keycode: String,
        allowed_mods: u8,
        options: u8,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeycodeView {
    pub code: u16,
    pub wire: String,
    pub label: String,
    pub description: String,
    pub category: String,
    pub parts: KeyParts,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MacroUsage {
    pub bytes: usize,
    pub action_bytes: Vec<usize>,
}
