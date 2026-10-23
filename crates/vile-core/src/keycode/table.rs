//! The Vial protocol v6 keycode table: names, aliases and numeric values,
//! parsed once from data/keycodes_v6.tsv.

use std::sync::OnceLock;

/// TSV group of a table entry. Function groups name an outer keycode that
/// takes one inner basic keycode, written NAME(KC_x).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Group {
    Special,
    Basic,
    Shifted,
    Iso,
    Boot,
    Modifiers,
    Mods,
    ModTap,
    ModsHex,
    ModTapHex,
    Quantum,
    Backlight,
    Media,
    MacroBase,
    Layers,
}

/// One row of the TSV file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableEntry {
    pub group: Group,
    pub name: &'static str,
    pub code: u16,
    pub aliases: Vec<&'static str>,
}

const TSV: &str = include_str!("../../data/keycodes_v6.tsv");

static TABLE: OnceLock<Vec<TableEntry>> = OnceLock::new();

impl Group {
    /// The group name as written in the TSV file.
    pub fn as_str(self) -> &'static str {
        match self {
            Group::Special => "special",
            Group::Basic => "basic",
            Group::Shifted => "shifted",
            Group::Iso => "iso",
            Group::Boot => "boot",
            Group::Modifiers => "modifiers",
            Group::Mods => "mods",
            Group::ModTap => "modtap",
            Group::ModsHex => "mods_hex",
            Group::ModTapHex => "modtap_hex",
            Group::Quantum => "quantum",
            Group::Backlight => "backlight",
            Group::Media => "media",
            Group::MacroBase => "macro_base",
            Group::Layers => "layers",
        }
    }

    /// Inverse of `as_str`; None for an unknown name.
    pub fn from_name(s: &str) -> Option<Group> {
        Some(match s {
            "special" => Group::Special,
            "basic" => Group::Basic,
            "shifted" => Group::Shifted,
            "iso" => Group::Iso,
            "boot" => Group::Boot,
            "modifiers" => Group::Modifiers,
            "mods" => Group::Mods,
            "modtap" => Group::ModTap,
            "mods_hex" => Group::ModsHex,
            "modtap_hex" => Group::ModTapHex,
            "quantum" => Group::Quantum,
            "backlight" => Group::Backlight,
            "media" => Group::Media,
            "macro_base" => Group::MacroBase,
            "layers" => Group::Layers,
            _ => return None,
        })
    }

    /// True for Mods, ModTap, ModsHex and ModTapHex.
    pub fn is_function(self) -> bool {
        matches!(
            self,
            Group::Mods | Group::ModTap | Group::ModsHex | Group::ModTapHex
        )
    }
}

/// All entries in file order, parsed on first use.
pub fn table() -> &'static [TableEntry] {
    TABLE.get_or_init(|| parse(TSV)).as_slice()
}

fn parse(tsv: &'static str) -> Vec<TableEntry> {
    tsv.lines()
        .enumerate()
        .filter(|(_, line)| !line.is_empty())
        .map(|(index, line)| parse_line(index + 1, line))
        .collect()
}

fn parse_line(number: usize, line: &'static str) -> TableEntry {
    let fields: Vec<&'static str> = line.split('\t').collect();
    if fields.len() != 4 {
        panic!(
            "keycodes_v6.tsv line {number} has {} fields: {line}",
            fields.len()
        );
    }
    let group = Group::from_name(fields[0])
        .unwrap_or_else(|| panic!("keycodes_v6.tsv line {number}: unknown group: {line}"));
    let code = fields[2]
        .strip_prefix("0x")
        .and_then(|hex| u16::from_str_radix(hex, 16).ok())
        .unwrap_or_else(|| panic!("keycodes_v6.tsv line {number}: bad value: {line}"));
    let aliases = if fields[3] == "-" {
        Vec::new()
    } else {
        fields[3].split(',').collect()
    };
    TableEntry {
        group,
        name: fields[1],
        code,
        aliases,
    }
}
