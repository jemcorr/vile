use std::collections::HashSet;
use vile_core::{Catalog, CatalogContext, Keycode, KeycodeKind, keycode::PickerCategory};

fn catalog() -> Catalog {
    Catalog::new(CatalogContext::default())
}

#[test]
fn every_selectable_advanced_action_has_behavior_text_and_static_firmware_actions_have_reviewed_copy()
 {
    let cat = catalog();
    let reviewed: HashSet<_> = include_str!("../data/picker.tsv")
        .lines()
        .map(|line| line.split('\t').next().unwrap())
        .collect();
    for hit in cat.search("", 4096) {
        if !hit.complete {
            let text = cat
                .template_tooltip(&hit.insert)
                .expect("search template has behavior text");
            assert!(text.contains("Choose a basic"), "{}: {text}", hit.name);
            continue;
        }
        let code = cat.parse(&hit.name).unwrap();
        if matches!(code.kind(), KeycodeKind::Other) {
            assert!(
                reviewed.contains(hit.name.as_str()),
                "unreviewed firmware action: {}",
                hit.name
            );
        }
        if matches!(
            cat.category(code),
            PickerCategory::Layers
                | PickerCategory::Modifiers
                | PickerCategory::Mouse
                | PickerCategory::Macros
                | PickerCategory::Quantum
                | PickerCategory::Special
        ) {
            let text = cat.tooltip(code);
            assert_ne!(
                text,
                cat.description(code),
                "{} only expands its identifier",
                hit.name
            );
            assert!(
                text.ends_with('.') && text.len() > 70,
                "{} lacks an explanation: {text}",
                hit.name
            );
        }
    }
}

#[test]
fn mouse_labels_fit_keycaps_and_speed_presets_keep_their_firmware_qualifications() {
    let cat = catalog();
    for (wire, expected) in [
        ("KC_BTN1", "Left\nclick"),
        ("KC_BTN2", "Right\nclick"),
        ("KC_BTN3", "Middle\nclick"),
        ("KC_BTN4", "Mouse\n4"),
        ("KC_BTN5", "Mouse\n5"),
        ("KC_ACL0", "Speed\n0"),
        ("KC_ACL1", "Speed\n1"),
        ("KC_ACL2", "Speed\n2"),
    ] {
        let code = cat.parse(wire).unwrap();
        assert_eq!(cat.label(code), expected);
        assert!(
            cat.label(code)
                .lines()
                .all(|line| line.chars().count() <= 6)
        );
        assert_eq!(cat.wire_name(code), wire);
    }
    for wire in ["KC_ACL0", "KC_ACL1", "KC_ACL2"] {
        let tooltip = cat.tooltip(cat.parse(wire).unwrap());
        assert!(
            tooltip.contains("hold")
                && tooltip.contains("latch")
                && tooltip.contains("Firmware settings")
        );
        assert!(!tooltip.contains("slow") && !tooltip.contains("fast"));
    }
    assert!(cat.tooltip(cat.parse("KC_BTN1").unwrap()).contains("drag"));
}

#[test]
fn generated_actions_explain_activation_release_and_persistence() {
    let cat = catalog();
    for (wire, required) in [
        ("MO(2)", vec!["Hold", "release", "Higher active layers"]),
        (
            "TO(2)",
            vec![
                "On press",
                "deactivate every other non-default",
                "not a saved default",
            ],
        ),
        ("TG(2)", vec!["after release", "restart", "does not save"]),
        ("TT(2)", vec!["Repeated taps", "firmware", "not saved"]),
        ("DF(2)", vec!["default base", "only until"]),
        (
            "PDF(2)",
            vec!["persistent memory", "survives", "power loss"],
        ),
        ("OSL(2)", vec!["next keypress", "Hold", "timeout"]),
        (
            "OSM(MOD_LSFT)",
            vec!["Left Shift", "next keypress", "locking"],
        ),
        ("LGUI_T(KC_A)", vec!["Tap", "Hold", "tapping term"]),
        ("LT2(KC_A)", vec!["Tap", "release", "tap-hold settings"]),
        ("LM(2,MOD_LSFT)", vec!["Hold", "Left Shift", "Release"]),
        ("LCTL(KC_A)", vec!["together with Left Ctrl", "Release"]),
        ("M0", vec!["On press", "stored macro slot", "separate"]),
        ("TD(0)", vec!["single tap", "double tap", "tapping term"]),
        ("USER00", vec!["firmware", "cannot infer"]),
    ] {
        let text = cat.tooltip(cat.parse(wire).unwrap());
        for fragment in required {
            assert!(text.contains(fragment), "{wire} lacks {fragment}: {text}");
        }
    }
    assert!(
        cat.tooltip(Keycode(0x7dff))
            .contains("Unknown firmware action")
    );
}

#[test]
fn firmware_actions_preserve_their_material_caveats() {
    let cat = catalog();
    for (wire, required) in [
        (
            "MAGIC_SWAP_CTL_GUI",
            vec!["saves it across restarts", "keyboard's own"],
        ),
        (
            "AU_ON",
            vec![
                "does not itself enter music mode",
                "not the computer's volume",
            ],
        ),
        (
            "CLICKY_RESET",
            vec!["Pitch changes last until", "do not", "reset the keymap"],
        ),
        (
            "HPT_RST",
            vec!["not the keyboard layout", "across restarts"],
        ),
        (
            "RGB_HUI",
            vec![
                "Holding Shift reverses",
                "shared RGB",
                "saved across restarts",
            ],
        ),
        (
            "RM_NEXT",
            vec!["key release", "configure it", "Holding Shift reverses"],
        ),
        (
            "DYN_REC_START1",
            vec!["On release", "replacing", "lost", "separate"],
        ),
        (
            "KC_ASRP",
            vec!["milliseconds", "focused application", "not saved"],
        ),
        (
            "QK_CAPS_WORD_TOGGLE",
            vec!["does not toggle host Caps Lock", "idle timeout"],
        ),
        (
            "QK_ALT_REPEAT_KEY",
            vec!["If no alternate exists", "sends nothing"],
        ),
    ] {
        let text = cat.tooltip(cat.parse(wire).unwrap());
        for fragment in required {
            assert!(text.contains(fragment), "{wire} lacks {fragment}: {text}");
        }
    }
}
