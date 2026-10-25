# Changelog

## [Unreleased]

- Updated direct npm, Rust, and GitHub Actions dependencies to their latest
  compatible stable releases. TypeScript remains on 6.0.3 because
  `svelte-check` does not declare TypeScript 7 support.
- Aligned the pinned Node.js and npm toolchains with 26.12.1 and 12.2.1.

## [0.1.0] - 2026-10-24

This is the first tagged release. It includes the development history from the
first commit through `bb6b343`.

This file uses commit IDs from the rewritten history. The `v0.1.0` tag retains
the original IDs in its changelog as pre-rewrite provenance.

### 2026-10-23

- Added a `.vil` document model with byte-identical round trips, strict Vial v6
  keycode parsing, validation, layer operations with reference renumbering,
  semantic diffs, and atomic file writes (`5dc0fb7`–`e6599ea`).
- Added CLI validation, inspection, diffs, key edits, and layer operations.
  Commands that change files require `--write` or `--output` to write changes
  (`3c81f35`, `b3f8f9a`).
- Added staged editing with undo, diff review, structured keycode edits, and
  editors for combos, macros, tap dances, key overrides, and alternate repeat
  (`63faa14`, `c1e6c7e`, `7516704`–`4011347`).
- Added the Tauri desktop app, definition-based geometry, cached Vial keyboard
  definitions, and read-only device requests for identity, geometry, and macro
  capacity (`546a8fe`, `ec5385c`, `c5b7035`, `dc51b43`). Device configuration
  writes are not supported.

### 2026-10-24

- Added PNG export for nonblank layers, then included feature entries in the
  export (`0a1c3ae`, `1283925`).
- Reworked the editor around a canvas UI and added clearer keycode picker
  descriptions, theme selection, and keyboard/matrix views (`03ea48d`,
  `663f7c9`, `9df0258`, `9e5ab3a`).
- Added copy and drag actions for feature slots, with drag feedback
  (`04ccb64`, `9b37fad`).
- Added CI configuration for checks and native bundle builds on macOS, Linux,
  and Windows (`7916d17`). The commit history does not establish that these jobs
  passed.

The app edits layouts offline. Save the `.vil` file, then load it onto a keyboard
with Vial. Device requests remain read-only; no live configuration apply or
firmware update is supported.
