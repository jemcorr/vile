//! Document validation: warnings for references that Vial cannot resolve.
use crate::document::{Document, LoadIssue, MacroAction};
use crate::keycode::KeycodeKind;

/// Warnings in the order of Document::keycodes(), then macro text warnings.
pub fn validate(doc: &Document) -> Vec<LoadIssue> {
    let layer_count = doc.layer_count();
    let macro_count = doc.macros.len();
    let tap_dance_count = doc.tap_dance.as_ref().map_or(0, Vec::len);
    let mut out = Vec::new();
    let mut warn = |path: String, message: String| out.push(LoadIssue { path, message });

    for (path, kc) in doc.keycodes() {
        if let Some(layer) = kc.layer_ref()
            && usize::from(layer) >= layer_count
        {
            warn(
                path.clone(),
                format!("references layer {layer} but the file has {layer_count} layers"),
            );
        }
        match kc.kind() {
            KeycodeKind::Macro { index } if usize::from(index) >= macro_count => warn(
                path,
                format!("references macro M{index} but the file has {macro_count} macro slots"),
            ),
            KeycodeKind::TapDance { index } if usize::from(index) >= tap_dance_count => warn(
                path,
                format!(
                    "references tap dance TD({index}) but the file has {tap_dance_count} tap dances"
                ),
            ),
            KeycodeKind::User { index } if index >= 16 => warn(
                path,
                format!("USER{index:02} is not a Vial user keycode (Vial knows USER00 to USER15)"),
            ),
            _ => {}
        }
    }

    for (m, slot) in doc.macros.iter().enumerate() {
        for (a, action) in slot.actions.iter().enumerate() {
            if let MacroAction::Text(text) = action
                && !text.is_ascii()
            {
                warn(
                    format!("macro[{m}][{a}]"),
                    "text contains non-ASCII characters; QMK types only ASCII".to_string(),
                );
            }
        }
    }
    out
}

/// Bytes the macros take in the keyboard's macro buffer (Vial protocol >= 2).
pub fn macro_buffer_bytes(doc: &Document) -> usize {
    doc.macros
        .iter()
        .map(|slot| {
            let actions: usize = slot
                .actions
                .iter()
                .map(|action| match action {
                    MacroAction::Tap(keys) | MacroAction::Down(keys) | MacroAction::Up(keys) => {
                        keys.iter()
                            .map(|kc| if kc.code() < 256 { 3 } else { 4 })
                            .sum::<usize>()
                    }
                    MacroAction::Text(text) => text.len(),
                    MacroAction::Delay(_) => 4,
                })
                .sum();
            actions + 1
        })
        .sum()
}
