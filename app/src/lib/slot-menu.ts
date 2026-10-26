// Rules for the feature slot menu: which items a family has, their labels, and why an item is off.
// Rust decides what an operation really does; this only decides what the menu shows.
import type { ClipboardDto, SlotKind } from "./types";

/** Singular and plural names, as the menu and the toasts write them. */
export const NOUNS: Record<SlotKind, [string, string]> = {
  combo: ["combo", "combos"],
  macro: ["macro", "macros"],
  tapDance: ["tap dance", "tap dances"],
  keyOverride: ["key override", "key overrides"],
  altRepeat: ["alt repeat key", "alt repeat keys"],
};

export interface SlotCell {
  index: number;
  set: boolean;
  /** Key overrides and alt repeat keys only: the Enabled switch. */
  enabled?: boolean;
}

export interface SlotMenuContext {
  kind: SlotKind;
  /** The slots the menu acts on: the picked set, or the active slot alone. */
  targets: number[];
  cells: SlotCell[];
  clipboard: ClipboardDto | null;
  /** A macro draft is open and not applied, so every item waits. */
  draft: boolean;
  /** "⌘" on macOS, "Ctrl+" elsewhere. */
  modifier: string;
}

export type SlotMenuId = "cut" | "copy" | "paste" | "duplicate" | "rename" | "toggle" | "clear";

export interface SlotMenuItem {
  id: SlotMenuId;
  label: string;
  title: string;
  /** Why the item is off. Empty when it can run. */
  reason: string;
  shortcut: string;
  /** True on the first item of each group after the first. */
  divider: boolean;
}

const DRAFT_REASON = "Apply or revert the draft";

/** "a macro" for one, "an alt repeat key" for one of those, "2 macros" for more. */
export function heldPhrase(kind: SlotKind, count: number): string {
  if (count !== 1) return `${count} ${NOUNS[kind][1]}`;
  const singular = NOUNS[kind][0];
  return `${/^[aeiou]/.test(singular) ? "an" : "a"} ${singular}`;
}

/**
 * Where `n` copied slots land: the selection when it holds `n` slots, else `n` consecutive slots
 * from the single selected slot. A string is the reason they cannot land there. Mirrors
 * `slots::paste_targets` in Rust for the reason text.
 */
export function pasteTargets(n: number, selection: number[], count: number): number[] | string {
  const chosen = [...new Set(selection)].sort((a, b) => a - b);
  if (chosen.length === n) return chosen;
  if (chosen.length === 1) {
    const start = chosen[0]!;
    if (start + n > count) return `Needs ${n} slots from ${start}`;
    return Array.from({ length: n }, (_, i) => start + i);
  }
  return n === 1 ? "Select one slot" : `Select 1 or ${n} slots`;
}

/**
 * For each set target in ascending order, the first slot after it that is empty, is not a target
 * and is not already chosen, wrapping to 0. Null when one cannot be found.
 */
export function duplicateTargets(cells: SlotCell[], targets: number[]): number[] | null {
  const setAt = new Set(cells.filter((cell) => cell.set).map((cell) => cell.index));
  const picked = new Set(targets);
  const count = cells.length;
  const chosen: number[] = [];
  for (const target of [...targets].sort((a, b) => a - b)) {
    if (!setAt.has(target)) continue;
    let found: number | null = null;
    for (let step = 1; step <= count && found === null; step++) {
      const candidate = (target + step) % count;
      if (!setAt.has(candidate) && !picked.has(candidate) && !chosen.includes(candidate)) {
        found = candidate;
      }
    }
    if (found === null) return null;
    chosen.push(found);
  }
  return chosen;
}

/** The menu for the current targets. Items a family cannot have are absent. */
export function slotMenu(ctx: SlotMenuContext): SlotMenuItem[] {
  const { kind, targets, cells, clipboard, draft, modifier } = ctx;
  const [, plural] = NOUNS[kind];
  const n = targets.length;
  const setTargets = targets.filter((index) => cells.some((cell) => cell.index === index && cell.set));
  const withCount = (verb: string) => (n > 1 ? `${verb} ${n} ${plural}` : verb);
  const off = (reason: string) => (draft ? DRAFT_REASON : reason);

  const toggleOn = (index: number) => cells.find((cell) => cell.index === index)?.enabled === true;
  const allOn = setTargets.length > 0 && setTargets.every(toggleOn);
  const toggleVerb = allOn ? "Disable" : "Enable";
  const toggleLabel = n > 1 ? `${toggleVerb} ${setTargets.length}` : toggleVerb;
  const toggleReason =
    setTargets.length > 0 ? "" : n === 1 ? "Empty slot" : "No set slots";

  const duplicateReason =
    setTargets.length === 0
      ? "Nothing to duplicate"
      : duplicateTargets(cells, targets) !== null
        ? ""
        : n === 1
          ? "No empty slot"
          : `Needs ${setTargets.length} empty slots`;

  const pasteReason = (() => {
    if (!clipboard) return "Clipboard is empty";
    if (clipboard.kind === "keys") return "Clipboard holds keys";
    if (clipboard.kind === "layer") return "Clipboard holds a layer";
    const held = clipboard.feature ?? kind;
    if (held !== kind) return `Clipboard holds ${heldPhrase(held, clipboard.count)}`;
    const landing = pasteTargets(clipboard.count, targets, cells.length);
    return typeof landing === "string" ? landing : "";
  })();

  const emptyReason = setTargets.length === 0 ? "Already empty" : "";
  const groups: Omit<SlotMenuItem, "divider">[][] = [
    [
      {
        id: "cut",
        label: withCount("Cut"),
        title: "Copy, then empty. Keys that use it keep their number",
        reason: off(emptyReason),
        shortcut: `${modifier}X`,
      },
      {
        id: "copy",
        label: withCount("Copy"),
        title: "Copy to paste into another slot",
        reason: off(""),
        shortcut: `${modifier}C`,
      },
      {
        id: "paste",
        label: "Paste",
        title: "Replace with the copied slots",
        reason: off(pasteReason),
        shortcut: `${modifier}V`,
      },
      {
        id: "duplicate",
        label: withCount("Duplicate"),
        title: "Copy into the next empty slot",
        reason: off(duplicateReason),
        shortcut: `${modifier}D`,
      },
    ],
  ];
  if (kind === "combo" || kind === "macro") {
    groups.push([
      {
        id: "rename",
        label: "Rename…",
        title: "Name this slot; names show in pickers and the changes sheet",
        reason: off(n > 1 ? "Select one slot" : ""),
        shortcut: "",
      },
    ]);
  }
  if (kind === "keyOverride" || kind === "altRepeat") {
    groups.push([
      {
        id: "toggle",
        label: toggleLabel,
        title: "Turn on or off without opening it",
        reason: off(toggleReason),
        shortcut: "",
      },
    ]);
  }
  groups.push([
    {
      id: "clear",
      label: withCount("Clear"),
      title: "Empty the slot. Keys that use it keep their number",
      reason: off(emptyReason),
      shortcut: "Delete",
    },
  ]);
  return groups.flatMap((group, g) => group.map((item, i) => ({ ...item, divider: g > 0 && i === 0 })));
}
