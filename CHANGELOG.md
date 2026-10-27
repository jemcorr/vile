# Changelog

## [Unreleased]

0.3.1 and older cannot update themselves. Install 0.4.0 once from this release;
later versions arrive automatically.

### Added

- vile can update itself. Restart to update appears once a new version is
  downloaded. Automatic checks are off by default; turn them on with "Updates:
  turn on automatic checks" in the command menu. "Check for updates" always
  works (`b83bd19`, `5a8e4a7`).

### Fixed

- Box selection picks every key the box touches, not only keys whose centre
  is inside it (`bbce9e7`).

## [0.3.1] - 2026-10-26

### Added

- Rename layers from the layer tab menu or the command palette; a rename is
  one undo step (`84bf507`).

### Fixed

- Board edits (clear, paste, cut, swaps, drags, picker drops, the keycode
  dialog and layer operations) are blocked while a macro draft is unsaved, so
  the draft is never lost (`1436f23`).
- A failed keyboard definition refresh now shows an error instead of failing
  silently (`afd2860`).

### Removed

- The unreachable keyboard shortcuts dialog and its letter-key remapping
  (`deab830`).

## [0.3.0] - 2026-10-26

### Added

- Feature slots work like keys. Clear (Delete), cut, copy, paste and duplicate
  (⌘X, ⌘C, ⌘V, ⌘D) one or several slots; arrows move between slots and
  Shift- or ⌘-click builds a selection. The slot menu adds Rename for combos
  and macros and Enable or Disable for key overrides and alternate repeat,
  with reasons beside unavailable items (`2a9f04b`, `75b6898`, `622cef3`,
  `b98d605`).
- Dragging one slot onto another swaps them and renumbers every key that uses
  either macro or tap dance, in one undo step; Alt-drag copies. The changes
  sheet words these moves and renumberings (`6dffb27`, `2eefa05`, `1ded2e9`).
- Macros and tap dances list the keys that use them as jump links, and
  clearing a slot that keys still use says so (`fa8ba5f`, `967a181`).
- Empty slots offer a new entry, and toasts can jump to what they mention
  (`cd9e19d`, `f8874e3`).

## [0.2.1] - 2026-10-26

### Fixed

- Collapsing or expanding the key inspector no longer moves or resizes the
  board; the inspector floats over the canvas (`8072553`).
- Opening a feature slot's menu no longer greys the workspace, and Escape
  closes it (`29d4b83`).
- Keycode search no longer matches letter by letter across descriptions, so
  exact names such as `KC_TRNS` or `QK_REBOOT` return only their keycode, and
  keys found through layer names rank after real matches (`5158468`,
  `8765eb2`).
- Searching inside an open picker group filters that group, with a way to
  search all keycodes when nothing matches (`284c977`).

### Changed

- The key inspector is narrower (280 px by default, at most 320 px), drops its
  deselect button, and uses segmented controls for the hold choice and the
  modifier side. Hold modifier and Tap modifier are collapsible sections with
  even spacing (`aa76553`–`75be15d`).

## [0.2.0] - 2026-10-25

### Added

- Zoom and pan on the keymap canvas: pinch or Cmd-scroll to zoom around the
  pointer, Cmd-= and Cmd-- to step, Cmd-0 to fit, two-finger scroll or
  middle-drag to pan, and fit / zoom controls in the corner. The fit or zoom
  level is remembered per viewer. Zooming with the controls or the keyboard
  keeps the whole cursor key in view (`e8269a8`, `7daf483`, `318a593`).
- Shift-arrow selection grows and shrinks a box from an anchor, as in a
  spreadsheet. Keyboard view uses the visible grid; Matrix view uses electrical
  rows and columns (`06b8b24`, `4b30ae2`).
- A changes sheet replaces the Review page and the save dialog. The toolbar
  count, Review & save, and Cmd-S open it. It groups changes by layer and
  feature in plain language with jump links, compares with the last save or the
  last commit, and states what saving writes (`88f0396`–`bdaae8e`).
- The sheet summarises intent: swaps, moves, cleared or transparent keys,
  copied layers and slots, and reordered layers each read as one change, and
  the count matches the rows. Uncertain cases fall back to per-key rows
  (`65bb339`–`292626f`).
- Changed fields, feature slots and layer tabs show the yellow changed dot, and
  every marker follows the chosen comparison (`5dee2e9`, `ebe3d0b`).

### Changed

- The key inspector is a collapsible floating card. Fit leaves room for it on
  wide windows; on narrow windows the collapsed card sits below the layer tabs
  (`40651b6`, `499e0a7`, `c0c652d`).
- Marquee selection also starts in the gaps between keys (`263e9aa`).
- The window has a minimum size of 640 × 540 (`f376fde`).
- Releases are built from a tag through one workflow that also publishes the
  assets and checksums (`11fb87f`, `2fadad4`). The README links the latest
  release (`479985c`).

## [0.1.1] - 2026-10-25

### Fixed

- Key override and alternate repeat option rows show the same padded hover
  whether their switch is on or off (`a8864ee`).
- Shift-click and Cmd-click (Ctrl-click on Linux and Windows) add or remove a
  key from the selection (`b9efd4e`).
- Keys removed from the selection no longer look selected. Selected keys use an
  accent ring over their own colour, and the cursor outline shows only during
  keyboard navigation (`24db02c`).
- Marquee selection starts anywhere in the keymap canvas, not only over the
  keyboard (`b344d41`).

### Changed

- Updated direct npm, Rust, and GitHub Actions dependencies to their latest
  compatible stable releases. Enabled TypeScript 7.0.2 through `svelte-check`
  native mode while retaining TypeScript 6.0.3 for the checker API.
- Aligned the pinned Node.js and npm toolchains with 26.12.1 and 12.2.1.

## [0.1.0] - 2026-10-24

This is the first tagged release. It includes the development history from the
first commit through `c7ae730`.

This file uses commit IDs from the rewritten history. The `v0.1.0` tag retains
the original IDs in its changelog as pre-rewrite provenance.

### 2026-10-23

- Added a `.vil` document model with byte-identical round trips, strict Vial v6
  keycode parsing, validation, layer operations with reference renumbering,
  semantic diffs, and atomic file writes (`a0fa00f`–`030bac7`).
- Added CLI validation, inspection, diffs, key edits, and layer operations.
  Commands that change files require `--write` or `--output` to write changes
  (`b10b7d3`, `8b312ee`).
- Added staged editing with undo, diff review, structured keycode edits, and
  editors for combos, macros, tap dances, key overrides, and alternate repeat
  (`fca01d3`, `0e8826a`, `b64e3f6`–`297a153`).
- Added the Tauri desktop app, definition-based geometry, cached Vial keyboard
  definitions, and read-only device requests for identity, geometry, and macro
  capacity (`70daee9`, `3038fac`, `f31ad21`, `1d0fc60`). Device configuration
  writes are not supported.

### 2026-10-24

- Added PNG export for nonblank layers, then included feature entries in the
  export (`1855d29`, `c90f6f0`).
- Reworked the editor around a canvas UI and added clearer keycode picker
  descriptions, theme selection, and keyboard/matrix views (`2670dc5`,
  `64b1488`, `355740f`, `ac419e7`).
- Added copy and drag actions for feature slots, with drag feedback
  (`f374fe4`, `53fb569`).
- Added CI configuration for checks and native bundle builds on macOS, Linux,
  and Windows (`5402502`). The commit history does not establish that these jobs
  passed.

The app edits layouts offline. Save the `.vil` file, then load it onto a keyboard
with Vial. Device requests remain read-only; no live configuration apply or
firmware update is supported.
