import { describe, expect, it } from "vitest";
import {
  duplicateTargets,
  pasteTargets,
  slotMenu,
  type SlotCell,
  type SlotMenuContext,
} from "./slot-menu";
import type { ClipboardDto } from "./types";

const cells = (set: number[], total: number, enabled: number[] = []): SlotCell[] =>
  Array.from({ length: total }, (_, index) => ({
    index,
    set: set.includes(index),
    enabled: enabled.includes(index),
  }));

function context(over: Partial<SlotMenuContext>): SlotMenuContext {
  return {
    kind: "macro",
    targets: [2],
    cells: cells([0, 2, 3], 16),
    clipboard: null,
    draft: false,
    modifier: "⌘",
    ...over,
  };
}

const ids = (items: ReturnType<typeof slotMenu>) => items.map((item) => item.id);
const item = (items: ReturnType<typeof slotMenu>, id: string) => items.find((i) => i.id === id)!;
const clip = (over: Partial<ClipboardDto>): ClipboardDto => ({
  kind: "feature",
  feature: "macro",
  count: 1,
  ...over,
});

describe("slot menu items", () => {
  it("offers rename only to combos and macros, and the enable switch only to overrides and alt repeat", () => {
    expect(ids(slotMenu(context({ kind: "combo", targets: [0], cells: cells([0], 8) })))).toEqual([
      "cut",
      "copy",
      "paste",
      "duplicate",
      "rename",
      "clear",
    ]);
    expect(ids(slotMenu(context({ kind: "tapDance", targets: [0], cells: cells([0], 8) })))).toEqual([
      "cut",
      "copy",
      "paste",
      "duplicate",
      "clear",
    ]);
    expect(ids(slotMenu(context({ kind: "keyOverride", targets: [0], cells: cells([0], 8) })))).toEqual([
      "cut",
      "copy",
      "paste",
      "duplicate",
      "toggle",
      "clear",
    ]);
    expect(ids(slotMenu(context({ kind: "altRepeat", targets: [0], cells: cells([0], 8) })))).toEqual([
      "cut",
      "copy",
      "paste",
      "duplicate",
      "toggle",
      "clear",
    ]);
  });

  it("starts a divider on the first item of each group after the first", () => {
    const items = slotMenu(context({ kind: "combo", targets: [0], cells: cells([0], 8) }));
    expect(items.filter((i) => i.divider).map((i) => i.id)).toEqual(["rename", "clear"]);
  });

  it("counts the targets in labels only when there are several", () => {
    const three = slotMenu(context({ kind: "combo", targets: [0, 1, 2], cells: cells([0, 1, 2], 8) }));
    expect(item(three, "cut").label).toBe("Cut 3 combos");
    expect(item(three, "copy").label).toBe("Copy 3 combos");
    expect(item(three, "duplicate").label).toBe("Duplicate 3 combos");
    expect(item(three, "clear").label).toBe("Clear 3 combos");
    expect(item(three, "paste").label).toBe("Paste");
    const one = slotMenu(context({ kind: "combo", targets: [0], cells: cells([0], 8) }));
    expect(item(one, "cut").label).toBe("Cut");
    expect(item(one, "clear").label).toBe("Clear");
  });

  it("flips the toggle label with the switch and counts only set targets", () => {
    const off = slotMenu(context({ kind: "keyOverride", targets: [4], cells: cells([4], 8) }));
    expect(item(off, "toggle").label).toBe("Enable");
    const on = slotMenu(context({ kind: "keyOverride", targets: [4], cells: cells([4], 8, [4]) }));
    expect(item(on, "toggle").label).toBe("Disable");
    const mixed = slotMenu(
      context({ kind: "keyOverride", targets: [4, 5, 9], cells: cells([4, 9], 16, [4]) }),
    );
    expect(item(mixed, "toggle").label).toBe("Enable 2");
    const all = slotMenu(
      context({ kind: "keyOverride", targets: [4, 5, 9], cells: cells([4, 9], 16, [4, 9]) }),
    );
    expect(item(all, "toggle").label).toBe("Disable 2");
  });

  it("uses the modifier in shortcuts and none for rename and toggle", () => {
    const items = slotMenu(context({ kind: "combo", targets: [0], cells: cells([0], 8), modifier: "Ctrl+" }));
    expect(item(items, "cut").shortcut).toBe("Ctrl+X");
    expect(item(items, "copy").shortcut).toBe("Ctrl+C");
    expect(item(items, "paste").shortcut).toBe("Ctrl+V");
    expect(item(items, "duplicate").shortcut).toBe("Ctrl+D");
    expect(item(items, "clear").shortcut).toBe("Delete");
    expect(item(items, "rename").shortcut).toBe("");
  });
});

describe("slot menu reasons", () => {
  it("disables every item while a macro draft is open", () => {
    const items = slotMenu(context({ draft: true }));
    expect(items.every((i) => i.reason === "Apply or revert the draft")).toBe(true);
  });

  it("explains empty cut, clear and duplicate targets", () => {
    const empty = slotMenu(context({ kind: "combo", targets: [1], cells: cells([0], 8) }));
    expect(item(empty, "cut").reason).toBe("Already empty");
    expect(item(empty, "clear").reason).toBe("Already empty");
    expect(item(empty, "duplicate").reason).toBe("Nothing to duplicate");
    expect(item(empty, "copy").reason).toBe("");
  });

  it("explains a duplicate without room", () => {
    const full = cells([...Array(16).keys()], 16);
    expect(item(slotMenu(context({ kind: "combo", targets: [3], cells: full })), "duplicate").reason).toBe(
      "No empty slot",
    );
    expect(
      item(slotMenu(context({ kind: "combo", targets: [0, 1], cells: full })), "duplicate").reason,
    ).toBe("Needs 2 empty slots");
  });

  it("explains paste from the clipboard and the selection", () => {
    const paste = (clipboard: ClipboardDto | null, targets: number[], kind: SlotMenuContext["kind"] = "combo") =>
      item(slotMenu(context({ kind, targets, cells: cells([0], 32), clipboard })), "paste").reason;
    expect(paste(null, [0])).toBe("Clipboard is empty");
    expect(paste({ kind: "keys", feature: null, count: 1 }, [0])).toBe("Clipboard holds keys");
    expect(paste({ kind: "layer", feature: null, count: 1 }, [0])).toBe("Clipboard holds a layer");
    expect(paste(clip({ feature: "macro", count: 1 }), [0])).toBe("Clipboard holds a macro");
    expect(paste(clip({ feature: "macro", count: 2 }), [0])).toBe("Clipboard holds 2 macros");
    expect(paste(clip({ feature: "combo", count: 2 }), [5])).toBe("");
    expect(paste(clip({ feature: "combo", count: 2 }), [5, 9, 10])).toBe("Select 1 or 2 slots");
    expect(paste(clip({ feature: "combo", count: 2 }), [31])).toBe("Needs 2 slots from 31");
  });

  it("explains rename with several targets and toggle without a set target", () => {
    expect(item(slotMenu(context({ kind: "combo", targets: [0, 1], cells: cells([0, 1], 8) })), "rename").reason).toBe(
      "Select one slot",
    );
    const single = slotMenu(context({ kind: "keyOverride", targets: [1], cells: cells([0], 8) }));
    expect(item(single, "toggle").reason).toBe("Empty slot");
    const several = slotMenu(context({ kind: "altRepeat", targets: [1, 2], cells: cells([0], 8) }));
    expect(item(several, "toggle").reason).toBe("No set slots");
  });
});

describe("duplicateTargets", () => {
  it("wraps to the start and skips set and chosen slots", () => {
    expect(duplicateTargets(cells([30, 31], 32), [30])).toEqual([0]);
    expect(duplicateTargets(cells([30], 32), [30])).toEqual([31]);
    expect(duplicateTargets(cells([0, 1], 4), [0, 1])).toEqual([2, 3]);
  });

  it("returns null when no empty slot is left", () => {
    expect(duplicateTargets(cells([0, 1, 2], 3), [0])).toBeNull();
  });
});

describe("pasteTargets", () => {
  it("matches the Rust rules", () => {
    expect(pasteTargets(2, [5], 32)).toEqual([5, 6]);
    expect(pasteTargets(2, [5, 9], 32)).toEqual([5, 9]);
    expect(pasteTargets(2, [9, 5], 32)).toEqual([5, 9]);
    expect(pasteTargets(2, [1, 2, 3], 32)).toBe("Select 1 or 2 slots");
    expect(pasteTargets(2, [31], 32)).toBe("Needs 2 slots from 31");
    expect(pasteTargets(2, [], 32)).toBe("Select 1 or 2 slots");
  });
});
