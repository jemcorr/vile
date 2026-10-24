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
    revision: u64,
    macro_capacity: Option<usize>,
    geometry_source: &'static str,
    cache_dir: Option<std::path::PathBuf>,
}

impl Session {
    pub fn new() -> Session {
        Session::default()
    }

    /// The view of the open file, or None when no file is open.
    pub fn view(&self) -> Option<ViewModel> {
        self.editor.as_ref().map(|editor| {
            build_view(
                editor,
                &self.warnings,
                self.revision,
                self.macro_capacity,
                self.geometry_source,
            )
        })
    }

    pub fn open_path(&mut self, path: &Path) -> Result<ViewModel, String> {
        let (editor, report) = Editor::open(path).map_err(|e| e.to_string())?;
        let mut editor = editor;
        if editor.board().is_none() {
            editor.set_board(Board::matrix(editor.doc()))?;
        }
        self.revision += 1;
        self.macro_capacity = None;
        self.geometry_source = "matrix";
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
        self.editor = Some(editor);
        self.warnings = warnings;
        self.load_cached_geometry();
        self.view_result()
    }

    pub fn save(&mut self) -> Result<ViewModel, String> {
        self.revision += 1;
        self.editor_mut()?.save().map_err(|e| e.to_string())?;
        self.warnings.clear();
        self.view_result()
    }

    pub fn save_as(&mut self, path: &Path) -> Result<ViewModel, String> {
        self.revision += 1;
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
        self.revision += 1;
        self.view_result()
    }

    pub fn undo(&mut self) -> Result<ViewModel, String> {
        let done = self.editor_mut()?.undo();
        self.revision += 1;
        self.warnings = if done {
            Vec::new()
        } else {
            vec!["nothing to undo".to_string()]
        };
        self.view_result()
    }

    pub fn redo(&mut self) -> Result<ViewModel, String> {
        let done = self.editor_mut()?.redo();
        self.revision += 1;
        self.warnings = if done {
            Vec::new()
        } else {
            vec!["nothing to redo".to_string()]
        };
        self.view_result()
    }

    pub fn search_keycodes(&self, query: &str, limit: usize) -> Vec<SearchHitDto> {
        let catalog = self.catalog();
        let mut hits = catalog.search(query, limit);
        if !query.trim().is_empty() {
            let needle: String = query
                .chars()
                .filter(|c| c.is_alphanumeric())
                .flat_map(char::to_lowercase)
                .collect();
            let mut named = Vec::new();
            for hit in catalog.search("", 4096) {
                if let Ok(kc) = catalog.parse(&hit.name) {
                    let description =
                        describe(self.editor.as_ref().map(|e| e.names()), &catalog, kc);
                    if description == catalog.description(kc) {
                        continue;
                    }
                    let text: String = description
                        .chars()
                        .filter(|c| c.is_alphanumeric())
                        .flat_map(char::to_lowercase)
                        .collect();
                    let mut wanted = needle.chars().peekable();
                    for c in text.chars() {
                        if wanted.peek() == Some(&c) {
                            wanted.next();
                        }
                    }
                    if wanted.peek().is_none() {
                        named.push(hit);
                    }
                }
            }
            named.retain(|hit| !hits.iter().any(|existing| existing.name == hit.name));
            named.extend(hits);
            hits = named;
            hits.truncate(limit);
        }
        hits.into_iter()
            .map(|hit| {
                let code = catalog.parse(&hit.name).ok();
                let picker = code.map(|kc| catalog.picker(kc));
                SearchHitDto {
                    picker_label: picker.as_ref().map(|p| p.label.clone()).unwrap_or_default(),
                    summary: picker
                        .as_ref()
                        .map(|p| p.summary.clone())
                        .unwrap_or_default(),
                    picker_group: picker.map(|p| p.group.into()).unwrap_or_default(),
                    code: code.map(|kc| kc.code()),
                    description: code
                        .map(|kc| {
                            let description =
                                describe(self.editor.as_ref().map(|e| e.names()), &catalog, kc);
                            if description == catalog.description(kc) {
                                catalog.picker_description(kc)
                            } else {
                                description
                            }
                        })
                        .unwrap_or_else(|| format!("{} builder", hit.name)),
                    category: code
                        .map(|kc| catalog.category(kc).as_str().into())
                        .unwrap_or_else(|| {
                            if hit.group == "layer_tap" {
                                "Layers"
                            } else {
                                "Modifiers"
                            }
                            .into()
                        }),
                    name: hit.name,
                    insert: hit.insert,
                    complete: hit.complete,
                    label: hit.label,
                    group: hit.group.to_string(),
                    matched: hit.matched,
                }
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

    pub fn check_revision(&self, revision: u64) -> Result<(), String> {
        if revision != self.revision {
            Err("The document changed; review the current values and try again".into())
        } else {
            Ok(())
        }
    }
    pub fn apply_op_at(&mut self, revision: u64, op: OpDto) -> Result<ViewModel, String> {
        self.check_revision(revision)?;
        self.apply_op(op)
    }
    pub fn keycode_view(&self, text: &str) -> Result<crate::KeycodeView, String> {
        let catalog = self.catalog();
        let kc = catalog.parse_input(text).map_err(err)?;
        Ok(keycode_view(
            self.editor.as_ref().map(|e| e.names()),
            &catalog,
            kc,
        ))
    }
    pub fn compose_keycode(
        &self,
        parts: vile_core::KeyParts,
    ) -> Result<crate::KeycodeView, String> {
        let kc = vile_core::keycode::compose(parts)?;
        Ok(keycode_view(
            self.editor.as_ref().map(|e| e.names()),
            &self.catalog(),
            kc,
        ))
    }
    pub fn macro_usage(&self, actions: &[MacroActionDto]) -> Result<crate::MacroUsage, String> {
        let m = macro_from_dto(&self.catalog(), actions)?;
        Ok(crate::MacroUsage {
            bytes: vile_core::validate::macro_slot_bytes(&m),
            action_bytes: m
                .actions
                .iter()
                .map(vile_core::validate::macro_action_bytes)
                .collect(),
        })
    }
    pub fn set_cache_dir(&mut self, directory: std::path::PathBuf) {
        self.cache_dir = Some(directory);
        self.load_cached_geometry();
    }
    fn load_cached_geometry(&mut self) {
        let Some(directory) = &self.cache_dir else {
            return;
        };
        let Some(editor) = &mut self.editor else {
            return;
        };
        match vile_device::DefinitionCache::load(directory, editor.doc()) {
            Ok(Some(cache)) => match cache
                .board(editor.doc())
                .and_then(|board| editor.set_board(board))
            {
                Ok(()) => {
                    self.macro_capacity = cache.macro_capacity;
                    self.geometry_source = "cache";
                }
                Err(e) => self.warnings.push(format!("Using matrix fallback: {e}")),
            },
            Ok(None) => {}
            Err(e) => self.warnings.push(format!(
                "Using matrix fallback; cached definition is invalid: {e}"
            )),
        }
    }
    pub fn cache_dir(&self) -> Option<&Path> {
        self.cache_dir.as_deref()
    }

    pub fn attach_board(
        &mut self,
        board: Board,
        capacity: Option<usize>,
    ) -> Result<ViewModel, String> {
        self.editor_mut()?.set_board(board)?;
        self.macro_capacity = capacity;
        self.geometry_source = "device";
        self.revision += 1;
        self.view_result()
    }
    pub fn document(&self) -> Result<&Document, String> {
        Ok(self.editor()?.doc())
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
        Ok(build_view(
            self.editor()?,
            &self.warnings,
            self.revision,
            self.macro_capacity,
            self.geometry_source,
        ))
    }
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn board_dto(board: &Board) -> BoardDto {
    BoardDto {
        id: board.id.clone(),
        can_mirror: board.can_mirror(),
        encoder_indices: board.encoder_indices.clone(),
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
                r: key.r,
                rx: key.rx,
                ry: key.ry,
                secondary: key.secondary.clone(),
                name: key.name.clone(),
            })
            .collect(),
    }
}

fn kind_name(kind: KeycodeKind, code: u16) -> &'static str {
    match kind {
        KeycodeKind::Basic { .. } => match code {
            0 => "no",
            1 => "trns",
            0xe0..=0xe7 => "modifier",
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
            match catalog.category(kc) {
                vile_core::keycode::PickerCategory::Mouse => "mouse".into(),
                vile_core::keycode::PickerCategory::Media => "media".into(),
                _ => kind_name(kc.kind(), kc.code()).to_string(),
            },
        ),
        _ => ("-1".to_string(), String::new(), "other".to_string()),
    };
    KeyCellDto {
        id: key.id.clone(),
        wire,
        label,
        kind,
        changed: base.get(layer, key.row, key.col) != slot,
        path: format!("layout[{layer}][{}][{}]", key.row, key.col),
        code: match slot {
            Some(Slot::Key(kc)) => kc.code(),
            _ => 0,
        },
        description: match slot {
            Some(Slot::Key(kc)) => catalog.description(kc),
            _ => "Absent position".into(),
        },
        previous: match base.get(layer, key.row, key.col) {
            Some(Slot::Key(kc)) => catalog.wire_name(kc),
            _ => "Absent".into(),
        },
    }
}

fn build_view(
    editor: &Editor,
    warnings: &[String],
    revision: u64,
    macro_capacity: Option<usize>,
    geometry_source: &str,
) -> ViewModel {
    let doc = editor.doc();
    let base = editor.disk_doc();
    let catalog = doc.catalog();
    let board = editor.board();
    let layers = match board {
        Some(board) => (0..doc.layer_count())
            .map(|layer| {
                board
                    .keys
                    .iter()
                    .map(|key| {
                        let mut cell = key_cell(doc, base, &catalog, layer, key);
                        if let Some(Slot::Key(kc)) = doc.get(layer, key.row, key.col) {
                            cell.description = describe(Some(editor.names()), &catalog, kc);
                        }
                        cell
                    })
                    .collect()
            })
            .collect(),
        None => Vec::new(),
    };
    ViewModel {
        revision,
        names: editor.names().clone(),
        keycodes: doc
            .keycodes()
            .into_iter()
            .map(|(path, kc)| (path, keycode_view(Some(editor.names()), &catalog, kc)))
            .collect(),
        pending_diff: diff_lines(editor, true),
        macro_bytes: vile_core::validate::macro_buffer_bytes(doc),
        macro_capacity,
        geometry_source: geometry_source.to_string(),
        path: editor.path().map(|p| p.display().to_string()),
        uid: doc.uid.to_string(),
        dirty: editor.is_dirty(),
        can_undo: editor.can_undo(),
        can_redo: editor.can_redo(),
        layer_count: doc.layer_count(),
        board: board.map(board_dto),
        layers,
        diff: diff_lines(editor, false),
        diff_base: match editor.diff_base() {
            DiffBase::Disk => "disk".to_string(),
            DiffBase::GitHead => "git".to_string(),
        },
        warnings: warnings
            .iter()
            .chain(editor.metadata_warnings())
            .cloned()
            .collect(),
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

fn describe(names: Option<&vile_core::metadata::Names>, catalog: &Catalog, kc: Keycode) -> String {
    let mut text = catalog.description(kc);
    if let Some(names) = names {
        if let Some(layer) = kc.layer_ref()
            && let Some(name) = names.layers.get(&(layer as usize))
        {
            text = text.replace(&format!("layer {layer}"), name);
        }
        if let KeycodeKind::Macro { index } = kc.kind()
            && let Some(name) = names.macros.get(&(index as usize))
        {
            text = format!("Macro {name}");
        }
    }
    text
}
fn keycode_view(
    names: Option<&vile_core::metadata::Names>,
    catalog: &Catalog,
    kc: Keycode,
) -> crate::KeycodeView {
    crate::KeycodeView {
        code: kc.code(),
        wire: catalog.wire_name(kc),
        label: catalog.label(kc),
        description: describe(names, catalog, kc),
        category: catalog.category(kc).as_str().into(),
        parts: vile_core::keycode::decompose(kc),
    }
}
fn describe_diff_value(
    doc: &Document,
    names: Option<&vile_core::metadata::Names>,
    entry: &vile_core::DiffEntry,
    value: &str,
) -> String {
    let catalog = doc.catalog();
    if let Some(index) = entry.index {
        match entry.section {
            vile_core::DiffSection::Macro => {
                if let Some(slot) = doc.macros.get(index) {
                    if slot.actions.is_empty() {
                        return "Empty macro".into();
                    }
                    return slot
                        .actions
                        .iter()
                        .map(|action| match action {
                            MacroAction::Text(text) => format!("Text {text:?}"),
                            MacroAction::Delay(ms) => format!("Delay {ms} ms"),
                            MacroAction::Tap(keys)
                            | MacroAction::Down(keys)
                            | MacroAction::Up(keys) => {
                                let kind = match action {
                                    MacroAction::Tap(_) => "Tap",
                                    MacroAction::Down(_) => "Press",
                                    _ => "Release",
                                };
                                format!(
                                    "{kind} {}",
                                    keys.iter()
                                        .map(|key| describe(names, &catalog, *key))
                                        .collect::<Vec<_>>()
                                        .join(" + ")
                                )
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("; ");
                }
            }
            vile_core::DiffSection::Combo => {
                if let Some(combo) = doc.combo.as_ref().and_then(|combos| combos.get(index)) {
                    let keys = combo
                        .keys
                        .iter()
                        .filter(|key| **key != Keycode::NO)
                        .map(|key| describe(names, &catalog, *key))
                        .collect::<Vec<_>>();
                    return format!(
                        "{} → {}",
                        if keys.is_empty() {
                            "No triggers".into()
                        } else {
                            keys.join(" + ")
                        },
                        describe(names, &catalog, combo.output)
                    );
                }
            }
            _ => {}
        }
    }
    catalog
        .parse(value)
        .map(|key| describe(names, &catalog, key))
        .unwrap_or_else(|_| value.into())
}

fn diff_lines(editor: &Editor, pending: bool) -> Vec<DiffLineDto> {
    let entries = if pending {
        vile_core::diff::diff(editor.disk_doc(), editor.doc(), editor.board())
    } else {
        editor.diff()
    };
    let mut lines: Vec<_> = entries
        .into_iter()
        .map(|d| {
            let old_doc = if pending {
                editor.disk_doc()
            } else {
                editor.base_doc()
            };
            let old_names = if !pending && editor.diff_base() == DiffBase::GitHead {
                None
            } else {
                Some(editor.disk_names())
            };
            let old_description = describe_diff_value(old_doc, old_names, &d, &d.old);
            let new_description =
                describe_diff_value(editor.doc(), Some(editor.names()), &d, &d.new);
            DiffLineDto {
                section: d.section.as_str().into(),
                line: d.line,
                layer: d.layer,
                key_id: d.key_id,
                index: d.index,
                field: d.field,
                old: d.old,
                new: d.new,
                old_description,
                new_description,
            }
        })
        .collect();
    for (kind, old, new) in [
        ("layer", &editor.disk_names().layers, &editor.names().layers),
        ("macro", &editor.disk_names().macros, &editor.names().macros),
        ("combo", &editor.disk_names().combos, &editor.names().combos),
    ] {
        let indices: std::collections::BTreeSet<_> =
            old.keys().chain(new.keys()).copied().collect();
        for index in indices {
            if old.get(&index) != new.get(&index) {
                let a = old.get(&index).cloned().unwrap_or_default();
                let b = new.get(&index).cloned().unwrap_or_default();
                lines.push(DiffLineDto {
                    section: "names".into(),
                    line: format!("{kind} {index} name: {a} → {b}"),
                    layer: (kind == "layer").then_some(index),
                    key_id: None,
                    index: Some(index),
                    field: Some(kind.into()),
                    old: a.clone(),
                    new: b.clone(),
                    old_description: a,
                    new_description: b,
                });
            }
        }
    }
    if pending && editor.names_pending() && lines.iter().all(|line| line.section != "names") {
        lines.push(DiffLineDto {
            section: "names".into(),
            line: "Names sidecar still needs saving".into(),
            layer: None,
            key_id: None,
            index: None,
            field: None,
            old: "unsaved".into(),
            new: "retry save".into(),
            old_description: "Names sidecar not saved".into(),
            new_description: "Retry saving names".into(),
        });
    }
    lines
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
        OpDto::PatchFields { paths, patch } => {
            let existing: std::collections::BTreeMap<_, _> =
                editor.doc().keycodes().into_iter().collect();
            let mut replacements = std::collections::BTreeMap::new();
            for path in paths {
                let current = existing
                    .get(&path)
                    .ok_or_else(|| format!("Keycode field no longer exists: {path}"))?;
                replacements.insert(
                    path.clone(),
                    patch.apply(*current).map_err(|e| format!("{path}: {e}"))?,
                );
            }
            editor
                .apply(|doc| doc.replace_keycodes(&replacements))
                .map_err(err)
        }
        OpDto::ReplaceFields { paths, keycode } => {
            let kc = key_code(catalog, &keycode)?;
            let replacements = paths.into_iter().map(|p| (p, kc)).collect();
            editor
                .apply(|doc| doc.replace_keycodes(&replacements))
                .map_err(err)
        }
        OpDto::Rename { kind, index, name } => {
            let name = name.trim().to_string();
            if name.chars().count() > 100 || name.chars().any(char::is_control) {
                return Err(
                    "Names must be at most 100 characters without control characters".into(),
                );
            }
            editor
                .apply_named(|doc, names| {
                    let (map, count) = match kind.as_str() {
                        "layer" => (&mut names.layers, doc.layer_count()),
                        "macro" => (&mut names.macros, doc.macros.len()),
                        "combo" => (&mut names.combos, doc.combo.as_ref().map_or(0, Vec::len)),
                        _ => return Err(ops::OpError::Invalid("Unknown name kind".into())),
                    };
                    if index >= count {
                        return Err(ops::OpError::Invalid("Named item does not exist".into()));
                    }
                    if name.is_empty() {
                        map.remove(&index);
                    } else {
                        map.insert(index, name);
                    }
                    Ok(Vec::new())
                })
                .map_err(err)
        }
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
        OpDto::SwapLayers { a, b } => editor
            .apply_named(|doc, names| {
                let result = ops::swap_layers(doc, a, b)?;
                let mut perm: Vec<_> = (0..doc.layer_count()).collect();
                perm.swap(a, b);
                names.remap_layers(&perm);
                Ok(result)
            })
            .map_err(err),
        OpDto::MoveLayer { from, to } => editor
            .apply_named(|doc, names| {
                let result = ops::move_layer(doc, from, to)?;
                let mut order: Vec<_> = (0..doc.layer_count()).collect();
                let old = order.remove(from);
                order.insert(to, old);
                let mut perm = vec![0; order.len()];
                for (new, old) in order.into_iter().enumerate() {
                    perm[old] = new;
                }
                names.remap_layers(&perm);
                Ok(result)
            })
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
