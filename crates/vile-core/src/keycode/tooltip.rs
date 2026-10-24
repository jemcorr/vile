//! Behavioral explanations shared by board keys, picker tiles and field editors.
//! These are deliberately separate from compact inspector and diff descriptions.
use super::{
    Catalog, Keycode, KeycodeKind, LayerOp, picker::action_description,
    presentation::mods_description,
};

const LAYER_PRIORITY: &str =
    "Higher active layers take priority; transparent keys fall through to lower layers.";
const TAP_HOLD: &str = "The firmware's tapping term and tap-hold settings decide whether a press counts as a tap or a hold.";
const ONE_SHOT: &str = "Firmware settings can add a timeout or repeated-tap locking. If one-shot support is disabled at runtime, this acts as a normal held key.";

impl Catalog {
    /// Full behavior for hover text. This never replaces the compact description.
    pub fn tooltip(&self, code: Keycode) -> String {
        match code.kind() {
            KeycodeKind::Mods { mods, key } => format!(
                "Send {} together with {} while held. Release this key to release the key and its modifiers. The host layout and application determine the resulting character or shortcut.",
                self.description(Keycode(key.into())),
                mods_description(mods)
            ),
            KeycodeKind::ModTap { mods, key } => format!(
                "Tap to send {}. Hold to keep {} pressed until release. {TAP_HOLD}",
                self.description(Keycode(key.into())),
                mods_description(mods)
            ),
            KeycodeKind::LayerTap { layer, key } => format!(
                "Tap to send {}. Hold to activate layer {layer}, then release to deactivate it. {TAP_HOLD} {LAYER_PRIORITY}",
                self.description(Keycode(key.into()))
            ),
            KeycodeKind::Layer { op, layer } => {
                let action = match op {
                    LayerOp::Mo => {
                        format!("Hold to activate layer {layer}; release to deactivate it.")
                    }
                    LayerOp::To => format!(
                        "On press, activate layer {layer} and deactivate every other non-default layer. The default layer remains active. This is a runtime switch, not a saved default."
                    ),
                    LayerOp::Tg => format!(
                        "On press, toggle layer {layer} on or off. It remains in that state after release, until another layer action or a keyboard restart changes it. This does not save a default layer."
                    ),
                    LayerOp::Tt => format!(
                        "Hold to activate layer {layer}; release to deactivate it. Repeated taps toggle it on or off. The tap count is set by the firmware, commonly five; tap timing and quick-tap settings also affect this behavior. The toggle is not saved across restart."
                    ),
                    LayerOp::Osl => format!(
                        "Tap to activate layer {layer} for the next keypress, then return to the previous layers. Hold it to use that layer until release. {ONE_SHOT}"
                    ),
                    LayerOp::Df => format!(
                        "Set layer {layer} as the default base layer. This changes the base underneath other active layers and lasts only until the keyboard restarts or another default-layer action changes it."
                    ),
                    LayerOp::Pdf => format!(
                        "Set layer {layer} as the default base layer and save it in the keyboard's persistent memory. It survives restarts and power loss. Requires persistent default-layer support."
                    ),
                };
                format!("{action} {LAYER_PRIORITY}")
            }
            KeycodeKind::OneShotMod { mods } => format!(
                "Tap to apply {} to the next keypress without keeping this key held. Hold it to use those modifiers normally until release. {ONE_SHOT}",
                mods_description(mods)
            ),
            KeycodeKind::LayerMod { layer, mods } => format!(
                "Hold to activate layer {layer} and keep {} pressed. Release to deactivate the layer and release those modifiers. {LAYER_PRIORITY}",
                mods_description(mods)
            ),
            KeycodeKind::Macro { index } => format!(
                "On press, play the configured key actions, text and delays in stored macro slot {index}. Its output depends on that slot's contents. An empty slot sends nothing. Stored Vial macros are separate from the temporary recordings made by Dynamic Macro keys. Requires firmware macro support."
            ),
            KeycodeKind::TapDance { index } => format!(
                "Run tap dance slot {index}. Its configured single tap, hold, double tap and tap-then-hold actions determine the output. In Vial, an unset hold falls back to the tap key, and an unset double tap repeats the tap key twice; a completely empty slot does nothing. The slot's tapping term and firmware tap-dance rules decide when each action runs. Requires Tap Dance support."
            ),
            KeycodeKind::User { index } => format!(
                "Run user keycode {index}. Its behavior is defined by this keyboard's firmware, so Vile cannot infer its output, timing or persistence from the numeric code. Consult the keyboard's firmware documentation."
            ),
            _ => self.plain_tooltip(code),
        }
    }

    /// Explain an incomplete search template without pretending its tap key exists.
    pub fn template_tooltip(&self, insert: &str) -> Option<String> {
        let code = self.parse(&format!("{insert}KC_NO)")).ok()?;
        Some(match code.kind() {
            KeycodeKind::Mods { mods, .. } => format!(
                "Choose a basic key to send together with {} while held. The host determines the resulting shortcut or character.",
                mods_description(mods)
            ),
            KeycodeKind::ModTap { mods, .. } => format!(
                "Choose a basic tap key. Tap sends that key; hold keeps {} pressed until release. {TAP_HOLD}",
                mods_description(mods)
            ),
            KeycodeKind::LayerTap { layer, .. } => format!(
                "Choose a basic tap key. Tap sends that key; hold activates layer {layer} until release. {TAP_HOLD} {LAYER_PRIORITY}"
            ),
            _ => return None,
        })
    }

    fn plain_tooltip(&self, code: Keycode) -> String {
        let wire = self.wire_name(code);
        if let Some(description) = action_description(&wire) {
            let mut text = description.to_string();
            let note = match code.0 {
                0x7c00..=0x7c01 | 0x7c03 => {
                    "Requires support in the keyboard firmware; firmware security settings can restrict this action."
                }
                0x7000..=0x701f => {
                    "This changes the keyboard's own configuration on press and saves it across restarts. Requires the corresponding Magic feature in firmware. Gui means Command on macOS or Windows/Super on other hosts."
                }
                0x7480..=0x7482 => {
                    "Applies on press to keyboard-generated sound, not the computer's volume. Requires keyboard audio hardware and firmware support. The on/off setting is saved across restarts."
                }
                0x748a => {
                    "Applies on press. Requires firmware Audio Clicky support and enabled keyboard audio. The click on/off setting is saved across restarts."
                }
                0x748d..=0x748f => {
                    "Applies on press. Requires firmware Audio Clicky support and enabled keyboard audio. Pitch changes last until the keyboard restarts; they do not change the host volume or reset the keymap."
                }
                0x7490..=0x7493 => {
                    "Applies on press. Requires firmware Music Mode and keyboard audio. Note-producing keys may stop typing to the host while music mode is enabled. Music mode and scale changes are not saved across restart."
                }
                0x7c40..=0x7c4c => {
                    "Applies on press and saves the haptic configuration across restarts. Requires compatible haptic hardware and firmware support. This affects vibration, not the keymap."
                }
                0x7c10..=0x7c15 => {
                    "Applies on press. Requires Auto Shift support; timeout adjustment/report keys also require its setup controls. Changes are runtime settings and are not saved across restart."
                }
                0x7c50..=0x7c52 => {
                    "Combos turn a configured chord of keys into another action. Applies on press and lasts until changed or the keyboard restarts. Requires firmware Combo support; it does not erase the combo definitions."
                }
                0x7800..=0x7806 => {
                    "Applies on press to the single-color keyboard backlight, not the display. Saves the setting across restarts. Requires firmware backlight support; breathing also requires its animation support."
                }
                0x7820..=0x7833 => {
                    "Requires the matching RGB lighting feature and effect in firmware. These lighting settings are saved across restarts; they do not change the host display. Some firmware also routes these shared RGB keys to the RGB matrix."
                }
                0x7840..=0x784c => {
                    "Requires RGB Matrix firmware and hardware support. Changes are saved across restarts. Normally applies on key release; firmware can configure it to apply on press instead."
                }
                _ => "",
            };
            if !note.is_empty() {
                text.push(' ');
                text.push_str(note);
            }
            if matches!(code.0, 0x7821..=0x782a | 0x7843..=0x784c) {
                text.push_str(" Holding Shift reverses the adjustment direction in standard QMK handling; keyboard firmware can override this.");
            }
            return text;
        }
        match code.0 {
            0 => "Send no action. This blocks lower-layer bindings at this position rather than falling through.".into(),
            1 => "Use the binding at this position on the next lower active layer. Continue through transparent bindings until an action is found.".into(),
            0xe0..=0xe7 => format!("Hold to keep {} pressed; release to release it. Modifier shortcuts depend on the host and application. Gui means Command on macOS or Windows/Super on other hosts.", self.description(code)),
            _ if wire.starts_with("0x") => format!("Unknown firmware action {wire}. The numeric value is preserved; Vile cannot infer its behavior or persistence."),
            _ => format!("Send {} to the host while held, then release it on key release. The host keyboard layout and application determine the result.", self.description(code)),
        }
    }
}
