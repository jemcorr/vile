//! Lossless keycode decomposition and validated structured edits.
use super::{Keycode, KeycodeKind, LayerOp};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum HoldBehavior {
    #[default]
    None,
    Mod {
        mods: u8,
    },
    Layer {
        layer: u8,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum KeyParts {
    Tap {
        base: u16,
        wrap_mods: u8,
        hold: HoldBehavior,
    },
    Layer {
        op: LayerOp,
        layer: u8,
    },
    OneShot {
        mods: u8,
    },
    LayerMod {
        layer: u8,
        mods: u8,
    },
    Raw {
        code: u16,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KeyPartsPatch {
    pub base: Option<u16>,
    pub wrap_mods: Option<u8>,
    pub hold: Option<HoldBehavior>,
    pub layer: Option<u8>,
    pub mods: Option<u8>,
}

fn valid_mods(mods: u8, allow_empty: bool) -> Result<(), String> {
    if mods > 31 || (!allow_empty && mods & 15 == 0) || (mods == 16) {
        return Err("Choose Ctrl, Shift, Alt or Gui and one shared left/right side".into());
    }
    Ok(())
}

pub fn compose(parts: KeyParts) -> Result<Keycode, String> {
    let code = match parts {
        KeyParts::Raw { code } => code,
        KeyParts::Tap {
            base,
            wrap_mods,
            hold,
        } => {
            valid_mods(wrap_mods, true)?;
            if wrap_mods != 0 && hold != HoldBehavior::None {
                return Err("Wrap modifiers and hold behavior cannot share one keycode".into());
            }
            if (wrap_mods != 0 || hold != HoldBehavior::None) && base > 255 {
                return Err(
                    "Wrapped, mod-tap and layer-tap keys need a basic tap key (0x00–0xFF)".into(),
                );
            }
            // Composite numeric codes must be decomposed rather than hidden in the base.
            if base > 255
                && !matches!(
                    Keycode(base).kind(),
                    KeycodeKind::Other
                        | KeycodeKind::Macro { .. }
                        | KeycodeKind::TapDance { .. }
                        | KeycodeKind::User { .. }
                )
            {
                return Err("Choose the composite key's individual parts".into());
            }
            match hold {
                HoldBehavior::None => base | (u16::from(wrap_mods) << 8),
                HoldBehavior::Mod { mods } => {
                    valid_mods(mods, false)?;
                    0x2000 | (u16::from(mods) << 8) | base
                }
                HoldBehavior::Layer { layer } => {
                    if layer > 15 {
                        return Err("Layer-tap can only reach layers 0–15".into());
                    }
                    0x4000 | (u16::from(layer) << 8) | base
                }
            }
        }
        KeyParts::Layer { op, layer } => {
            if layer > 31 {
                return Err("Layer keys can only reach layers 0–31".into());
            }
            op.base() | u16::from(layer)
        }
        KeyParts::OneShot { mods } => {
            valid_mods(mods, false)?;
            0x52a0 | u16::from(mods)
        }
        KeyParts::LayerMod { layer, mods } => {
            valid_mods(mods, false)?;
            if layer > 15 {
                return Err("Layer-mod can only reach layers 0–15".into());
            }
            0x5000 | (u16::from(layer) << 5) | u16::from(mods)
        }
    };
    Ok(Keycode(code))
}

pub fn decompose(code: Keycode) -> KeyParts {
    let parts = match code.kind() {
        KeycodeKind::Mods { mods, key } => KeyParts::Tap {
            base: key.into(),
            wrap_mods: mods,
            hold: HoldBehavior::None,
        },
        KeycodeKind::ModTap { mods, key } => KeyParts::Tap {
            base: key.into(),
            wrap_mods: 0,
            hold: HoldBehavior::Mod { mods },
        },
        KeycodeKind::LayerTap { layer, key } => KeyParts::Tap {
            base: key.into(),
            wrap_mods: 0,
            hold: HoldBehavior::Layer { layer },
        },
        KeycodeKind::Layer { op, layer } => KeyParts::Layer { op, layer },
        KeycodeKind::LayerMod { layer, mods } => KeyParts::LayerMod { layer, mods },
        KeycodeKind::OneShotMod { mods } => KeyParts::OneShot { mods },
        _ => KeyParts::Tap {
            base: code.0,
            wrap_mods: 0,
            hold: HoldBehavior::None,
        },
    };
    if compose(parts) == Ok(code) {
        parts
    } else {
        KeyParts::Raw { code: code.0 }
    }
}

impl KeyPartsPatch {
    pub fn apply(self, code: Keycode) -> Result<Keycode, String> {
        let mut parts = decompose(code);
        match &mut parts {
            KeyParts::Tap {
                base,
                wrap_mods,
                hold,
            } => {
                if self.layer.is_some() || self.mods.is_some() {
                    return Err("This key has no standalone layer or modifier field".into());
                }
                if let Some(value) = self.base {
                    *base = value;
                }
                if let Some(value) = self.wrap_mods {
                    *wrap_mods = value;
                }
                if let Some(value) = self.hold {
                    *hold = value;
                }
            }
            KeyParts::Layer { layer, .. } | KeyParts::LayerMod { layer, .. } => {
                if self.base.is_some() || self.wrap_mods.is_some() || self.hold.is_some() {
                    return Err("This standalone key has no base or hold field".into());
                }
                if let Some(value) = self.layer {
                    *layer = value;
                }
                if let Some(value) = self.mods {
                    if let KeyParts::LayerMod { mods, .. } = &mut parts {
                        *mods = value;
                    } else {
                        return Err("This layer key has no modifier field".into());
                    }
                }
            }
            KeyParts::OneShot { mods } => {
                if self.base.is_some()
                    || self.wrap_mods.is_some()
                    || self.hold.is_some()
                    || self.layer.is_some()
                {
                    return Err("One-shot modifiers have no base, hold or layer field".into());
                }
                if let Some(value) = self.mods {
                    *mods = value;
                }
            }
            KeyParts::Raw { .. } => {
                return Err(
                    "This numeric encoding is preserved as-is; replace the whole key to edit it"
                        .into(),
                );
            }
        }
        compose(parts)
    }
}
