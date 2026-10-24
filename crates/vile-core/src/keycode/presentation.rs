//! User-facing names and picker categories. Wire names remain unchanged.
use super::{Catalog, Group, Keycode, KeycodeKind, table};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickerCategory {
    Basic,
    Symbols,
    Numbers,
    Modifiers,
    Layers,
    Mouse,
    Media,
    Macros,
    Quantum,
    Special,
}
impl PickerCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Basic => "Basic",
            Self::Symbols => "Symbols",
            Self::Numbers => "Numbers and F-keys",
            Self::Modifiers => "Modifiers",
            Self::Layers => "Layers",
            Self::Mouse => "Mouse",
            Self::Media => "Media",
            Self::Macros => "Macros",
            Self::Quantum => "Quantum",
            Self::Special => "Special",
        }
    }
    pub fn for_entry(group: Group, code: u16) -> Self {
        match code {
            0..=1 => return Self::Special,
            0x1e..=0x27 | 0x3a..=0x45 | 0x68..=0x73 => return Self::Numbers,
            0x2d..=0x38 => return Self::Symbols,
            0xcd..=0xdf => return Self::Mouse,
            _ => {}
        }
        match group {
            Group::Special => Self::Special,
            Group::Basic => Self::Basic,
            Group::Shifted | Group::Iso => Self::Symbols,
            Group::Modifiers | Group::Mods | Group::ModsHex | Group::ModTap | Group::ModTapHex => {
                Self::Modifiers
            }
            Group::Layers => Self::Layers,
            Group::Media => Self::Media,
            Group::MacroBase => Self::Macros,
            _ => Self::Quantum,
        }
    }
}

pub fn mods_description(mods: u8) -> String {
    let side = if mods & 16 != 0 { "Right" } else { "Left" };
    let names: Vec<_> = [(1, "Ctrl"), (2, "Shift"), (4, "Alt"), (8, "Gui")]
        .into_iter()
        .filter(|(bit, _)| mods & bit != 0)
        .map(|(_, name)| format!("{side} {name}"))
        .collect();
    names.join(" + ")
}

impl Catalog {
    pub fn category(&self, code: Keycode) -> PickerCategory {
        match code.kind() {
            KeycodeKind::Macro { .. } => PickerCategory::Macros,
            KeycodeKind::Layer { .. }
            | KeycodeKind::LayerTap { .. }
            | KeycodeKind::LayerMod { .. } => PickerCategory::Layers,
            KeycodeKind::Mods { .. }
            | KeycodeKind::ModTap { .. }
            | KeycodeKind::OneShotMod { .. } => PickerCategory::Modifiers,
            _ => table()
                .iter()
                .find(|e| e.code == code.0 && !e.group.is_function())
                .map(|e| e.category)
                .unwrap_or(PickerCategory::Quantum),
        }
    }
    pub fn description(&self, code: Keycode) -> String {
        match code.kind() {
            KeycodeKind::Mods { mods, key } => {
                let mut description = format!(
                    "{} + {}",
                    mods_description(mods),
                    self.description(Keycode(key.into()))
                );
                if mods == 2 {
                    let wire = self.wire_name(code);
                    if let Some((symbol, _)) = super::catalog::SYMBOLS
                        .iter()
                        .find(|(_, name)| *name == wire)
                    {
                        description.push_str(&format!(" ({symbol}, US layout)"));
                    }
                }
                description
            }
            KeycodeKind::ModTap { mods, key } => format!(
                "Hold {}, tap {}",
                mods_description(mods),
                self.description(Keycode(key.into()))
            ),
            KeycodeKind::LayerTap { layer, key } => format!(
                "Hold layer {layer}, tap {}",
                self.description(Keycode(key.into()))
            ),
            KeycodeKind::Layer { op, layer } => {
                let action = match op {
                    super::LayerOp::Mo => "Hold layer",
                    super::LayerOp::To => "Switch to layer",
                    super::LayerOp::Tg => "Toggle layer",
                    super::LayerOp::Tt => "Tap-toggle layer",
                    super::LayerOp::Osl => "One-shot layer",
                    super::LayerOp::Df => "Set default layer",
                    super::LayerOp::Pdf => "Persist default layer",
                };
                format!("{action} {layer}")
            }
            KeycodeKind::OneShotMod { mods } => format!("One-shot {}", mods_description(mods)),
            KeycodeKind::LayerMod { layer, mods } => {
                format!("Hold layer {layer} with {}", mods_description(mods))
            }
            KeycodeKind::Macro { index } => format!("Macro {index}"),
            KeycodeKind::TapDance { index } => format!("Tap dance {index}"),
            KeycodeKind::User { index } => format!("User key {index}"),
            _ => self.plain_description(code),
        }
    }
    fn plain_description(&self, code: Keycode) -> String {
        let known = match code.0 {
            0 => "No key",
            1 => "Transparent (falls through to the layer below)",
            0x28 => "Enter",
            0x29 => "Escape",
            0x2a => "Backspace",
            0x2b => "Tab",
            0x2c => "Space",
            0x2d => "Minus (-, US layout)",
            0x2e => "Equal (=, US layout)",
            0x2f => "Left bracket ([, US layout)",
            0x30 => "Right bracket (], US layout)",
            0x31 => "Backslash (US layout)",
            0x33 => "Semicolon (;, US layout)",
            0x34 => "Quote (US layout)",
            0x35 => "Grave (US layout)",
            0x36 => "Comma (US layout)",
            0x37 => "Period (US layout)",
            0x38 => "Slash (US layout)",
            0x39 => "Caps Lock",
            0x46 => "Print Screen",
            0x47 => "Scroll Lock",
            0x48 => "Pause",
            0x49 => "Insert",
            0x4a => "Home",
            0x4b => "Page Up",
            0x4c => "Delete",
            0x4d => "End",
            0x4e => "Page Down",
            0x4f => "Right arrow",
            0x50 => "Left arrow",
            0x51 => "Down arrow",
            0x52 => "Up arrow",
            0x53 => "Num Lock",
            0x54 => "Keypad divide (/)",
            0x55 => "Keypad multiply (*)",
            0x56 => "Keypad subtract (-)",
            0x57 => "Keypad add (+)",
            0x58 => "Keypad Enter",
            0x63 => "Keypad decimal (.)",
            0x65 => "Application menu",
            0x67 => "Keypad equal (=)",
            0x85 => "Keypad comma (,)",
            0xe0 => "Left Ctrl",
            0xe1 => "Left Shift",
            0xe2 => "Left Alt",
            0xe3 => "Left Gui",
            0xe4 => "Right Ctrl",
            0xe5 => "Right Shift",
            0xe6 => "Right Alt",
            0xe7 => "Right Gui",
            _ => "",
        };
        if !known.is_empty() {
            return known.into();
        }
        if (4..=29).contains(&code.0) {
            return char::from(b'A' + (code.0 - 4) as u8).to_string();
        }
        if (0x59..=0x62).contains(&code.0) {
            return format!("Keypad {}", (code.0 - 0x58) % 10);
        }
        let wire = self.wire_name(code);
        let named = match wire.as_str() {
            "KC_VOLU" => "Volume up",
            "KC_VOLD" => "Volume down",
            "KC_MUTE" => "Mute",
            "KC_MPLY" => "Play or pause",
            "KC_MNXT" => "Next track",
            "KC_MPRV" => "Previous track",
            "KC_MSTP" => "Stop media",
            "KC_BTN1" => "Mouse button 1 (left click)",
            "KC_BTN2" => "Mouse button 2 (right click)",
            "KC_BTN3" => "Mouse button 3 (middle click)",
            "KC_MS_U" => "Mouse up",
            "KC_MS_D" => "Mouse down",
            "KC_MS_L" => "Mouse left",
            "KC_MS_R" => "Mouse right",
            "KC_WH_U" => "Mouse wheel up",
            "KC_WH_D" => "Mouse wheel down",
            "KC_WH_L" => "Mouse wheel left",
            "KC_WH_R" => "Mouse wheel right",
            "QK_REPEAT_KEY" => "Repeat last key",
            "QK_ALT_REPEAT_KEY" => "Alternate repeat",
            "QK_BOOT" => "Enter bootloader",
            "QK_CAPS_WORD_TOGGLE" => "Toggle Caps Word",
            "QK_LAYER_LOCK" => "Layer lock",
            _ => "",
        };
        if !named.is_empty() {
            return named.into();
        }
        if wire.starts_with("0x") {
            return format!("Unknown keycode {wire}");
        }
        wire.trim_start_matches("KC_")
            .trim_start_matches("QK_")
            .replace('_', " ")
            .to_lowercase()
    }
}
