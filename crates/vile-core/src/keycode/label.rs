//! Short key cap labels for the keyboard drawing.

use super::catalog::SYMBOLS;
use super::{Catalog, Keycode, KeycodeKind};

/// Canonical name to label. Lines of the label table in PLAN.md.
const LABELS: &[(&str, &str)] = &[
    ("KC_ENTER", "Ent"),
    ("KC_ESCAPE", "Esc"),
    ("KC_BSPACE", "Bspc"),
    ("KC_TAB", "Tab"),
    ("KC_SPACE", "Spc"),
    ("KC_MINUS", "-"),
    ("KC_EQUAL", "="),
    ("KC_LBRACKET", "["),
    ("KC_RBRACKET", "]"),
    ("KC_BSLASH", "\\"),
    ("KC_NONUS_HASH", "#"),
    ("KC_SCOLON", ";"),
    ("KC_QUOTE", "'"),
    ("KC_GRAVE", "`"),
    ("KC_COMMA", ","),
    ("KC_DOT", "."),
    ("KC_SLASH", "/"),
    ("KC_CAPSLOCK", "Caps"),
    ("KC_PSCREEN", "PrtSc"),
    ("KC_SCROLLLOCK", "ScrLk"),
    ("KC_PAUSE", "Pause"),
    ("KC_INSERT", "Ins"),
    ("KC_HOME", "Home"),
    ("KC_PGUP", "PgUp"),
    ("KC_DELETE", "Del"),
    ("KC_END", "End"),
    ("KC_PGDOWN", "PgDn"),
    ("KC_RIGHT", "→"),
    ("KC_LEFT", "←"),
    ("KC_DOWN", "↓"),
    ("KC_UP", "↑"),
    ("KC_APPLICATION", "Menu"),
    ("KC_NUMLOCK", "NumLk"),
    ("KC_KP_SLASH", "Num /"),
    ("KC_KP_ASTERISK", "Num *"),
    ("KC_KP_MINUS", "Num -"),
    ("KC_KP_PLUS", "Num +"),
    ("KC_KP_ENTER", "Num Ent"),
    ("KC_KP_0", "Num 0"),
    ("KC_KP_1", "Num 1"),
    ("KC_KP_2", "Num 2"),
    ("KC_KP_3", "Num 3"),
    ("KC_KP_4", "Num 4"),
    ("KC_KP_5", "Num 5"),
    ("KC_KP_6", "Num 6"),
    ("KC_KP_7", "Num 7"),
    ("KC_KP_8", "Num 8"),
    ("KC_KP_9", "Num 9"),
    ("KC_KP_DOT", "Num ."),
    ("KC_KP_EQUAL", "Num ="),
    ("KC_KP_COMMA", "Num ,"),
    ("KC_LCTRL", "⌃"),
    ("KC_LSHIFT", "⇧"),
    ("KC_LALT", "⌥"),
    ("KC_LGUI", "⌘"),
    ("KC_RCTRL", "R⌃"),
    ("KC_RSHIFT", "R⇧"),
    ("KC_RALT", "R⌥"),
    ("KC_RGUI", "R⌘"),
    ("KC_MPLY", "Play"),
    ("KC_MNXT", "Next"),
    ("KC_MPRV", "Prev"),
    ("KC_MSTP", "Stop"),
    ("KC_MUTE", "Mute"),
    ("KC_VOLU", "Vol+"),
    ("KC_VOLD", "Vol-"),
    ("KC_BRIU", "Bri+"),
    ("KC_BRID", "Bri-"),
    ("KC_MS_U", "Mouse\n↑"),
    ("KC_MS_D", "Mouse\n↓"),
    ("KC_MS_L", "Mouse\n←"),
    ("KC_MS_R", "Mouse\n→"),
    ("KC_WH_U", "Wheel\n↑"),
    ("KC_WH_D", "Wheel\n↓"),
    ("KC_WH_L", "Wheel\n←"),
    ("KC_WH_R", "Wheel\n→"),
    ("KC_BTN1", "Left\nclick"),
    ("KC_BTN2", "Right\nclick"),
    ("KC_BTN3", "Middle\nclick"),
    ("KC_BTN4", "Mouse\n4"),
    ("KC_BTN5", "Mouse\n5"),
    ("KC_ACL0", "Speed\n0"),
    ("KC_ACL1", "Speed\n1"),
    ("KC_ACL2", "Speed\n2"),
    ("QK_REPEAT_KEY", "Rep"),
    ("QK_ALT_REPEAT_KEY", "ARep"),
    ("KC_GESC", "Esc~"),
    ("QK_BOOT", "Boot"),
    ("QK_CAPS_WORD_TOGGLE", "CapsW"),
    ("QK_LAYER_LOCK", "LLock"),
];

/// "⌃", "⇧", "⌥", "⌘" for the modifier bits, "R" first when 0x10 is set.
fn mods_short(mods: u8) -> String {
    let mut s = String::new();
    if mods & 0x10 != 0 {
        s.push('R');
    }
    if mods & 0x01 != 0 {
        s.push('⌃');
    }
    if mods & 0x02 != 0 {
        s.push('⇧');
    }
    if mods & 0x04 != 0 {
        s.push('⌥');
    }
    if mods & 0x08 != 0 {
        s.push('⌘');
    }
    s
}

impl Catalog {
    /// Short key cap label; lines separated by '\n'.
    pub fn label(&self, kc: Keycode) -> String {
        match kc.kind() {
            KeycodeKind::Basic { .. } => match kc.code() {
                0x0000 => String::new(),
                0x0001 => "▽".to_string(),
                code => self.basic_label(code),
            },
            KeycodeKind::Mods { mods, key } => {
                if mods == 0x02 {
                    let target = format!("LSFT({})", self.wire_name(Keycode(u16::from(key))));
                    if let Some((symbol, _)) = SYMBOLS.iter().find(|(_, name)| *name == target) {
                        return symbol.to_string();
                    }
                }
                format!("{}{}", mods_short(mods), self.basic_label(u16::from(key)))
            }
            KeycodeKind::ModTap { mods, key } => {
                format!("{}\n{}", self.basic_label(u16::from(key)), mods_short(mods))
            }
            KeycodeKind::LayerTap { layer, key } => {
                format!("{}\nL{layer}", self.basic_label(u16::from(key)))
            }
            KeycodeKind::Layer { op, layer } => format!("{}{layer}", op.name()),
            KeycodeKind::OneShotMod { mods } => format!("OSM\n{}", mods_short(mods)),
            KeycodeKind::LayerMod { layer, mods } => {
                format!("LM{layer}\n{}", mods_short(mods))
            }
            KeycodeKind::TapDance { index } => format!("TD{index}"),
            KeycodeKind::Macro { index } => format!("M{index}"),
            KeycodeKind::User { index } => format!("U{index}"),
            KeycodeKind::Other => self.basic_label(kc.code()),
        }
    }

    /// The label table entry for the code, else its wire name without KC_ or QK_.
    fn basic_label(&self, code: u16) -> String {
        let wire = self.wire_name(Keycode(code));
        if let Some((_, label)) = LABELS.iter().find(|(name, _)| *name == wire) {
            return (*label).to_string();
        }
        let bare = wire
            .strip_prefix("KC_")
            .or_else(|| wire.strip_prefix("QK_"))
            .unwrap_or(wire.as_str());
        bare.to_string()
    }
}
