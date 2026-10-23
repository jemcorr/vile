//! The editing session behind the desktop app: wraps vile_core::Editor and
//! turns its state into JSON-friendly view models.

use std::path::Path;

use vile_core::ops;
use vile_core::{
    AltRepeatKey, Board, Catalog, CatalogContext, Combo, Editor, KeyOverride, KeyRef, KeycodeKind,
    Macro, MacroAction, MirrorMode, Pos, Slot, TapDance,
};
use vile_core::{DiffBase, Document, Keycode};

use crate::dto::{
    AltRepeatDto, BoardDto, BoardKeyDto, ComboDto, DiffLineDto, KeyCellDto, KeyOverrideDto,
    KeyRefDto, MacroActionDto, MacroDto, OpDto, SearchHitDto, TapDanceDto, ViewModel,
};

/// At most one open document. Every method returns user-facing error text.
#[derive(Debug, Default)]
pub struct Session {
    editor: Option<Editor>,
    /// Warnings from the last load or operation, shown in the UI.
    warnings: Vec<String>,
}

impl Session {
    pub fn new() -> Session {
        Session::default()
    }

    /// The view of the open file, or None when no file is open.
    pub fn view(&self) -> Option<ViewModel> {
        self.editor
            .as_ref()
            .map(|editor| build_view(editor, &self.warnings))
    }

    pub fn open_path(&mut self, path: &Path) -> Result<ViewModel, String> {
        let (editor, report) = Editor::open(path).map_err(|e| e.to_string())?;
        if editor.board().is_none() {
            return Err(
                "unsupported board: vile 0.1 supports only the Corne v4.1 (8x7 matrix, 46 keys)"
                    .to_string(),
            );
        }
        let mut warnings: Vec<String> = report
            .warnings
            .iter()
            .map(|w| format!("{}: {}", w.path, w.message))
            .collect();
        if !report.canonical {
            warnings.push(
                "this file is not in Vial's exact format; saving will rewrite it in Vial's format"
                    .to_string(),
            );
        }
        let view = build_view(&editor, &warnings);
        self.editor = Some(editor);
        self.warnings = warnings;
        Ok(view)
    }

    pub fn save(&mut self) -> Result<ViewModel, String> {
        self.editor_mut()?.save().map_err(|e| e.to_string())?;
        self.warnings.clear();
        self.view_result()
    }

    pub fn save_as(&mut self, path: &Path) -> Result<ViewModel, String> {
        self.editor_mut()?
            .save_as(path)
            .map_err(|e| e.to_string())?;
        self.warnings.clear();
        self.view_result()
    }

    pub fn apply_op(&mut self, op: OpDto) -> Result<ViewModel, String> {
        let catalog = self.catalog();
        let board = self
            .editor()?
            .board()
            .cloned()
            .ok_or_else(|| "unsupported board".to_string())?;
        let warnings = apply_to(self.editor_mut()?, op, &board, &catalog)?;
        self.warnings = warnings;
        self.view_result()
    }

    pub fn undo(&mut self) -> Result<ViewModel, String> {
        let done = self.editor_mut()?.undo();
        self.warnings = if done {
            Vec::new()
        } else {
            vec!["nothing to undo".to_string()]
        };
        self.view_result()
    }

    pub fn redo(&mut self) -> Result<ViewModel, String> {
        let done = self.editor_mut()?.redo();
        self.warnings = if done {
            Vec::new()
        } else {
            vec!["nothing to redo".to_string()]
        };
        self.view_result()
    }

    pub fn search_keycodes(&self, query: &str, limit: usize) -> Vec<SearchHitDto> {
        self.catalog()
            .search(query, limit)
            .into_iter()
            .map(|hit| SearchHitDto {
                name: hit.name,
                insert: hit.insert,
                complete: hit.complete,
                label: hit.label,
                group: hit.group.to_string(),
                matched: hit.matched,
            })
            .collect()
    }

    pub fn resolve_keycode(&self, text: &str) -> Result<String, String> {
        let catalog = self.catalog();
        let kc = catalog.parse_input(text).map_err(|e| e.to_string())?;
        Ok(catalog.wire_name(kc))
    }

    pub fn set_diff_base(&mut self, base: &str) -> Result<ViewModel, String> {
        let editor = self.editor_mut()?;
        match base {
            "disk" => editor.set_diff_base_disk(),
            "git" => editor.set_diff_base_git()?,
            other => return Err(format!("unknown diff base \"{other}\"")),
        }
        self.view_result()
    }

    /// The open document's catalog, or Catalog::new(CatalogContext::default()).
    fn catalog(&self) -> Catalog {
        match &self.editor {
            Some(editor) => editor.doc().catalog(),
            None => Catalog::new(CatalogContext::default()),
        }
    }

    fn editor(&self) -> Result<&Editor, String> {
        self.editor
            .as_ref()
            .ok_or_else(|| "no file is open".to_string())
    }

    fn editor_mut(&mut self) -> Result<&mut Editor, String> {
        self.editor
            .as_mut()
            .ok_or_else(|| "no file is open".to_string())
    }

    fn view_result(&self) -> Result<ViewModel, String> {
        Ok(build_view(self.editor()?, &self.warnings))
    }
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn board_dto(board: &Board) -> BoardDto {
    BoardDto {
        id: board.id.clone(),
        name: board.name.clone(),
        keys: board
            .keys
            .iter()
            .map(|key| BoardKeyDto {
                id: key.id.clone(),
                row: key.row,
                col: key.col,
                x: key.x,
                y: key.y,
                w: key.w,
                h: key.h,
            })
            .collect(),
    }
}

fn kind_name(kind: KeycodeKind, code: u16) -> &'static str {
    match kind {
        KeycodeKind::Basic { .. } => match code {
            0 => "no",
            1 => "trns",
            _ => "basic",
        },
        KeycodeKind::Mods { .. } => "mods",
        KeycodeKind::ModTap { .. } => "modTap",
        KeycodeKind::LayerTap { .. } => "layerTap",
        KeycodeKind::LayerMod { .. } => "layerMod",
        KeycodeKind::Layer { .. } => "layer",
        KeycodeKind::OneShotMod { .. } => "oneShotMod",
        KeycodeKind::TapDance { .. } => "tapDance",
        KeycodeKind::Macro { .. } => "macro",
        KeycodeKind::User { .. } => "user",
        KeycodeKind::Other => "other",
    }
}

fn key_cell(
    doc: &Document,
    base: &Document,
    catalog: &Catalog,
    layer: usize,
    key: &vile_core::BoardKey,
) -> KeyCellDto {
    let slot = doc.get(layer, key.row, key.col);
    let (wire, label, kind) = match slot {
        Some(Slot::Key(kc)) => (
            catalog.wire_name(kc),
            catalog.label(kc),
            kind_name(kc.kind(), kc.code()).to_string(),
        ),
        _ => ("-1".to_string(), String::new(), "other".to_string()),
    };
    KeyCellDto {
        id: key.id.clone(),
        wire,
        label,
        kind,
        changed: base.get(layer, key.row, key.col) != slot,
    }
}

fn build_view(editor: &Editor, warnings: &[String]) -> ViewModel {
    let doc = editor.doc();
    let base = editor.base_doc();
    let catalog = doc.catalog();
    let board = editor.board();
    let layers = match board {
        Some(board) => (0..doc.layer_count())
            .map(|layer| {
                board
                    .keys
                    .iter()
                    .map(|key| key_cell(doc, base, &catalog, layer, key))
                    .collect()
            })
            .collect(),
        None => Vec::new(),
    };
    ViewModel {
        path: editor.path().map(|p| p.display().to_string()),
        uid: doc.uid.to_string(),
        dirty: editor.is_dirty(),
        can_undo: editor.can_undo(),
        can_redo: editor.can_redo(),
        layer_count: doc.layer_count(),
        board: board.map(board_dto),
        layers,
        diff: editor
            .diff()
            .into_iter()
            .map(|d| DiffLineDto {
                section: d.section.as_str().to_string(),
                line: d.line,
                layer: d.layer,
                key_id: d.key_id,
            })
            .collect(),
        diff_base: match editor.diff_base() {
            DiffBase::Disk => "disk".to_string(),
            DiffBase::GitHead => "git".to_string(),
        },
        warnings: warnings.to_vec(),
        combos: doc
            .combo
            .as_ref()
            .map(|list| {
                list.iter()
                    .enumerate()
                    .map(|(index, combo)| ComboDto {
                        index,
                        keys: combo.keys.iter().map(|k| catalog.wire_name(*k)).collect(),
                        output: catalog.wire_name(combo.output),
                        output_label: catalog.label(combo.output),
                    })
                    .collect()
            })
            .unwrap_or_default(),
        macros: doc
            .macros
            .iter()
            .enumerate()
            .map(|(index, m)| macro_to_dto(&catalog, index, m))
            .collect(),
        tap_dances: doc
            .tap_dance
            .as_ref()
            .map(|list| {
                list.iter()
                    .enumerate()
                    .map(|(index, td)| TapDanceDto {
                        index,
                        on_tap: catalog.wire_name(td.on_tap),
                        on_hold: catalog.wire_name(td.on_hold),
                        on_double_tap: catalog.wire_name(td.on_double_tap),
                        on_tap_hold: catalog.wire_name(td.on_tap_hold),
                        tapping_term: td.tapping_term,
                    })
                    .collect()
            })
            .unwrap_or_default(),
        key_overrides: doc
            .key_override
            .as_ref()
            .map(|list| {
                list.iter()
                    .enumerate()
                    .map(|(index, ko)| KeyOverrideDto {
                        index,
                        trigger: catalog.wire_name(ko.trigger),
                        replacement: catalog.wire_name(ko.replacement),
                        layers: ko.layers,
                        trigger_mods: ko.trigger_mods,
                        negative_mod_mask: ko.negative_mod_mask,
                        suppressed_mods: ko.suppressed_mods,
                        options: ko.options,
                    })
                    .collect()
            })
            .unwrap_or_default(),
        alt_repeat_keys: doc
            .alt_repeat_key
            .as_ref()
            .map(|list| {
                list.iter()
                    .enumerate()
                    .map(|(index, ar)| AltRepeatDto {
                        index,
                        keycode: catalog.wire_name(ar.keycode),
                        alt_keycode: catalog.wire_name(ar.alt_keycode),
                        allowed_mods: ar.allowed_mods,
                        options: ar.options,
                    })
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn macro_to_dto(catalog: &Catalog, index: usize, m: &Macro) -> MacroDto {
    let actions = m
        .actions
        .iter()
        .map(|action| match action {
            MacroAction::Tap(keys) => MacroActionDto {
                kind: "tap".to_string(),
                keys: keys.iter().map(|k| catalog.wire_name(*k)).collect(),
                text: String::new(),
                delay: 0,
            },
            MacroAction::Down(keys) => MacroActionDto {
                kind: "down".to_string(),
                keys: keys.iter().map(|k| catalog.wire_name(*k)).collect(),
                text: String::new(),
                delay: 0,
            },
            MacroAction::Up(keys) => MacroActionDto {
                kind: "up".to_string(),
                keys: keys.iter().map(|k| catalog.wire_name(*k)).collect(),
                text: String::new(),
                delay: 0,
            },
            MacroAction::Text(text) => MacroActionDto {
                kind: "text".to_string(),
                keys: Vec::new(),
                text: text.clone(),
                delay: 0,
            },
            MacroAction::Delay(ms) => MacroActionDto {
                kind: "delay".to_string(),
                keys: Vec::new(),
                text: String::new(),
                delay: *ms,
            },
        })
        .collect();
    MacroDto { index, actions }
}

fn macro_from_dto(catalog: &Catalog, actions: &[MacroActionDto]) -> Result<Macro, String> {
    let mut out = Vec::new();
    for action in actions {
        let parsed = match action.kind.as_str() {
            "tap" | "down" | "up" => {
                let keys = action
                    .keys
                    .iter()
                    .map(|k| catalog.parse_input(k).map_err(err))
                    .collect::<Result<Vec<_>, String>>()?;
                match action.kind.as_str() {
                    "tap" => MacroAction::Tap(keys),
                    "down" => MacroAction::Down(keys),
                    _ => MacroAction::Up(keys),
                }
            }
            "text" => MacroAction::Text(action.text.clone()),
            "delay" => MacroAction::Delay(action.delay),
            other => return Err(format!("unknown macro action \"{other}\"")),
        };
        out.push(parsed);
    }
    Ok(Macro { actions: out })
}

fn positions(board: &Board, ids: &[String]) -> Result<Vec<Pos>, String> {
    ids.iter()
        .map(|id| {
            board
                .key_by_id(id)
                .map(|key| Pos {
                    row: key.row,
                    col: key.col,
                })
                .ok_or_else(|| format!("unknown key id \"{id}\""))
        })
        .collect()
}

fn key_refs(board: &Board, refs: &[KeyRefDto]) -> Result<Vec<KeyRef>, String> {
    refs.iter()
        .map(|r| -> Result<KeyRef, String> {
            let key = board
                .key_by_id(&r.key)
                .ok_or_else(|| format!("unknown key id \"{}\"", r.key))?;
            Ok(KeyRef {
                layer: r.layer,
                pos: Pos {
                    row: key.row,
                    col: key.col,
                },
            })
        })
        .collect()
}

fn key_code(catalog: &Catalog, text: &str) -> Result<Keycode, String> {
    catalog.parse_input(text).map_err(err)
}

/// Applies one operation to the editor and returns the warnings it produced.
fn apply_to(
    editor: &mut Editor,
    op: OpDto,
    board: &Board,
    catalog: &Catalog,
) -> Result<Vec<String>, String> {
    match op {
        OpDto::SetKeys {
            layer,
            keys,
            keycode,
        } => {
            let targets = positions(board, &keys)?;
            let kc = key_code(catalog, &keycode)?;
            editor
                .apply(|doc| ops::set_keys(doc, layer, &targets, kc))
                .map_err(err)
        }
        OpDto::ClearKeys { layer, keys } => {
            let targets = positions(board, &keys)?;
            editor
                .apply(|doc| ops::clear_keys(doc, layer, &targets))
                .map_err(err)
        }
        OpDto::TransparentKeys { layer, keys } => {
            let targets = positions(board, &keys)?;
            editor
                .apply(|doc| ops::transparent_keys(doc, layer, &targets))
                .map_err(err)
        }
        OpDto::CopyKeys { layer, keys } => {
            let targets = positions(board, &keys)?;
            editor.copy_keys(layer, &targets).map_err(err)?;
            Ok(Vec::new())
        }
        OpDto::PasteKeys { layer, keys } => {
            let selection = positions(board, &keys)?;
            editor.paste_keys(layer, &selection).map_err(err)
        }
        OpDto::SwapKeys { a, b } => {
            let a = key_refs(board, &a)?;
            let b = key_refs(board, &b)?;
            editor.apply(|doc| ops::swap_keys(doc, &a, &b)).map_err(err)
        }
        OpDto::ClearLayer { layer, transparent } => {
            let kc = if transparent {
                Keycode::TRNS
            } else {
                Keycode::NO
            };
            editor
                .apply(|doc| ops::fill_layer(doc, layer, kc))
                .map_err(err)
        }
        OpDto::CopyLayer { layer } => {
            editor.copy_layer(layer).map_err(err)?;
            Ok(Vec::new())
        }
        OpDto::PasteLayer { layer } => editor.paste_layer(layer).map_err(err),
        OpDto::SwapLayers { a, b } => editor.apply(|doc| ops::swap_layers(doc, a, b)).map_err(err),
        OpDto::MoveLayer { from, to } => editor
            .apply(|doc| ops::move_layer(doc, from, to))
            .map_err(err),
        OpDto::MirrorLayer { layer, mode } => {
            let mode = match mode.as_str() {
                "swap" => MirrorMode::Swap,
                "leftToRight" => MirrorMode::LeftToRight,
                "rightToLeft" => MirrorMode::RightToLeft,
                other => return Err(format!("unknown mirror mode \"{other}\"")),
            };
            editor
                .apply(|doc| ops::mirror_layer(doc, board, layer, mode))
                .map_err(err)
        }
        OpDto::SetCombo {
            index,
            keys,
            output,
        } => {
            if keys.len() != 4 {
                return Err(
                    "a combo needs exactly 4 input keys (use KC_NO for unused ones)".to_string(),
                );
            }
            let combo = Combo {
                keys: [
                    key_code(catalog, &keys[0])?,
                    key_code(catalog, &keys[1])?,
                    key_code(catalog, &keys[2])?,
                    key_code(catalog, &keys[3])?,
                ],
                output: key_code(catalog, &output)?,
            };
            editor
                .apply(|doc| ops::set_combo(doc, index, combo))
                .map_err(err)
        }
        OpDto::SetMacro { index, actions } => {
            let m = macro_from_dto(catalog, &actions)?;
            editor
                .apply(|doc| ops::set_macro(doc, index, m))
                .map_err(err)
        }
        OpDto::SetTapDance {
            index,
            on_tap,
            on_hold,
            on_double_tap,
            on_tap_hold,
            tapping_term,
        } => {
            let td = TapDance {
                on_tap: key_code(catalog, &on_tap)?,
                on_hold: key_code(catalog, &on_hold)?,
                on_double_tap: key_code(catalog, &on_double_tap)?,
                on_tap_hold: key_code(catalog, &on_tap_hold)?,
                tapping_term,
            };
            editor
                .apply(|doc| ops::set_tap_dance(doc, index, td))
                .map_err(err)
        }
        OpDto::SetKeyOverride {
            index,
            trigger,
            replacement,
            layers,
            trigger_mods,
            negative_mod_mask,
            suppressed_mods,
            options,
        } => {
            let ko = KeyOverride {
                trigger: key_code(catalog, &trigger)?,
                replacement: key_code(catalog, &replacement)?,
                layers,
                trigger_mods,
                negative_mod_mask,
                suppressed_mods,
                options,
            };
            editor
                .apply(|doc| ops::set_key_override(doc, index, ko))
                .map_err(err)
        }
        OpDto::SetAltRepeat {
            index,
            keycode,
            alt_keycode,
            allowed_mods,
            options,
        } => {
            let ar = AltRepeatKey {
                keycode: key_code(catalog, &keycode)?,
                alt_keycode: key_code(catalog, &alt_keycode)?,
                allowed_mods,
                options,
            };
            editor
                .apply(|doc| ops::set_alt_repeat_key(doc, index, ar))
                .map_err(err)
        }
    }
}
