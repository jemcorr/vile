//! Semantic diff between two documents, one line per changed item, for
//! example "L0 thumb-left-mid: MO(7) -> LT8(KC_ESCAPE)".

use crate::board::Board;
use crate::document::{AltRepeatKey, Combo, Document, KeyOverride, Macro, MacroAction, Slot};
use crate::keycode::{Catalog, Keycode};
use crate::pyjson::write_py_string;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DiffSection {
    Meta,
    Layout,
    Encoder,
    Macro,
    TapDance,
    Combo,
    KeyOverride,
    AltRepeatKey,
    Settings,
}

impl DiffSection {
    pub fn as_str(self) -> &'static str {
        match self {
            DiffSection::Meta => "meta",
            DiffSection::Layout => "layout",
            DiffSection::Encoder => "encoder",
            DiffSection::Macro => "macro",
            DiffSection::TapDance => "tapDance",
            DiffSection::Combo => "combo",
            DiffSection::KeyOverride => "keyOverride",
            DiffSection::AltRepeatKey => "altRepeatKey",
            DiffSection::Settings => "settings",
        }
    }
}

/// One changed item. `line` is the formatted text; the other fields let a UI
/// jump to the item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffEntry {
    pub section: DiffSection,
    pub layer: Option<usize>,
    pub row: Option<usize>,
    pub col: Option<usize>,
    pub key_id: Option<String>,
    pub index: Option<usize>,
    pub field: Option<String>,
    pub old: String,
    pub new: String,
    pub line: String,
}

pub fn diff(old: &Document, new: &Document, board: Option<&Board>) -> Vec<DiffEntry> {
    let oc = old.catalog();
    let nc = new.catalog();
    let mut out = Vec::new();

    // 1. Meta.
    let meta = |out: &mut Vec<DiffEntry>, name: &str, a: String, b: String| {
        if a != b {
            let mut e = entry(DiffSection::Meta, a, b);
            e.line = format!("{name}: {} -> {}", e.old, e.new);
            e.field = Some(name.to_string());
            out.push(e);
        }
    };
    meta(
        &mut out,
        "version",
        old.version.to_string(),
        new.version.to_string(),
    );
    meta(&mut out, "uid", old.uid.to_string(), new.uid.to_string());
    meta(
        &mut out,
        "layout_options",
        old.layout_options.to_string(),
        new.layout_options.to_string(),
    );
    meta(
        &mut out,
        "vial_protocol",
        old.vial_protocol.to_string(),
        new.vial_protocol.to_string(),
    );
    meta(
        &mut out,
        "via_protocol",
        old.via_protocol.to_string(),
        new.via_protocol.to_string(),
    );
    meta(
        &mut out,
        "layers",
        old.layer_count().to_string(),
        new.layer_count().to_string(),
    );
    meta(
        &mut out,
        "matrix",
        format!("{}x{}", old.rows(), old.cols()),
        format!("{}x{}", new.rows(), new.cols()),
    );

    // 2. Layout, when the matrix shape is the same.
    if old.rows() == new.rows() && old.cols() == new.cols() {
        let board = board.filter(|b| b.matches(old) && b.matches(new));
        for l in 0..old.layer_count().min(new.layer_count()) {
            let mut positions: Vec<(usize, usize, Option<String>)> = Vec::new();
            match board {
                Some(b) => {
                    for key in &b.keys {
                        positions.push((key.row, key.col, Some(key.id.clone())));
                    }
                }
                None => {
                    for r in 0..old.rows() {
                        for c in 0..old.cols() {
                            positions.push((r, c, None));
                        }
                    }
                }
            }
            for (r, c, key_id) in positions {
                let (Some(a), Some(b)) = (old.get(l, r, c), new.get(l, r, c)) else {
                    continue;
                };
                if a == b {
                    continue;
                }
                let name = key_id.clone().unwrap_or_else(|| format!("r{r}c{c}"));
                let (from, to) = (slot_text(&oc, a), slot_text(&nc, b));
                let mut e = entry(DiffSection::Layout, from.clone(), to.clone());
                e.line = format!("L{l} {name}: {from} -> {to}");
                e.layer = Some(l);
                e.row = Some(r);
                e.col = Some(c);
                e.key_id = key_id;
                out.push(e);
            }
        }
    }

    // 3. Encoders.
    match (&old.encoder_layout, &new.encoder_layout) {
        (Some(oe), Some(ne)) => {
            for l in 0..oe.len().min(ne.len()) {
                for e in 0..oe[l].len().min(ne[l].len()) {
                    let (a, b) = (oe[l][e], ne[l][e]);
                    for (side, x, y) in [("cw", a[0], b[0]), ("ccw", a[1], b[1])] {
                        if x != y {
                            let (from, to) = (slot_text(&oc, x), slot_text(&nc, y));
                            let mut item = entry(DiffSection::Encoder, from.clone(), to.clone());
                            item.line = format!("L{l} encoder {e} {side}: {from} -> {to}");
                            item.layer = Some(l);
                            item.index = Some(e);
                            item.field = Some(side.to_string());
                            out.push(item);
                        }
                    }
                }
            }
        }
        (a, b) => presence(
            &mut out,
            DiffSection::Encoder,
            "encoder_layout",
            a.is_some(),
            b.is_some(),
        ),
    }

    // 4. Macros.
    for i in 0..old.macros.len().max(new.macros.len()) {
        let (a, b) = (old.macros.get(i), new.macros.get(i));
        if a == b {
            continue;
        }
        let from = a.map_or_else(|| "(missing)".to_string(), |m| macro_text(&oc, m));
        let to = b.map_or_else(|| "(missing)".to_string(), |m| macro_text(&nc, m));
        let mut item = entry(DiffSection::Macro, from.clone(), to.clone());
        item.line = format!("M{i}: {from} -> {to}");
        item.index = Some(i);
        out.push(item);
    }

    // 5. Tap dances.
    match (&old.tap_dance, &new.tap_dance) {
        (Some(ot), Some(nt)) => {
            for i in 0..ot.len().min(nt.len()) {
                let (o, n) = (ot[i], nt[i]);
                let fields = [
                    (
                        "tap",
                        o.on_tap != n.on_tap,
                        oc.wire_name(o.on_tap),
                        nc.wire_name(n.on_tap),
                    ),
                    (
                        "hold",
                        o.on_hold != n.on_hold,
                        oc.wire_name(o.on_hold),
                        nc.wire_name(n.on_hold),
                    ),
                    (
                        "double_tap",
                        o.on_double_tap != n.on_double_tap,
                        oc.wire_name(o.on_double_tap),
                        nc.wire_name(n.on_double_tap),
                    ),
                    (
                        "tap_hold",
                        o.on_tap_hold != n.on_tap_hold,
                        oc.wire_name(o.on_tap_hold),
                        nc.wire_name(n.on_tap_hold),
                    ),
                    (
                        "term",
                        o.tapping_term != n.tapping_term,
                        o.tapping_term.to_string(),
                        n.tapping_term.to_string(),
                    ),
                ];
                for (field, changed, from, to) in fields {
                    if changed {
                        let mut item = entry(DiffSection::TapDance, from.clone(), to.clone());
                        item.line = format!("TD({i}) {field}: {from} -> {to}");
                        item.index = Some(i);
                        item.field = Some(field.to_string());
                        out.push(item);
                    }
                }
            }
        }
        (a, b) => presence(
            &mut out,
            DiffSection::TapDance,
            "tap_dance",
            a.is_some(),
            b.is_some(),
        ),
    }

    // 6. Combos.
    match (&old.combo, &new.combo) {
        (Some(oc_list), Some(nc_list)) => {
            for i in 0..oc_list.len().min(nc_list.len()) {
                if oc_list[i] != nc_list[i] {
                    let from = combo_text(&oc, &oc_list[i]);
                    let to = combo_text(&nc, &nc_list[i]);
                    let mut item = entry(DiffSection::Combo, from.clone(), to.clone());
                    item.line = format!("combo {i}: {from} -> {to}");
                    item.index = Some(i);
                    out.push(item);
                }
            }
        }
        (a, b) => presence(
            &mut out,
            DiffSection::Combo,
            "combo",
            a.is_some(),
            b.is_some(),
        ),
    }

    // 7. Key overrides.
    match (&old.key_override, &new.key_override) {
        (Some(ok_list), Some(nk_list)) => {
            for i in 0..ok_list.len().min(nk_list.len()) {
                let (o, n) = (&ok_list[i], &nk_list[i]);
                for (field, changed, from, to) in key_override_fields(&oc, &nc, o, n) {
                    if changed {
                        let mut item = entry(DiffSection::KeyOverride, from.clone(), to.clone());
                        item.line = format!("key_override {i} {field}: {from} -> {to}");
                        item.index = Some(i);
                        item.field = Some(field.to_string());
                        out.push(item);
                    }
                }
            }
        }
        (a, b) => presence(
            &mut out,
            DiffSection::KeyOverride,
            "key_override",
            a.is_some(),
            b.is_some(),
        ),
    }

    // 8. Alt repeat keys.
    match (&old.alt_repeat_key, &new.alt_repeat_key) {
        (Some(oa), Some(na)) => {
            for i in 0..oa.len().min(na.len()) {
                let (o, n) = (&oa[i], &na[i]);
                for (field, changed, from, to) in alt_repeat_fields(&oc, &nc, o, n) {
                    if changed {
                        let mut item = entry(DiffSection::AltRepeatKey, from.clone(), to.clone());
                        item.line = format!("alt_repeat_key {i} {field}: {from} -> {to}");
                        item.index = Some(i);
                        item.field = Some(field.to_string());
                        out.push(item);
                    }
                }
            }
        }
        (a, b) => presence(
            &mut out,
            DiffSection::AltRepeatKey,
            "alt_repeat_key",
            a.is_some(),
            b.is_some(),
        ),
    }

    // 9. Settings.
    match (&old.settings, &new.settings) {
        (Some(os), Some(ns)) => {
            for s in os {
                let before = s.value.to_string();
                let after = ns
                    .iter()
                    .find(|x| x.qsid == s.qsid)
                    .map(|x| x.value.to_string());
                if after.as_deref() != Some(before.as_str()) {
                    let to = after.unwrap_or_else(|| "(none)".to_string());
                    push_setting(&mut out, &s.qsid, before, to);
                }
            }
            for s in ns {
                if !os.iter().any(|x| x.qsid == s.qsid) {
                    push_setting(&mut out, &s.qsid, "(none)".to_string(), s.value.to_string());
                }
            }
        }
        (a, b) => presence(
            &mut out,
            DiffSection::Settings,
            "settings",
            a.is_some(),
            b.is_some(),
        ),
    }

    out
}

/// One line per changed item: its text, joined by newlines, with a final newline.
pub fn format_diff(entries: &[DiffEntry]) -> String {
    if entries.is_empty() {
        return String::new();
    }
    let mut s = entries
        .iter()
        .map(|e| e.line.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    s.push('\n');
    s
}

fn entry(section: DiffSection, old: String, new: String) -> DiffEntry {
    DiffEntry {
        section,
        layer: None,
        row: None,
        col: None,
        key_id: None,
        index: None,
        field: None,
        old,
        new,
        line: String::new(),
    }
}

fn slot_text(catalog: &Catalog, slot: Slot) -> String {
    match slot {
        Slot::Absent => "(none)".to_string(),
        Slot::Key(kc) => catalog.wire_name(kc),
    }
}

/// A section that exists in only one document: "{key}: (absent) -> present" and the reverse.
fn presence(out: &mut Vec<DiffEntry>, section: DiffSection, key: &str, old: bool, new: bool) {
    if old == new {
        return;
    }
    let from = if old { "present" } else { "(absent)" };
    let to = if new { "present" } else { "(absent)" };
    let mut item = entry(section, from.to_string(), to.to_string());
    item.line = format!("{key}: {from} -> {to}");
    out.push(item);
}

fn push_setting(out: &mut Vec<DiffEntry>, qsid: &str, from: String, to: String) {
    let mut item = entry(DiffSection::Settings, from.clone(), to.clone());
    item.line = format!("setting {qsid}: {from} -> {to}");
    item.field = Some(qsid.to_string());
    out.push(item);
}

fn join_keys(name: &str, catalog: &Catalog, keys: &[Keycode]) -> String {
    let mut parts = vec![name.to_string()];
    parts.extend(keys.iter().map(|kc| catalog.wire_name(*kc)));
    parts.join(" ")
}

fn macro_text(catalog: &Catalog, m: &Macro) -> String {
    if m.actions.is_empty() {
        return "(empty)".to_string();
    }
    m.actions
        .iter()
        .map(|action| match action {
            MacroAction::Tap(keys) => join_keys("tap", catalog, keys),
            MacroAction::Down(keys) => join_keys("down", catalog, keys),
            MacroAction::Up(keys) => join_keys("up", catalog, keys),
            MacroAction::Text(text) => {
                let mut s = String::from("text ");
                write_py_string(&mut s, text);
                s
            }
            MacroAction::Delay(ms) => format!("delay {ms}"),
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn combo_text(catalog: &Catalog, combo: &Combo) -> String {
    if combo.keys.iter().all(|kc| *kc == Keycode::NO) && combo.output == Keycode::NO {
        return "(empty)".to_string();
    }
    let mut inputs: Vec<Keycode> = combo.keys.to_vec();
    while inputs.last() == Some(&Keycode::NO) {
        inputs.pop();
    }
    let left = if inputs.is_empty() {
        "(none)".to_string()
    } else {
        inputs
            .iter()
            .map(|kc| catalog.wire_name(*kc))
            .collect::<Vec<_>>()
            .join(" + ")
    };
    format!("{left} => {}", catalog.wire_name(combo.output))
}

/// (field, changed, old text, new text) for every field of a key override.
fn key_override_fields(
    oc: &Catalog,
    nc: &Catalog,
    o: &KeyOverride,
    n: &KeyOverride,
) -> Vec<(&'static str, bool, String, String)> {
    vec![
        (
            "trigger",
            o.trigger != n.trigger,
            oc.wire_name(o.trigger),
            nc.wire_name(n.trigger),
        ),
        (
            "replacement",
            o.replacement != n.replacement,
            oc.wire_name(o.replacement),
            nc.wire_name(n.replacement),
        ),
        (
            "layers",
            o.layers != n.layers,
            o.layers.to_string(),
            n.layers.to_string(),
        ),
        (
            "trigger_mods",
            o.trigger_mods != n.trigger_mods,
            o.trigger_mods.to_string(),
            n.trigger_mods.to_string(),
        ),
        (
            "negative_mod_mask",
            o.negative_mod_mask != n.negative_mod_mask,
            o.negative_mod_mask.to_string(),
            n.negative_mod_mask.to_string(),
        ),
        (
            "suppressed_mods",
            o.suppressed_mods != n.suppressed_mods,
            o.suppressed_mods.to_string(),
            n.suppressed_mods.to_string(),
        ),
        (
            "options",
            o.options != n.options,
            o.options.to_string(),
            n.options.to_string(),
        ),
    ]
}

/// (field, changed, old text, new text) for every field of an alt repeat key.
fn alt_repeat_fields(
    oc: &Catalog,
    nc: &Catalog,
    o: &AltRepeatKey,
    n: &AltRepeatKey,
) -> Vec<(&'static str, bool, String, String)> {
    vec![
        (
            "keycode",
            o.keycode != n.keycode,
            oc.wire_name(o.keycode),
            nc.wire_name(n.keycode),
        ),
        (
            "alt_keycode",
            o.alt_keycode != n.alt_keycode,
            oc.wire_name(o.alt_keycode),
            nc.wire_name(n.alt_keycode),
        ),
        (
            "allowed_mods",
            o.allowed_mods != n.allowed_mods,
            o.allowed_mods.to_string(),
            n.allowed_mods.to_string(),
        ),
        (
            "options",
            o.options != n.options,
            o.options.to_string(),
            n.options.to_string(),
        ),
    ]
}
