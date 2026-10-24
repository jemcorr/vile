//! Picker copy is separate from the compact labels drawn on the keyboard.
//!
//! Copy sources: https://docs.qmk.fm/keycodes and https://docs.qmk.fm/keycodes_magic,
//! https://docs.qmk.fm/features/audio, https://docs.qmk.fm/features/haptic_feedback,
//! https://docs.qmk.fm/features/tri_layer, and Vial QMK's process_space_cadet.c.
//! Legacy names: vial-kb/vial-gui src/main/python/keycodes/keycodes.py.
//! Numeric identity stays in keycodes_v6.tsv.
use std::{collections::HashMap, sync::OnceLock};

use super::{Catalog, Keycode, KeycodeKind, PickerCategory};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PickerPresentation {
    pub label: String,
    pub summary: String,
    pub group: &'static str,
}

struct PickerCopy {
    label: &'static str,
    summary: &'static str,
    description: &'static str,
}

fn picker_copy(wire: &str) -> Option<&'static PickerCopy> {
    static COPY: OnceLock<HashMap<&'static str, PickerCopy>> = OnceLock::new();
    COPY.get_or_init(|| {
        include_str!("../../data/picker.tsv")
            .lines()
            .map(|line| {
                let fields: Vec<_> = line.split('\t').collect();
                assert_eq!(fields.len(), 4, "invalid picker copy: {line}");
                (
                    fields[0],
                    PickerCopy {
                        label: fields[1],
                        summary: fields[2],
                        description: fields[3],
                    },
                )
            })
            .collect()
    })
    .get(wire)
}

impl Catalog {
    /// Full picker tooltip text. Keep inspector headings and diff names compact.
    pub fn picker_description(&self, code: Keycode) -> String {
        picker_copy(&self.wire_name(code))
            .map(|copy| copy.description.to_string())
            .unwrap_or_else(|| self.description(code))
    }

    pub fn picker(&self, code: Keycode) -> PickerPresentation {
        let wire = self.wire_name(code);
        let group = self.picker_group(code);
        if let Some(copy) = picker_copy(&wire) {
            return PickerPresentation {
                label: copy.label.into(),
                summary: copy.summary.into(),
                group,
            };
        }
        let description = self.description(code);
        let label = match code.0 {
            0 => "No key".into(),
            1 => "Transparent".into(),
            _ => self.label(code).replace('\n', " "),
        };
        let summary = match code.kind() {
            KeycodeKind::Layer { op, .. } => match op {
                super::LayerOp::Mo => "While held",
                super::LayerOp::To => "Switch layer",
                super::LayerOp::Tg => "Toggle on / off",
                super::LayerOp::Tt => "Hold / tap-toggle",
                super::LayerOp::Osl => "Next keypress",
                super::LayerOp::Df => "Default until reset",
                super::LayerOp::Pdf => "Save default",
            }
            .into(),
            KeycodeKind::OneShotMod { .. } => "Next keypress".into(),
            KeycodeKind::Macro { .. } => "Play macro".into(),
            KeycodeKind::TapDance { .. } => "Tap / hold action".into(),
            KeycodeKind::User { .. } => "Firmware-defined".into(),
            _ => match code.0 {
                0 => "No action".into(),
                1 => "Use lower layer".into(),
                4..=29 => "Letter".into(),
                0x1e..=0x27 => "Number".into(),
                0x3a..=0x45 | 0x68..=0x73 => "Function key".into(),
                0x2d..=0x38 => description
                    .split(" (")
                    .next()
                    .unwrap_or(&description)
                    .to_string(),
                0x100..=0x1fff => "Shifted symbol".into(),
                _ => description
                    .split(" (US layout)")
                    .next()
                    .unwrap_or(&description)
                    .to_string(),
            },
        };
        PickerPresentation {
            label,
            summary,
            group,
        }
    }

    fn picker_group(&self, code: Keycode) -> &'static str {
        match code.kind() {
            KeycodeKind::Layer { op, .. } => {
                return match op {
                    super::LayerOp::Mo => "Momentary layers",
                    super::LayerOp::To => "Switch layers",
                    super::LayerOp::Tg => "Toggle layers",
                    super::LayerOp::Tt => "Tap-toggle layers",
                    super::LayerOp::Osl => "One-shot layers",
                    super::LayerOp::Df => "Default layers",
                    super::LayerOp::Pdf => "Persistent default layers",
                };
            }
            KeycodeKind::OneShotMod { .. } => return "One-shot modifiers",
            KeycodeKind::Macro { .. } => return "Macro slots",
            KeycodeKind::TapDance { .. } => return "Tap dances",
            KeycodeKind::User { .. } => return "User keycodes",
            _ => {}
        }
        match self.category(code) {
            PickerCategory::Basic => match code.0 {
                4..=29 => "Letters",
                0x49..=0x52 => "Navigation",
                _ => "Editing and locks",
            },
            PickerCategory::Numbers => match code.0 {
                0x1e..=0x27 => "Number row",
                0x3a..=0x45 | 0x68..=0x73 => "Function keys",
                _ => "Keypad",
            },
            PickerCategory::Symbols => match code.0 {
                0x100..=0x1fff => "Shifted symbols · US layout",
                _ => "Punctuation · US layout",
            },
            PickerCategory::Modifiers => match code.0 {
                0xe0..=0xe3 => "Left modifiers",
                0xe4..=0xe7 => "Right modifiers",
                _ => "Dual-purpose modifiers",
            },
            PickerCategory::Mouse => match code.0 {
                0xcd..=0xd0 => "Pointer movement",
                0xd1..=0xd5 => "Mouse buttons",
                0xd9..=0xdc => "Mouse wheel",
                _ => "Mouse acceleration",
            },
            PickerCategory::Media => match code.0 {
                0xa8..=0xae | 0xb0 | 0xbb..=0xbc | 0x80..=0x81 => "Playback and volume",
                0xb4..=0xba => "Browser",
                0xa5..=0xa7 | 0xbd..=0xbe => "Power and display",
                0xaf | 0xb1..=0xb3 => "Launch applications",
                0x82..=0x84 => "Mechanical locking keys",
                _ => "Application actions",
            },
            PickerCategory::Macros => "Dynamic recording",
            PickerCategory::Layers => "Layer helpers",
            PickerCategory::Special => "Blank and transparent",
            PickerCategory::Quantum => match code.0 {
                0x7c00..=0x7c03 => "Firmware and reset",
                0x7000..=0x7010 | 0x7014..=0x701d => "Key remapping",
                0x7011..=0x7013 => "N-key rollover",
                0x701e..=0x701f => "Split handedness",
                0x7480..=0x748f => "Audio and click sounds",
                0x7490..=0x7493 => "Music mode",
                0x7c40..=0x7c4c => "Haptic feedback",
                0x7c10..=0x7c15 => "Auto Shift",
                0x7c50..=0x7c52 | 0x7c73 | 0x7c79..=0x7c7a => "Typing helpers",
                0x7800..=0x7806 => "Backlight",
                0x7820..=0x782a => "RGB lighting",
                0x782b..=0x7833 => "RGB lighting presets",
                0x7840..=0x784c => "RGB matrix",
                _ => "Firmware actions",
            },
        }
    }
}
