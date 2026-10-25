# Changelog

## [Unreleased]

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
