use vile_core::{Keycode, KeycodeKind, LayerOp};

#[test]
fn kinds_follow_the_v6_code_ranges() {
    assert_eq!(Keycode(0x0004).kind(), KeycodeKind::Basic { key: 0x04 });
    assert_eq!(
        Keycode(0x0225).kind(),
        KeycodeKind::Mods {
            mods: 0x02,
            key: 0x25
        }
    );
    assert_eq!(
        Keycode(0x2211).kind(),
        KeycodeKind::ModTap {
            mods: 0x02,
            key: 0x11
        }
    );
    assert_eq!(
        Keycode(0x380B).kind(),
        KeycodeKind::ModTap {
            mods: 0x18,
            key: 0x0B
        }
    );
    assert_eq!(
        Keycode(0x4829).kind(),
        KeycodeKind::LayerTap {
            layer: 8,
            key: 0x29
        }
    );
    assert_eq!(
        Keycode(0x5022).kind(),
        KeycodeKind::LayerMod {
            layer: 1,
            mods: 0x02
        }
    );
    assert_eq!(
        Keycode(0x5201).kind(),
        KeycodeKind::Layer {
            op: LayerOp::To,
            layer: 1
        }
    );
    assert_eq!(
        Keycode(0x5229).kind(),
        KeycodeKind::Layer {
            op: LayerOp::Mo,
            layer: 9
        }
    );
    assert_eq!(
        Keycode(0x5242).kind(),
        KeycodeKind::Layer {
            op: LayerOp::Df,
            layer: 2
        }
    );
    assert_eq!(
        Keycode(0x5263).kind(),
        KeycodeKind::Layer {
            op: LayerOp::Tg,
            layer: 3
        }
    );
    assert_eq!(
        Keycode(0x5284).kind(),
        KeycodeKind::Layer {
            op: LayerOp::Osl,
            layer: 4
        }
    );
    assert_eq!(
        Keycode(0x52A2).kind(),
        KeycodeKind::OneShotMod { mods: 0x02 }
    );
    assert_eq!(
        Keycode(0x52C5).kind(),
        KeycodeKind::Layer {
            op: LayerOp::Tt,
            layer: 5
        }
    );
    assert_eq!(
        Keycode(0x52E6).kind(),
        KeycodeKind::Layer {
            op: LayerOp::Pdf,
            layer: 6
        }
    );
    assert_eq!(Keycode(0x5703).kind(), KeycodeKind::TapDance { index: 3 });
    assert_eq!(Keycode(0x7700).kind(), KeycodeKind::Macro { index: 0 });
    assert_eq!(Keycode(0x7E02).kind(), KeycodeKind::User { index: 2 });
    assert_eq!(Keycode(0x7C79).kind(), KeycodeKind::Other);
}

#[test]
fn layer_references_and_rewrites() {
    assert_eq!(Keycode(0x0004).layer_ref(), None);
    assert_eq!(Keycode(0x4829).layer_ref(), Some(8));
    assert_eq!(Keycode(0x5022).layer_ref(), Some(1));
    assert_eq!(Keycode(0x52C4).layer_ref(), Some(4));
    assert_eq!(Keycode(0x52A2).layer_ref(), None);
    assert_eq!(Keycode(0x4829).with_layer(7), Ok(Keycode(0x4729)));
    assert_eq!(Keycode(0x5229).with_layer(2), Ok(Keycode(0x5222)));
    assert_eq!(Keycode(0x5229).with_layer(31), Ok(Keycode(0x523F)));
    assert_eq!(Keycode(0x5022).with_layer(3), Ok(Keycode(0x5062)));
    assert!(Keycode(0x4829).with_layer(16).is_err());
    assert!(Keycode(0x5022).with_layer(16).is_err());
    assert!(Keycode(0x5229).with_layer(32).is_err());
    assert!(Keycode(0x0004).with_layer(1).is_err());
    assert_eq!(Keycode(0x7C77).fixed_layer_refs(), &[1, 3]);
    assert_eq!(Keycode(0x7C78).fixed_layer_refs(), &[2, 3]);
    assert!(Keycode(0x0004).fixed_layer_refs().is_empty());
}

#[test]
fn constants() {
    assert_eq!(Keycode::NO, Keycode(0x0000));
    assert_eq!(Keycode::TRNS, Keycode(0x0001));
    assert_eq!(Keycode(0x1234).code(), 0x1234);
    assert_eq!(LayerOp::Mo.name(), "MO");
    assert_eq!(LayerOp::Pdf.base(), 0x52E0);
}
