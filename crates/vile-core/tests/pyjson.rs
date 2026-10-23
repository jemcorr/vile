use vile_core::pyjson::write_py_string;

fn esc(s: &str) -> String {
    let mut out = String::new();
    write_py_string(&mut out, s);
    out
}

#[test]
fn escapes_match_python_json_dumps() {
    // Expected values come from: python3 -c 'import json; print(json.dumps(s))'
    assert_eq!(esc("# %%"), "\"# %%\"");
    assert_eq!(esc("a\"b\\c"), "\"a\\\"b\\\\c\"");
    assert_eq!(esc("\n\t\r\u{08}\u{0C}"), "\"\\n\\t\\r\\b\\f\"");
    assert_eq!(esc("\u{01}\u{1F}\u{7F}"), "\"\\u0001\\u001f\\u007f\"");
    assert_eq!(esc("é€"), "\"\\u00e9\\u20ac\"");
    assert_eq!(esc("😀"), "\"\\ud83d\\ude00\"");
    assert_eq!(esc("/ ~"), "\"/ ~\"");
    assert_eq!(esc(""), "\"\"");
}
