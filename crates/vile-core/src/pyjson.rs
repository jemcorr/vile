//! Python-compatible JSON output. Vial writes .vil files with Python's
//! `json.dumps(obj)` default arguments: `ensure_ascii=True`, item separator
//! ", " and key separator ": ", all on one line. These helpers reproduce that
//! byte for byte.

use std::fmt::Write as _;

/// Append `s` as a JSON string literal, escaped exactly like `json.dumps`.
pub fn write_py_string(out: &mut String, s: &str) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 || (c as u32) >= 0x7F => {
                let mut units = [0u16; 2];
                for unit in c.encode_utf16(&mut units) {
                    // Writing to a String cannot fail.
                    let _ = write!(out, "\\u{unit:04x}");
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Append `"key": ` (the key, then Python's key separator).
pub fn write_key(out: &mut String, key: &str) {
    write_py_string(out, key);
    out.push_str(": ");
}

/// Append `[a, b, c]`, calling `item` to write each element.
pub fn write_list<T>(out: &mut String, items: &[T], mut item: impl FnMut(&mut String, &T)) {
    out.push('[');
    for (i, x) in items.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        item(out, x);
    }
    out.push(']');
}
