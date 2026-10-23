//! Keycodes in Vial protocol v6 numbering, their kinds, and layer references.

mod catalog;
mod table;

pub use catalog::{Catalog, CatalogContext};
pub use table::{Group, TableEntry, table};

// Code ranges of Vial protocol v6 (QMK 0.19 and later).
pub const QK_MODS: u16 = 0x0100;
pub const QK_MOD_TAP: u16 = 0x2000;
pub const QK_LAYER_TAP: u16 = 0x4000;
pub const QK_LAYER_MOD: u16 = 0x5000;
pub const QK_TO: u16 = 0x5200;
pub const QK_MOMENTARY: u16 = 0x5220;
pub const QK_DEF_LAYER: u16 = 0x5240;
pub const QK_TOGGLE_LAYER: u16 = 0x5260;
pub const QK_ONE_SHOT_LAYER: u16 = 0x5280;
pub const QK_ONE_SHOT_MOD: u16 = 0x52A0;
pub const QK_LAYER_TAP_TOGGLE: u16 = 0x52C0;
pub const QK_PERSISTENT_DEF_LAYER: u16 = 0x52E0;
pub const QK_TAP_DANCE: u16 = 0x5700;
pub const QK_MACRO: u16 = 0x7700;
pub const QK_KB: u16 = 0x7E00;
pub const FN_MO13: u16 = 0x7C77;
pub const FN_MO23: u16 = 0x7C78;

/// A QMK keycode in Vial protocol v6 numbering. The number is the identity:
/// two spellings of one code (KC_ESC and KC_ESCAPE) are the same Keycode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Keycode(pub u16);

/// Layer functions that take one layer number: TO(n), MO(n), DF(n), TG(n),
/// OSL(n), TT(n), PDF(n).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LayerOp {
    To,
    Mo,
    Df,
    Tg,
    Osl,
    Tt,
    Pdf,
}

/// Structured view of a keycode, derived only from its code range.
/// `mods` values use QMK's 5-bit encoding: 0x01 ctrl, 0x02 shift, 0x04 alt,
/// 0x08 gui, plus 0x10 when the modifiers are the right-hand ones.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeycodeKind {
    /// 0x0000..=0x00FF
    Basic { key: u8 },
    /// 0x0100..=0x1FFF, for example LSFT(KC_8)
    Mods { mods: u8, key: u8 },
    /// 0x2000..=0x3FFF, for example LSFT_T(KC_N)
    ModTap { mods: u8, key: u8 },
    /// 0x4000..=0x4FFF, for example LT8(KC_ESCAPE)
    LayerTap { layer: u8, key: u8 },
    /// 0x5000..=0x51FF, LM(layer, mods)
    LayerMod { layer: u8, mods: u8 },
    /// 0x5200..=0x529F and 0x52C0..=0x52FF
    Layer { op: LayerOp, layer: u8 },
    /// 0x52A0..=0x52BF, OSM(mods)
    OneShotMod { mods: u8 },
    /// 0x5700..=0x57FF, TD(n)
    TapDance { index: u8 },
    /// 0x7700..=0x77FF, M0..M255
    Macro { index: u8 },
    /// 0x7E00..=0x7E3F, USER00..USER63
    User { index: u8 },
    /// Every other code (quantum, media, lighting, ...)
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum KeycodeError {
    #[error("empty keycode")]
    Empty,
    #[error("unknown keycode \"{0}\"")]
    Unknown(String),
    #[error("unknown function \"{name}\" in \"{input}\"")]
    UnknownFunction { name: String, input: String },
    #[error("bad number \"{number}\" in \"{input}\"")]
    BadNumber { number: String, input: String },
    #[error("{value} is out of range 0..={max} in \"{input}\"")]
    OutOfRange { input: String, value: u32, max: u32 },
    #[error("\"{0}\" needs a basic keycode (code 0x00..0xFF) inside")]
    InnerNotBasic(String),
    #[error("cannot mix left and right modifiers in \"{0}\"")]
    MixedHands(String),
    #[error("unknown modifier \"{modifier}\" in \"{input}\"")]
    UnknownModifier { modifier: String, input: String },
    #[error("malformed keycode \"{0}\"")]
    Malformed(String),
    #[error("keycode 0x{code:04x} cannot reference layer {layer}")]
    LayerDoesNotFit { code: u16, layer: u8 },
}

impl Keycode {
    pub const NO: Keycode = Keycode(0x0000);
    pub const TRNS: Keycode = Keycode(0x0001);

    pub fn code(self) -> u16 {
        self.0
    }

    pub fn kind(self) -> KeycodeKind {
        let c = self.0;
        match c {
            0x0000..=0x00FF => KeycodeKind::Basic { key: c as u8 },
            0x0100..=0x1FFF => KeycodeKind::Mods {
                mods: ((c >> 8) & 0x1F) as u8,
                key: (c & 0xFF) as u8,
            },
            0x2000..=0x3FFF => KeycodeKind::ModTap {
                mods: ((c >> 8) & 0x1F) as u8,
                key: (c & 0xFF) as u8,
            },
            0x4000..=0x4FFF => KeycodeKind::LayerTap {
                layer: ((c >> 8) & 0x0F) as u8,
                key: (c & 0xFF) as u8,
            },
            0x5000..=0x51FF => KeycodeKind::LayerMod {
                layer: ((c >> 5) & 0x0F) as u8,
                mods: (c & 0x1F) as u8,
            },
            0x5200..=0x521F => KeycodeKind::Layer {
                op: LayerOp::To,
                layer: (c & 0x1F) as u8,
            },
            0x5220..=0x523F => KeycodeKind::Layer {
                op: LayerOp::Mo,
                layer: (c & 0x1F) as u8,
            },
            0x5240..=0x525F => KeycodeKind::Layer {
                op: LayerOp::Df,
                layer: (c & 0x1F) as u8,
            },
            0x5260..=0x527F => KeycodeKind::Layer {
                op: LayerOp::Tg,
                layer: (c & 0x1F) as u8,
            },
            0x5280..=0x529F => KeycodeKind::Layer {
                op: LayerOp::Osl,
                layer: (c & 0x1F) as u8,
            },
            0x52A0..=0x52BF => KeycodeKind::OneShotMod {
                mods: (c & 0x1F) as u8,
            },
            0x52C0..=0x52DF => KeycodeKind::Layer {
                op: LayerOp::Tt,
                layer: (c & 0x1F) as u8,
            },
            0x52E0..=0x52FF => KeycodeKind::Layer {
                op: LayerOp::Pdf,
                layer: (c & 0x1F) as u8,
            },
            0x5700..=0x57FF => KeycodeKind::TapDance {
                index: (c & 0xFF) as u8,
            },
            0x7700..=0x77FF => KeycodeKind::Macro {
                index: (c & 0xFF) as u8,
            },
            0x7E00..=0x7E3F => KeycodeKind::User {
                index: (c & 0x3F) as u8,
            },
            _ => KeycodeKind::Other,
        }
    }

    pub fn layer_ref(self) -> Option<u8> {
        match self.kind() {
            KeycodeKind::LayerTap { layer, .. }
            | KeycodeKind::LayerMod { layer, .. }
            | KeycodeKind::Layer { layer, .. } => Some(layer),
            _ => None,
        }
    }

    pub fn with_layer(self, layer: u8) -> Result<Keycode, KeycodeError> {
        let does_not_fit = KeycodeError::LayerDoesNotFit {
            code: self.0,
            layer,
        };
        match self.kind() {
            KeycodeKind::LayerTap { key, .. } if layer <= 15 => Ok(Keycode(
                QK_LAYER_TAP | (u16::from(layer) << 8) | u16::from(key),
            )),
            KeycodeKind::LayerMod { mods, .. } if layer <= 15 => Ok(Keycode(
                QK_LAYER_MOD | (u16::from(layer) << 5) | u16::from(mods),
            )),
            KeycodeKind::Layer { op, .. } if layer <= 31 => {
                Ok(Keycode(op.base() + u16::from(layer)))
            }
            _ => Err(does_not_fit),
        }
    }

    pub fn fixed_layer_refs(self) -> &'static [u8] {
        match self.0 {
            FN_MO13 => &[1, 3],
            FN_MO23 => &[2, 3],
            _ => &[],
        }
    }
}

impl LayerOp {
    pub const ALL: [LayerOp; 7] = [
        LayerOp::To,
        LayerOp::Mo,
        LayerOp::Df,
        LayerOp::Tg,
        LayerOp::Osl,
        LayerOp::Tt,
        LayerOp::Pdf,
    ];

    /// "TO", "MO", "DF", "TG", "OSL", "TT" or "PDF".
    pub fn name(self) -> &'static str {
        match self {
            LayerOp::To => "TO",
            LayerOp::Mo => "MO",
            LayerOp::Df => "DF",
            LayerOp::Tg => "TG",
            LayerOp::Osl => "OSL",
            LayerOp::Tt => "TT",
            LayerOp::Pdf => "PDF",
        }
    }

    /// The first code of this function's range.
    pub fn base(self) -> u16 {
        match self {
            LayerOp::To => QK_TO,
            LayerOp::Mo => QK_MOMENTARY,
            LayerOp::Df => QK_DEF_LAYER,
            LayerOp::Tg => QK_TOGGLE_LAYER,
            LayerOp::Osl => QK_ONE_SHOT_LAYER,
            LayerOp::Tt => QK_LAYER_TAP_TOGGLE,
            LayerOp::Pdf => QK_PERSISTENT_DEF_LAYER,
        }
    }

    pub fn from_name(s: &str) -> Option<LayerOp> {
        LayerOp::ALL.into_iter().find(|op| op.name() == s)
    }
}
