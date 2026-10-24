use vile_session::{KeyRefDto, MacroActionDto, OpDto};

#[test]
fn op_dto_reads_camel_case_json() {
    let op: OpDto =
        serde_json::from_str(r#"{"type":"setKeys","layer":0,"keys":["r3c4"],"keycode":"KC_A"}"#)
            .unwrap();
    assert_eq!(
        op,
        OpDto::SetKeys {
            layer: 0,
            keys: vec!["r3c4".to_string()],
            keycode: "KC_A".to_string()
        }
    );
    let op: OpDto = serde_json::from_str(
        r#"{"type":"setTapDance","index":1,"onTap":"KC_A","onHold":"KC_LGUI","onDoubleTap":"KC_NO","onTapHold":"KC_NO","tappingTerm":180}"#,
    )
    .unwrap();
    assert_eq!(
        op,
        OpDto::SetTapDance {
            index: 1,
            on_tap: "KC_A".to_string(),
            on_hold: "KC_LGUI".to_string(),
            on_double_tap: "KC_NO".to_string(),
            on_tap_hold: "KC_NO".to_string(),
            tapping_term: 180
        }
    );
    let op: OpDto = serde_json::from_str(
        r#"{"type":"swapKeys","a":[{"layer":0,"key":"r0c2"}],"b":[{"layer":9,"key":"r0c1"}]}"#,
    )
    .unwrap();
    assert_eq!(
        op,
        OpDto::SwapKeys {
            a: vec![KeyRefDto {
                layer: 0,
                key: "r0c2".to_string()
            }],
            b: vec![KeyRefDto {
                layer: 9,
                key: "r0c1".to_string()
            }]
        }
    );
    let op: OpDto = serde_json::from_str(
        r#"{"type":"setMacro","index":2,"actions":[{"kind":"text","keys":[],"text":"hi","delay":0}]}"#,
    )
    .unwrap();
    assert_eq!(
        op,
        OpDto::SetMacro {
            index: 2,
            actions: vec![MacroActionDto {
                kind: "text".to_string(),
                keys: vec![],
                text: "hi".to_string(),
                delay: 0
            }]
        }
    );
    let op: OpDto = serde_json::from_str(
        r#"{"type":"setKeyOverride","index":0,"trigger":"KC_A","replacement":"KC_B","layers":65535,"triggerMods":2,"negativeModMask":0,"suppressedMods":2,"options":135}"#,
    )
    .unwrap();
    assert!(matches!(
        op,
        OpDto::SetKeyOverride {
            negative_mod_mask: 0,
            suppressed_mods: 2,
            ..
        }
    ));
    let op: OpDto =
        serde_json::from_str(r#"{"type":"mirrorLayer","layer":1,"mode":"leftToRight"}"#).unwrap();
    assert_eq!(
        op,
        OpDto::MirrorLayer {
            layer: 1,
            mode: "leftToRight".to_string()
        }
    );
    assert!(serde_json::from_str::<OpDto>(r#"{"type":"bogus"}"#).is_err());
}
