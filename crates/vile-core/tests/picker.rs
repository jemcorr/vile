use std::collections::HashSet;

use vile_core::{Catalog, CatalogContext, keycode::PickerCategory};

fn catalog() -> Catalog {
    Catalog::new(CatalogContext::default())
}

#[test]
fn picker_copy_keeps_wire_identity_and_distinguishes_firmware_resets() {
    let cat = catalog();
    let boot = cat.parse("QK_BOOT").unwrap();
    let reboot = cat.parse("QK_REBOOT").unwrap();
    let reset = cat.parse("QK_CLEAR_EEPROM").unwrap();
    assert_eq!(cat.picker(boot).label, "Bootloader");
    assert!(cat.tooltip(boot).contains("flash firmware"));
    assert!(
        cat.tooltip(reboot)
            .contains("without entering the bootloader")
    );
    assert!(cat.tooltip(reset).contains("persistent settings"));
    assert!(cat.tooltip(reset).contains("depending on the firmware"));
    assert_eq!(cat.description(boot), "Enter bootloader");
    assert_eq!(cat.description(reset), "clear eeprom");
    assert_eq!(cat.wire_name(reset), "QK_CLEAR_EEPROM");
    assert_eq!(cat.label(reset), "CLEAR_EEPROM");
}

#[test]
fn picker_copy_is_bounded_and_covers_real_catalog_entries() {
    let cat = catalog();
    let mut seen = HashSet::new();
    for row in include_str!("../data/picker.tsv").lines() {
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(fields.len(), 4);
        assert!(seen.insert(fields[0]), "duplicate copy for {}", fields[0]);
        let code = cat.parse(fields[0]).unwrap();
        assert_eq!(cat.picker(code).label, fields[1], "{}", fields[0]);
    }
    for hit in cat.search("", 4096).into_iter().filter(|hit| hit.complete) {
        let code = cat.parse(&hit.name).unwrap();
        let picker = cat.picker(code);
        assert!(!picker.label.is_empty(), "{}", hit.name);
        assert!(
            picker.label.chars().count() <= 20,
            "{}: {}",
            hit.name,
            picker.label
        );
        assert!(
            picker.summary.chars().count() <= 32,
            "{}: {}",
            hit.name,
            picker.summary
        );
        assert!(!picker.group.is_empty(), "{}", hit.name);
    }
}

#[test]
fn categories_and_groups_match_how_keys_are_used() {
    let cat = catalog();
    for (wire, category, group) in [
        ("KC_LSHIFT", PickerCategory::Modifiers, "Left modifiers"),
        (
            "KC_EXLM",
            PickerCategory::Symbols,
            "Shifted symbols · US layout",
        ),
        ("KC_RGUI", PickerCategory::Modifiers, "Right modifiers"),
        ("KC_KP_1", PickerCategory::Numbers, "Keypad"),
        ("KC_F13", PickerCategory::Numbers, "Function keys"),
        (
            "MAGIC_SWAP_LCTL_LGUI",
            PickerCategory::Quantum,
            "Key remapping",
        ),
        ("MAGIC_HOST_NKRO", PickerCategory::Quantum, "N-key rollover"),
        ("HPT_FBK", PickerCategory::Quantum, "Haptic feedback"),
        ("RGB_MOD", PickerCategory::Quantum, "RGB lighting"),
        ("RM_NEXT", PickerCategory::Quantum, "RGB matrix"),
        ("KC_WBAK", PickerCategory::Media, "Browser"),
        ("M0", PickerCategory::Macros, "Macro slots"),
        (
            "DYN_REC_START1",
            PickerCategory::Macros,
            "Dynamic recording",
        ),
    ] {
        let code = cat.parse(wire).unwrap();
        assert_eq!(cat.category(code), category, "{wire}");
        assert_eq!(cat.picker(code).group, group, "{wire}");
    }
}

#[test]
fn search_discovers_picker_words_and_preserves_raw_names_and_aliases() {
    let cat = catalog();
    for (query, wire) in [
        ("higher pitch", "CLICKY_UP"),
        ("erase saved settings", "QK_CLEAR_EEPROM"),
        ("MAGIC_SWAP_CONTROL_CAPSLOCK", "MAGIC_SWAP_CONTROL_CAPSLOCK"),
        ("CL_SWAP", "MAGIC_SWAP_CONTROL_CAPSLOCK"),
    ] {
        assert!(
            cat.search(query, 4096).iter().any(|hit| hit.name == wire),
            "{query}"
        );
    }
}
