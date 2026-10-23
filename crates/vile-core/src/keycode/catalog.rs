//! Parsing keycode strings and naming keycodes exactly the way Vial does.

use super::{Keycode, KeycodeError};

/// What the names in one document depend on: Vial names MO(n), LT{n}(kc)
/// and M{n} only for layers and macros that exist on the keyboard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CatalogContext {
    pub layers: u8,
    pub macros: u16,
    pub tap_dances: u16,
}

impl Default for CatalogContext {
    /// The Corne fixture: 10 layers, 16 macros, 32 tap dances.
    fn default() -> Self {
        CatalogContext {
            layers: 10,
            macros: 16,
            tap_dances: 32,
        }
    }
}

/// Keycode parsing and naming for one document context. Cheap to copy:
/// the name tables behind it are built once per process.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Catalog {
    ctx: CatalogContext,
}

use std::collections::HashMap;
use std::sync::OnceLock;

use super::{
    FN_MO13, FN_MO23, Group, KeycodeKind, LayerOp, QK_KB, QK_LAYER_MOD, QK_LAYER_TAP, QK_MACRO,
    QK_MOD_TAP, QK_ONE_SHOT_MOD, QK_TAP_DANCE, table,
};

/// Name tables shared by every catalog, built once per process.
struct Names {
    /// Name and aliases of every non-function entry, mapped to the code.
    plain: HashMap<&'static str, u16>,
    /// Code to canonical name for the non-function entries.
    plain_by_code: HashMap<u16, &'static str>,
    /// Name and aliases of every function entry, mapped to (group, code).
    funcs: HashMap<&'static str, (Group, u16)>,
    /// Code to canonical name for the Mods and ModTap entries (NAME(inner) outer codes).
    masked: HashMap<u16, &'static str>,
}

static NAMES: OnceLock<Names> = OnceLock::new();

fn names() -> &'static Names {
    NAMES.get_or_init(|| {
        let mut names = Names {
            plain: HashMap::new(),
            plain_by_code: HashMap::new(),
            funcs: HashMap::new(),
            masked: HashMap::new(),
        };
        for entry in table() {
            let all_names = std::iter::once(entry.name).chain(entry.aliases.iter().copied());
            if entry.group.is_function() {
                for name in all_names {
                    names.funcs.insert(name, (entry.group, entry.code));
                }
                if matches!(entry.group, Group::Mods | Group::ModTap) {
                    names.masked.insert(entry.code, entry.name);
                }
            } else {
                for name in all_names {
                    names.plain.insert(name, entry.code);
                }
                names.plain_by_code.insert(entry.code, entry.name);
            }
        }
        names
    })
}

fn hex(code: u16) -> String {
    format!("0x{code:x}")
}

fn is_ascii_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// A decimal argument: digits only, no leading zero except "0", at most `max`.
fn decimal(arg: &str, max: u32, input: &str) -> Result<u32, KeycodeError> {
    let well_formed = is_ascii_digits(arg) && (arg == "0" || !arg.starts_with('0'));
    if !well_formed {
        return Err(KeycodeError::BadNumber {
            number: arg.to_string(),
            input: input.to_string(),
        });
    }
    let value = arg.parse::<u32>().unwrap_or(u32::MAX);
    if value > max {
        return Err(KeycodeError::OutOfRange {
            input: input.to_string(),
            value,
            max,
        });
    }
    Ok(value)
}

/// Splits function arguments at commas that are not inside parentheses.
fn top_level_args(args: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    for (i, ch) in args.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                parts.push(&args[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&args[start..]);
    parts
}

impl Catalog {
    /// A catalog for one document context.
    pub fn new(ctx: CatalogContext) -> Catalog {
        Catalog { ctx }
    }

    pub fn context(&self) -> CatalogContext {
        self.ctx
    }

    /// Strict parse of a keycode string as found in a .vil file.
    pub fn parse(&self, s: &str) -> Result<Keycode, KeycodeError> {
        self.parse_code(s, false).map(Keycode)
    }

    /// The exact string Vial writes for this keycode in this context.
    pub fn wire_name(&self, kc: Keycode) -> String {
        let c = kc.0;
        let hi = c & 0xFF00;
        let names = names();
        let is_layer_tap_hi = (QK_LAYER_TAP..=QK_LAYER_TAP + 0x0F00).contains(&hi);
        if names.masked.contains_key(&hi) || is_layer_tap_hi {
            let outer = match names.masked.get(&hi) {
                Some(name) => Some((*name).to_string()),
                None => {
                    let n = (hi - QK_LAYER_TAP) >> 8;
                    (n < u16::from(self.ctx.layers.min(16))).then(|| format!("LT{n}"))
                }
            };
            let inner = self.plain_name(c & 0x00FF);
            return match (outer, inner) {
                (Some(outer), Some(inner)) => format!("{outer}({inner})"),
                _ => hex(c),
            };
        }
        self.plain_name(c).unwrap_or_else(|| hex(c))
    }

    /// Parses by the rules of step 1.5. `lenient` adds the "KC_" lookup of
    /// step 2.4 for plain names; the strict parser passes false.
    fn parse_code(&self, s: &str, lenient: bool) -> Result<u16, KeycodeError> {
        let names = names();
        if s.is_empty() {
            return Err(KeycodeError::Empty);
        }
        if let Some(hex_digits) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
            let valid = (1..=4).contains(&hex_digits.len())
                && hex_digits.bytes().all(|b| b.is_ascii_hexdigit());
            return match u16::from_str_radix(hex_digits, 16) {
                Ok(code) if valid => Ok(code),
                _ => Err(KeycodeError::Malformed(s.to_string())),
            };
        }
        if let Some(&code) = names.plain.get(s) {
            return Ok(code);
        }
        if lenient {
            let prefixed = format!("KC_{s}");
            if let Some(&code) = names.plain.get(prefixed.as_str()) {
                return Ok(code);
            }
        }
        if let Some(digits) = s.strip_prefix('M')
            && is_ascii_digits(digits)
        {
            let n = decimal(digits, 255, s)?;
            return Ok(QK_MACRO + n as u16);
        }
        if let Some(digits) = s.strip_prefix("USER")
            && digits.len() == 2
            && is_ascii_digits(digits)
        {
            let n = digits
                .bytes()
                .fold(0u32, |acc, b| acc * 10 + u32::from(b - b'0'));
            if n > 63 {
                return Err(KeycodeError::OutOfRange {
                    input: s.to_string(),
                    value: n,
                    max: 63,
                });
            }
            return Ok(QK_KB + n as u16);
        }
        if let Some(open) = s.find('(')
            && s.ends_with(')')
        {
            return self.parse_function(s, &s[..open], &s[open + 1..s.len() - 1], lenient);
        }
        Err(KeycodeError::Unknown(s.to_string()))
    }

    /// Rule 6: NAME(args), where `input` is the whole string.
    fn parse_function(
        &self,
        input: &str,
        name: &str,
        args: &str,
        lenient: bool,
    ) -> Result<u16, KeycodeError> {
        let malformed = || KeycodeError::Malformed(input.to_string());
        if input.chars().any(char::is_whitespace) {
            return Err(malformed());
        }
        let mut depth = 0i32;
        for ch in args.chars() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth < 0 {
                        return Err(malformed());
                    }
                }
                _ => {}
            }
        }
        if depth != 0 {
            return Err(malformed());
        }
        let parts = top_level_args(args);

        if let Some(op) = LayerOp::from_name(name) {
            let [arg] = parts.as_slice() else {
                return Err(malformed());
            };
            let n = decimal(arg, 31, input)?;
            return Ok(op.base() + n as u16);
        }
        if name == "TD" {
            let [arg] = parts.as_slice() else {
                return Err(malformed());
            };
            let n = decimal(arg, 255, input)?;
            return Ok(QK_TAP_DANCE + n as u16);
        }
        if name == "LT" {
            let [layer, inner] = parts.as_slice() else {
                return Err(malformed());
            };
            let layer = decimal(layer, 15, input)? as u16;
            let inner = self.parse_basic(inner, input, lenient)?;
            return Ok(QK_LAYER_TAP | (layer << 8) | inner);
        }
        if let Some(digits) = name.strip_prefix("LT")
            && is_ascii_digits(digits)
        {
            let layer = decimal(digits, 15, input)? as u16;
            let [arg] = parts.as_slice() else {
                return Err(malformed());
            };
            let inner = self.parse_basic(arg, input, lenient)?;
            return Ok(QK_LAYER_TAP | (layer << 8) | inner);
        }
        if name == "OSM" {
            let [arg] = parts.as_slice() else {
                return Err(malformed());
            };
            return Ok(QK_ONE_SHOT_MOD | modifiers(arg, input)?);
        }
        if name == "LM" {
            let [layer, mods] = parts.as_slice() else {
                return Err(malformed());
            };
            let layer = decimal(layer, 15, input)? as u16;
            let mods = modifiers(mods, input)?;
            return Ok(QK_LAYER_MOD | (layer << 5) | mods);
        }
        if name == "MT" {
            let [mods, inner] = parts.as_slice() else {
                return Err(malformed());
            };
            let mods = modifiers(mods, input)?;
            let inner = self.parse_basic(inner, input, lenient)?;
            return Ok(QK_MOD_TAP | (mods << 8) | inner);
        }

        let Some(&(group, outer)) = names().funcs.get(name) else {
            return Err(KeycodeError::UnknownFunction {
                name: name.to_string(),
                input: input.to_string(),
            });
        };
        let [arg] = parts.as_slice() else {
            return Err(malformed());
        };
        match group {
            Group::Mods | Group::ModsHex => {
                let inner = self.parse_code(arg, lenient)?;
                if inner <= 0xFF {
                    Ok(outer | inner)
                } else if (0x0100..=0x1FFF).contains(&inner) {
                    let om = (outer >> 8) & 0x1F;
                    let im = (inner >> 8) & 0x1F;
                    if om & 0x10 != im & 0x10 {
                        Err(KeycodeError::MixedHands(input.to_string()))
                    } else {
                        Ok(((om | im) << 8) | (inner & 0xFF))
                    }
                } else {
                    Err(KeycodeError::InnerNotBasic(input.to_string()))
                }
            }
            Group::ModTap | Group::ModTapHex => {
                let inner = self.parse_basic(arg, input, lenient)?;
                Ok(outer | inner)
            }
            _ => Err(KeycodeError::UnknownFunction {
                name: name.to_string(),
                input: input.to_string(),
            }),
        }
    }

    /// An argument that must parse to a code of 0x00..=0xFF.
    fn parse_basic(&self, arg: &str, input: &str, lenient: bool) -> Result<u16, KeycodeError> {
        let code = self.parse_code(arg, lenient)?;
        if code <= 0xFF {
            Ok(code)
        } else {
            Err(KeycodeError::InnerNotBasic(input.to_string()))
        }
    }

    /// The canonical name of a code without its outer function, if Vial has one.
    fn plain_name(&self, code: u16) -> Option<String> {
        let names = names();
        let ctx = self.ctx;
        if code == FN_MO13 || code == FN_MO23 {
            return if ctx.layers >= 4 {
                names
                    .plain_by_code
                    .get(&code)
                    .map(|name| (*name).to_string())
            } else {
                None
            };
        }
        if let Some(name) = names.plain_by_code.get(&code) {
            return Some((*name).to_string());
        }
        match code {
            0x5200..=0x52FF if !(0x52A0..=0x52BF).contains(&code) => {
                let n = code & 0x1F;
                match Keycode(code).kind() {
                    KeycodeKind::Layer { op, .. } if n < u16::from(ctx.layers) => {
                        Some(format!("{}({})", op.name(), n))
                    }
                    _ => None,
                }
            }
            0x5700..=0x57FF => Some(format!("TD({})", code & 0xFF)),
            0x7700..=0x77FF => {
                let n = code & 0xFF;
                (n < ctx.macros).then(|| format!("M{n}"))
            }
            0x7E00..=0x7E0F => Some(format!("USER{:02}", code & 0xFF)),
            _ => None,
        }
    }
}

/// A modifier expression such as MOD_LSFT|MOD_LCTL.
fn modifiers(expr: &str, input: &str) -> Result<u16, KeycodeError> {
    let mut value = 0u16;
    let mut right = false;
    let mut left = false;
    for token in expr.split('|') {
        let bits: u16 = match token {
            "MOD_LCTL" => 0x01,
            "MOD_LSFT" => 0x02,
            "MOD_LALT" => 0x04,
            "MOD_LGUI" => 0x08,
            "MOD_RCTL" => 0x11,
            "MOD_RSFT" => 0x12,
            "MOD_RALT" => 0x14,
            "MOD_RGUI" => 0x18,
            "MOD_MEH" => 0x07,
            "MOD_HYPR" => 0x0F,
            other => {
                return Err(KeycodeError::UnknownModifier {
                    modifier: other.to_string(),
                    input: input.to_string(),
                });
            }
        };
        if bits & 0x10 != 0 {
            right = true;
        } else {
            left = true;
        }
        value |= bits;
    }
    if left && right {
        return Err(KeycodeError::MixedHands(input.to_string()));
    }
    Ok(value)
}

/// Symbols a person may type as one character, and their keycode strings (US layout).
pub(super) const SYMBOLS: &[(char, &str)] = &[
    ('-', "KC_MINUS"),
    ('=', "KC_EQUAL"),
    ('[', "KC_LBRACKET"),
    (']', "KC_RBRACKET"),
    ('\\', "KC_BSLASH"),
    (';', "KC_SCOLON"),
    ('\'', "KC_QUOTE"),
    ('`', "KC_GRAVE"),
    (',', "KC_COMMA"),
    ('.', "KC_DOT"),
    ('/', "KC_SLASH"),
    ('_', "LSFT(KC_MINUS)"),
    ('+', "LSFT(KC_EQUAL)"),
    ('{', "LSFT(KC_LBRACKET)"),
    ('}', "LSFT(KC_RBRACKET)"),
    ('|', "LSFT(KC_BSLASH)"),
    (':', "LSFT(KC_SCOLON)"),
    ('"', "LSFT(KC_QUOTE)"),
    ('~', "LSFT(KC_GRAVE)"),
    ('<', "LSFT(KC_COMMA)"),
    ('>', "LSFT(KC_DOT)"),
    ('?', "LSFT(KC_SLASH)"),
    ('!', "LSFT(KC_1)"),
    ('@', "LSFT(KC_2)"),
    ('#', "LSFT(KC_3)"),
    ('$', "LSFT(KC_4)"),
    ('%', "LSFT(KC_5)"),
    ('^', "LSFT(KC_6)"),
    ('&', "LSFT(KC_7)"),
    ('*', "LSFT(KC_8)"),
    ('(', "LSFT(KC_9)"),
    (')', "LSFT(KC_0)"),
];

impl Catalog {
    /// Lenient parse for text a person typed: trims, accepts one symbol, any
    /// letter case, optional KC_ prefixes, and missing closing parentheses.
    pub fn parse_input(&self, s: &str) -> Result<Keycode, KeycodeError> {
        let text = s.trim();
        if text.is_empty() {
            return Err(KeycodeError::Empty);
        }
        let mut chars = text.chars();
        if let (Some(only), None) = (chars.next(), chars.next())
            && let Some((_, name)) = SYMBOLS.iter().find(|(symbol, _)| *symbol == only)
        {
            return self.parse_code(name, false).map(Keycode);
        }
        let mut upper: String = text
            .to_ascii_uppercase()
            .chars()
            .filter(|c| *c != ' ')
            .collect();
        let opens = upper.matches('(').count();
        let closes = upper.matches(')').count();
        for _ in closes..opens {
            upper.push(')');
        }
        self.parse_code(&upper, true).map(Keycode)
    }
}
