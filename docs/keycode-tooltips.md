# Keycode hover text

`Catalog::description` supplies compact inspector headings and diff names.
`Catalog::tooltip` supplies behavioral explanations to board cells, keycode
fields, inspector headings and builder previews. Picker search results use the
same explanation in their `description` field, with the short caption kept in
`summary`. Incomplete inline-search templates use `Catalog::template_tooltip`.
Layer and macro names are added by the session without discarding the explanation
or its numeric slot/layer identity. Raw wire names remain in each key hover.

Mouse labels are shared by the board and PNG renderer. They use two short lines,
including `Left / click`, `Wheel / ↑`, and `Speed / 0`. Speed preset numbers do not
promise a universal slow/medium/fast ordering.

The content audit covers every selectable catalog entry, generated layer keys,
one-shot modifiers, tap dances, stored macro slots, user keycodes and incomplete
modifier/layer-tap templates. Static firmware actions must have reviewed copy in
`picker.tsv`; generated families have explicit behavior in `tooltip.rs`. Unknown
numeric firmware actions keep an explanation of what cannot be inferred.

## Semantic sources

The explanations describe standard QMK/Vial handling. Custom firmware and host
key mappings can change behavior, and a `.vil` file does not prove that a
particular optional firmware feature or hardware driver is enabled.

| Family | Verified behavior and primary source |
| --- | --- |
| Mouse keys | Button press/release, dragging, movement, wheel direction and firmware-dependent speed selection. Constant-speed configurations can select a preset while held or retain it after release. [QMK Mouse Keys](https://docs.qmk.fm/features/mouse_keys), [Vial mousekey.c](https://github.com/vial-kb/vial-qmk/blob/vial/quantum/mousekey.c). |
| Layers and modifiers | Momentary release, toggle state, default versus persistent default, tap-hold timing, one-shot timeout/locking, layer priority and composite modifier output. [QMK Layers](https://docs.qmk.fm/feature_layers), [One Shot Keys](https://docs.qmk.fm/one_shot_keys), [Mod-Tap](https://docs.qmk.fm/mod_tap). |
| Magic | The configuration changes on press and is written to EEPROM. Gui has host-specific names. [Vial process_magic.c](https://github.com/vial-kb/vial-qmk/blob/vial/quantum/process_keycode/process_magic.c). |
| Audio and music | Audio enable and Clicky enable are saved. Click pitch, Music Mode and music scale are runtime state. Music can consume typing keys. [Vial audio.c](https://github.com/vial-kb/vial-qmk/blob/vial/quantum/audio/audio.c), [process_clicky.c](https://github.com/vial-kb/vial-qmk/blob/vial/quantum/process_keycode/process_clicky.c), [process_music.c](https://github.com/vial-kb/vial-qmk/blob/vial/quantum/process_keycode/process_music.c). |
| Haptics | Driver-dependent vibration controls save their configuration. Haptic reset does not reset the keymap. [Vial haptic.c](https://github.com/vial-kb/vial-qmk/blob/vial/quantum/haptic.c), [QMK Haptic Feedback](https://docs.qmk.fm/features/haptic_feedback). |
| Lighting | Backlight, RGB lighting and matrix controls save settings. Shared RGB keys can affect a matrix; Shift reverses directional RGB adjustments. Matrix trigger timing can be configured. [Vial process_backlight.c](https://github.com/vial-kb/vial-qmk/blob/vial/quantum/process_keycode/process_backlight.c), [process_underglow.c](https://github.com/vial-kb/vial-qmk/blob/vial/quantum/process_keycode/process_underglow.c), [process_rgb_matrix.c](https://github.com/vial-kb/vial-qmk/blob/vial/quantum/process_keycode/process_rgb_matrix.c). |
| Auto Shift and combos | Runtime toggles do not erase definitions. Auto Shift setup keys adjust by 5 ms or type the timeout into the focused application. [Vial process_auto_shift.c](https://github.com/vial-kb/vial-qmk/blob/vial/quantum/process_keycode/process_auto_shift.c), [process_combo.c](https://github.com/vial-kb/vial-qmk/blob/vial/quantum/process_keycode/process_combo.c). |
| Caps Word, repeat and layer lock | Word continuation, alternate lookup, remembered modifiers, lock release and optional timeouts depend on firmware settings. [QMK Caps Word](https://docs.qmk.fm/features/caps_word), [Repeat Key](https://docs.qmk.fm/features/repeat_key), [Layer Lock](https://docs.qmk.fm/features/layer_lock). |
| Dynamic versus stored macros | Dynamic record/play starts on release, Stop Recording on press; recordings are temporary. Stored Vial macros play on press from saved slots. [Vial process_dynamic_macro.c](https://github.com/vial-kb/vial-qmk/blob/vial/quantum/process_keycode/process_dynamic_macro.c), [via.c](https://github.com/vial-kb/vial-qmk/blob/vial/quantum/via.c), [dynamic_keymap.c](https://github.com/vial-kb/vial-qmk/blob/vial/quantum/dynamic_keymap.c). |
| Tap dances and user codes | Vial tap-dance fallback rules come from its implementation. A user keycode's behavior cannot be inferred from its number. [Vial vial.c](https://github.com/vial-kb/vial-qmk/blob/vial/quantum/vial.c). |
| Firmware and host actions | Bootloader, ordinary reboot and persistent-settings reset are distinct. HID media/application requests depend on host support. [QMK Keycodes](https://docs.qmk.fm/keycodes). |

Core tests require reviewed static action coverage and check material behavior
boundaries. Session tests check that board cells, field views and picker results
carry the same full explanation while their visible descriptions stay compact.
Frontend tests check SVG title elements and HTML title attributes and keep hover prose out
of drawn key labels and exported artwork.
