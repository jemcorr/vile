# Changelog

No releases have been tagged or published. The manifests currently declare
`0.1.0` for the intended first release. This entry records the development
history from the first commit through `6164ae4`.

## [Unreleased]

### 2026-10-23

- Added a `.vil` document model with byte-identical round trips, strict Vial v6
  keycode parsing, validation, layer operations with reference renumbering,
  semantic diffs, and atomic file writes (`225abe1`–`7036838`).
- Added CLI validation, inspection, diffs, key edits, and layer operations.
  Commands that change files require `--write` or `--output` to write changes
  (`1aea9cb`, `77f2a70`).
- Added staged editing with undo, diff review, structured keycode edits, and
  editors for combos, macros, tap dances, key overrides, and alternate repeat
  (`c11ca8f`, `d4cbf0f`, `df9ddf6`–`280feb9`).
- Added the Tauri desktop app, definition-based geometry, cached Vial keyboard
  definitions, and read-only device requests for identity, geometry, and macro
  capacity (`706e073`, `12057f1`, `cf5e97b`, `c73f58f`). Device configuration
  writes are not supported.

### 2026-10-24

- Added PNG export for nonblank layers, then included feature entries in the
  export (`fdcb25a`, `fec6bae`).
- Reworked the editor around a canvas UI and added clearer keycode picker
  descriptions, theme selection, and keyboard/matrix views (`a88ec6d`,
  `bdc4f3d`, `8057472`, `744d65c`).
- Added copy and drag actions for feature slots, with drag feedback
  (`2d1be1a`, `b6683e9`).
- Added CI configuration for checks and native bundle builds on macOS, Linux,
  and Windows (`6f02dca`). The commit history does not establish that these jobs
  passed.

The app edits layouts offline. Save the `.vil` file, then load it onto a keyboard
with Vial. Device requests remain read-only; no live configuration apply or
firmware update is supported.
