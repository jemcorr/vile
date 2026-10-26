import { describe, expect, it } from "vitest";
import { changeCount, changeGroups, changedSlots, compareErrorText, shortLabel } from "./changes";
import type { DiffLineDto, IntentDto, KeycodeView, KeyCellDto, StoryDto, ViewModel } from "./types";

const parts = { kind: "tap", base: 4, wrapMods: 0, hold: { kind: "none" } } as const;
const keycode = (code: number, label: string, description = label): KeycodeView => ({
  code,
  wire: label === "No key" ? "KC_NO" : `KC_${label}`,
  label,
  description,
  tooltip: description,
  category: "Basic",
  parts,
}) as KeycodeView;
const NO_KEY = keycode(0, "", "No key");
const B = keycode(5, "B");
const A = keycode(4, "A");
const W = keycode(24, "W");

const cell = (id: string, path: string, value: KeycodeView): KeyCellDto => ({
  id,
  wire: value.wire,
  label: value.label,
  kind: "basic",
  changed: false,
  path,
  code: value.code,
  description: value.description,
  tooltip: value.tooltip,
  previous: "KC_NO",
});

const line = (partial: Partial<DiffLineDto> & Pick<DiffLineDto, "section">): DiffLineDto => ({
  line: "",
  layer: null,
  keyId: null,
  index: null,
  field: null,
  old: "",
  new: "",
  oldDescription: "",
  newDescription: "",
  ...partial,
});

/** Two keys on layer 0 (r0c0, r0c1), one knob, one combo slot, names for layer 0. */
function makeView(over: Partial<ViewModel> & { keycodes?: Record<string, KeycodeView>; baselineKeycodes?: Record<string, KeycodeView> } = {}): ViewModel {
  const { keycodes = {}, baselineKeycodes = {}, ...rest } = over;
  const current: Record<string, KeycodeView> = { "layout[0][0][0]": A, "layout[0][0][1]": NO_KEY, ...keycodes };
  return {
    names: { layers: { 0: "Base" }, macros: {}, combos: {} },
    keycodes: current,
    pendingDiff: [],
    diff: [],
    board: {
      id: "test",
      name: "test",
      keys: [
        { id: "r0c0", row: 0, col: 0, x: 0, y: 0, w: 1, h: 1, name: null },
        { id: "r0c1", row: 0, col: 1, x: 1, y: 0, w: 1, h: 1, name: null },
      ],
    },
    layers: [[cell("r0c0", "layout[0][0][0]", A), cell("r0c1", "layout[0][0][1]", NO_KEY)]],
    combos: [],
    macros: [],
    tapDances: [],
    keyOverrides: [],
    altRepeatKeys: [],
    baseline: {
      keycodes: { "layout[0][0][0]": A, "layout[0][0][1]": B, ...baselineKeycodes },
      combos: [],
      macros: [],
      tapDances: [],
      keyOverrides: [],
      altRepeatKeys: [],
      names: { layers: {}, macros: {}, combos: {} },
    },
    ...rest,
  } as unknown as ViewModel;
}

const layoutLine = (keyId: string, old: string, neu: string, oldDescription: string, newDescription: string) =>
  line({ section: "layout", layer: 0, keyId, line: `L0 ${keyId}`, old, new: neu, oldDescription, newDescription });

describe("layer changes", () => {
  it("groups key changes under the layer name with a chip and a key jump", () => {
    const view = makeView({ diff: [layoutLine("r0c1", "KC_B", "KC_NO", "B", "No key")] });
    const [group] = changeGroups(view);
    expect(group!.title).toBe("Layer 0 · Base");
    expect(group!.count).toBe("1 key");
    expect(group!.chips.map((chip) => chip.text)).toEqual(["B → No key"]);
    expect(group!.chips[0]!.title.startsWith("r0c1:")).toBe(true);
    expect(group!.chips[0]!.jump).toEqual({ kind: "keys", layer: 0, keyIds: ["r0c1"] });
    expect(group!.jump).toEqual({ kind: "layer", layer: 0 });
  });

  it("counts two changed keys", () => {
    const view = makeView({
      keycodes: { "layout[0][0][0]": W },
      diff: [layoutLine("r0c1", "KC_B", "KC_NO", "B", "No key"), layoutLine("r0c0", "KC_A", "KC_W", "A", "W")],
    });
    const [group] = changeGroups(view);
    expect(group!.count).toBe("2 keys");
    expect(changeCount(changeGroups(view))).toBe(2);
  });

  it("skips a key that is not on the board", () => {
    const view = makeView({ diff: [layoutLine("r9c9", "KC_B", "KC_NO", "B", "No key")] });
    expect(changeGroups(view)).toEqual([]);
  });

  it("words a knob change as a row", () => {
    const view = makeView({
      keycodes: { "encoder_layout[0][0][0]": keycode(0x81, "Vol−", "Vol−") },
      baselineKeycodes: { "encoder_layout[0][0][0]": keycode(0x80, "Vol+", "Vol+") },
      diff: [line({ section: "encoder", layer: 0, index: 0, field: "cw", old: "KC_VOLU", new: "KC_VOLD", oldDescription: "Vol+", newDescription: "Vol−" })],
    });
    const [group] = changeGroups(view);
    expect(group!.count).toBe("1 knob");
    expect(group!.rows[0]!.text).toBe("Knob 1 clockwise");
    expect(group!.rows[0]!.detail).toBe("Vol+ → Vol−");
  });
});

describe("feature slot changes", () => {
  const combo = (keys: string[], output: string, outputLabel: string) => ({ index: 0, keys, output, outputLabel });
  const triggers = ["KC_S", "KC_T", "KC_NO", "KC_NO"];

  it("compares a changed combo with its baseline text", () => {
    const view = makeView({
      combos: [combo(triggers, "KC_ESCAPE", "Esc")] as never,
      keycodes: { "combo[0][0]": keycode(0x10, "S"), "combo[0][1]": keycode(0x11, "T") },
      baselineKeycodes: { "combo[0][0]": keycode(0x10, "S"), "combo[0][1]": keycode(0x12, "H") },
      diff: [line({ section: "combo", index: 0, line: "combo 0" })],
    });
    view.baseline.combos = [combo(["KC_S", "KC_H", "KC_NO", "KC_NO"], "KC_ESCAPE", "Esc")] as never;
    const [group] = changeGroups(view);
    expect(group!.title).toBe("Combos");
    expect(group!.rows[0]!.text).toBe("Combo 0");
    expect(group!.rows[0]!.detail).toBe("S + T → Esc · was S + H → Esc");
    expect(group!.rows[0]!.jump).toEqual({ kind: "slot", panel: "combos", index: 0 });
  });

  it("says New for a combo that was empty and Removed for one that is now empty", () => {
    const empty = combo(["KC_NO", "KC_NO", "KC_NO", "KC_NO"], "KC_NO", "");
    const added = makeView({
      combos: [combo(triggers, "KC_ESCAPE", "Esc")] as never,
      baselineKeycodes: {},
      keycodes: { "combo[0][0]": keycode(0x10, "S"), "combo[0][1]": keycode(0x11, "T") },
      diff: [line({ section: "combo", index: 0, line: "combo 0" })],
    });
    added.baseline.combos = [empty] as never;
    expect(changeGroups(added)[0]!.rows[0]!.detail).toBe("New: S + T → Esc");

    const removed = makeView({
      baselineKeycodes: { "combo[0][0]": keycode(0x10, "S"), "combo[0][1]": keycode(0x11, "T") },
      diff: [line({ section: "combo", index: 0, line: "combo 0" })],
    });
    removed.combos = [empty] as never;
    removed.baseline.combos = [combo(triggers, "KC_ESCAPE", "Esc")] as never;
    expect(changeGroups(removed)[0]!.rows[0]!.detail).toBe("Removed: S + T → Esc");
  });

  it("words override, tap dance and alt repeat fields and joins them", () => {
    const view = makeView({
      diff: [
        line({ section: "tapDance", index: 2, field: "term", old: "200", new: "180" }),
        line({ section: "keyOverride", index: 0, field: "layers", old: "65535", new: "65531" }),
        line({ section: "keyOverride", index: 0, field: "trigger_mods", old: "0", new: "1" }),
        line({ section: "keyOverride", index: 0, field: "options", old: "7", new: "135" }),
        line({ section: "altRepeatKey", index: 3, field: "allowed_mods", old: "0", new: "4" }),
      ],
    });
    const groups = changeGroups(view);
    const byTitle = Object.fromEntries(groups.map((group) => [group.title, group]));
    expect(byTitle["Tap dances"]!.rows[0]!.detail).toBe("Tapping term: 200 → 180 ms");
    expect(byTitle["Key overrides"]!.rows[0]!.detail).toBe("Layers: −L2 · Trigger mods: +L Ctrl · Enabled on");
    expect(byTitle["Alt repeat"]!.rows[0]!.detail).toBe("Allowed mods: +L Alt");
    expect(byTitle["Alt repeat"]!.rows[0]!.jump).toEqual({ kind: "slot", panel: "altRepeat", index: 3 });
  });

  it("orders slot rows by index", () => {
    const view = makeView({
      diff: [line({ section: "tapDance", index: 4, field: "term", old: "1", new: "2" }), line({ section: "tapDance", index: 1, field: "term", old: "1", new: "3" })],
    });
    const [group] = changeGroups(view);
    expect(group!.rows.map((row) => row.text)).toEqual(["Tap dance 1", "Tap dance 4"]);
    expect(group!.jump).toEqual({ kind: "slot", panel: "tapDances", index: 1 });
  });
});

describe("names, settings and file", () => {
  it("words added, removed and renamed names and links layer and slot names", () => {
    const view = makeView({
      diff: [
        line({ section: "names", index: 0, field: "layer", old: "", new: "Base" }),
        line({ section: "names", index: 2, field: "layer", old: "Numbers", new: "" }),
        line({ section: "names", index: 1, field: "combo", old: "A", new: "B" }),
      ],
    });
    const groups = changeGroups(view);
    const names = groups.find((group) => group.id === "names")!;
    expect(names.rows.map((row) => row.detail)).toEqual([
      "Named “Base”",
      "Name removed (was “Numbers”)",
      "“A” → “B”",
    ]);
    expect(names.rows[0]!.jump).toEqual({ kind: "layer", layer: 0 });
    expect(names.rows[2]!.jump).toEqual({ kind: "slot", panel: "combos", index: 1 });
    expect(names.count).toBe("3 names");
  });

  it("takes the names-file retry line from the pending diff once", () => {
    const retry = line({ section: "names", index: null, old: "unsaved", new: "retry save" });
    const view = makeView({ pendingDiff: [retry] });
    const names = changeGroups(view).find((group) => group.id === "names")!;
    expect(names.rows).toHaveLength(1);
    expect(names.rows[0]).toMatchObject({ text: "Names file", detail: "Not saved yet. Saving retries it.", jump: null });
    const withLine = makeView({ pendingDiff: [retry], diff: [retry] });
    expect(changeGroups(withLine).find((group) => group.id === "names")!.rows).toHaveLength(1);
  });

  it("words settings, meta and presence lines", () => {
    const view = makeView({
      diff: [
        line({ section: "settings", field: "debounce", old: "(none)", new: "5" }),
        line({ section: "meta", field: "layout_options", old: "0", new: "3" }),
        line({ section: "combo", old: "present", new: "(absent)" }),
      ],
    });
    const file = changeGroups(view).find((group) => group.id === "file")!;
    const settings = changeGroups(view).find((group) => group.id === "settings")!;
    expect(settings.rows[0]).toMatchObject({ text: "QMK setting debounce", detail: "not set → 5", jump: null });
    expect(file.title).toBe("File");
    expect(file.rows.map((row) => [row.text, row.detail])).toEqual([
      ["Layout options", "0 → 3"],
      ["Combos section", "Removed"],
    ]);
  });

  it("orders groups by rank with layers first, and counts every chip and row", () => {
    const view = makeView({
      diff: [
        line({ section: "meta", field: "version", old: "1", new: "2" }),
        line({ section: "names", index: 0, field: "layer", old: "", new: "Base" }),
        line({ section: "tapDance", index: 0, field: "term", old: "1", new: "2" }),
        line({ section: "encoder", layer: 0, index: 0, field: "cw", old: "KC_VOLU", new: "KC_VOLD", oldDescription: "Vol+", newDescription: "Vol−" }),
        layoutLine("r0c1", "KC_B", "KC_NO", "B", "No key"),
      ],
    });
    const groups = changeGroups(view);
    expect(groups.map((group) => group.title)).toEqual(["Layer 0 · Base", "Tap dances", "Names", "File"]);
    expect(changeCount(groups)).toBe(5);
    expect(changeGroups(makeView({ diff: [] }))).toEqual([]);
  });
});

/** A story over `makeView`, with the rest and the reference defaulting to the view's diff and baseline. */
function withStory(intents: IntentDto[], over: Partial<ViewModel> = {}, rest: DiffLineDto[] = []): ViewModel {
  const base = makeView(over);
  const story: StoryDto = { intents, rest, reference: base.baseline };
  return { ...base, story } as ViewModel;
}

const at = (layer: number, keyId: string) => ({ layer, keyId });

describe("intent rows", () => {
  it("words a cleared layer as one row with its key count", () => {
    const groups = changeGroups(withStory([{ kind: "layerFilled", layer: 0, value: "KC_NO", keys: 41 }]));
    expect(groups.map((group) => group.id)).toEqual(["layer-0"]);
    const [row] = groups[0]!.rows;
    expect(row).toMatchObject({ text: "Cleared all keys", detail: "41 keys", jump: { kind: "layer", layer: 0 } });
    expect(groups[0]!.count).toBe("1 change");
    expect(changeCount(groups)).toBe(1);
  });

  it("words a transparent fill as one row", () => {
    const [group] = changeGroups(withStory([{ kind: "layerFilled", layer: 1, value: "KC_TRNS", keys: 26 }]));
    expect(group!.id).toBe("layer-1");
    expect(group!.rows[0]!.text).toBe("Made all keys transparent");
    expect(group!.rows[0]!.detail).toBe("26 keys");
  });

  it("words a copied layer with its source name and files it under the target layer", () => {
    const view = withStory([{ kind: "layerCopied", from: 1, to: 3, keys: 26 }], {
      names: { layers: { 1: "Symbols" }, macros: {}, combos: {} },
    });
    const [group] = changeGroups(view);
    expect(group!.id).toBe("layer-3");
    expect(group!.rows[0]).toMatchObject({ text: "Copied from layer 1 · Symbols", detail: "26 keys", jump: { kind: "layer", layer: 3 } });
  });

  it("shows a swap as one chip with the current labels and selects both keys", () => {
    // After the swap r0c0 holds W and r0c1 holds L.
    const view = withStory([{ kind: "keysSwapped", a: at(0, "r0c0"), b: at(0, "r0c1") }], {
      layers: [[cell("r0c0", "layout[0][0][0]", W), cell("r0c1", "layout[0][0][1]", A)]],
      keycodes: { "layout[0][0][0]": W, "layout[0][0][1]": A },
    });
    const [group] = changeGroups(view);
    const [chip] = group!.chips;
    expect(chip!.text).toBe("W ⇄ A");
    expect(chip!.jump).toEqual({ kind: "keys", layer: 0, keyIds: ["r0c0", "r0c1"] });
    expect(group!.count).toBe("1 change");
    expect(changeCount(changeGroups(view))).toBe(1);
  });

  it("names the other layer on a swap across layers and selects only the first key", () => {
    const view = withStory([{ kind: "keysSwapped", a: at(0, "r0c0"), b: at(2, "r0c1") }], {
      layers: [
        [cell("r0c0", "layout[0][0][0]", W), cell("r0c1", "layout[0][0][1]", NO_KEY)],
        [],
        [cell("r0c0", "layout[2][0][0]", NO_KEY), cell("r0c1", "layout[2][0][1]", A)],
      ],
    });
    const [group] = changeGroups(view);
    expect(group!.chips[0]).toMatchObject({ text: "W ⇄ A (layer 2)", jump: { kind: "keys", layer: 0, keyIds: ["r0c0"] } });
  });

  it("words a moved key, names the emptied key and the key it replaced", () => {
    const view = withStory([{ kind: "keyMoved", from: at(0, "r0c1"), to: at(0, "r0c0") }], {
      layers: [[cell("r0c0", "layout[0][0][0]", W), cell("r0c1", "layout[0][0][1]", NO_KEY)]],
      keycodes: { "layout[0][0][0]": W },
    });
    const [group] = changeGroups(view);
    expect(group!.chips[0]).toMatchObject({
      text: "Moved W",
      title: "Moved W from r0c1 to r0c0. r0c1 is now empty. It replaced A.",
      jump: { kind: "keys", layer: 0, keyIds: ["r0c0"] },
    });
  });

  it("does not mention a replacement when the target was empty", () => {
    const view = withStory([{ kind: "keyMoved", from: at(0, "r0c1"), to: at(0, "r0c0") }], {
      layers: [[cell("r0c0", "layout[0][0][0]", W), cell("r0c1", "layout[0][0][1]", NO_KEY)]],
      baseline: { ...makeView().baseline, keycodes: { "layout[0][0][1]": B } },
    });
    const [group] = changeGroups(view);
    expect(group!.chips[0]!.title).toBe("Moved W from r0c1 to r0c0. r0c1 is now empty.");
  });

  it("words a cleared group of keys as a row that selects them all", () => {
    const [group] = changeGroups(withStory([{ kind: "keysFilled", layer: 0, value: "KC_NO", keys: ["r0c0", "r0c1"] }]));
    expect(group!.rows[0]).toMatchObject({
      text: "Cleared 2 keys",
      jump: { kind: "keys", layer: 0, keyIds: ["r0c0", "r0c1"] },
    });
  });

  it("words swapped and copied slots with the slot word and the first slot as jump", () => {
    const groups = changeGroups(
      withStory([
        { kind: "slotsSwapped", feature: "combo", a: 1, b: 4 },
        { kind: "slotCopied", feature: "altRepeat", from: 0, to: 2 },
      ]),
    );
    expect(groups.map((group) => group.id)).toEqual(["slot-combo", "slot-altRepeatKey"]);
    expect(groups[0]!.rows[0]).toMatchObject({ text: "Swapped combo 1 and combo 4", jump: { kind: "slot", panel: "combos", index: 1 } });
    expect(groups[1]!.rows[0]).toMatchObject({ text: "Copied alt repeat 0 to alt repeat 2", jump: { kind: "slot", panel: "altRepeat", index: 2 } });
    expect(changeCount(groups)).toBe(2);
  });

  it("puts intent rows in front of the diff rows of their group and counts them together", () => {
    const view = withStory(
      [{ kind: "keysSwapped", a: at(0, "r0c0"), b: at(0, "r0c1") }],
      {
        layers: [[cell("r0c0", "layout[0][0][0]", W), cell("r0c1", "layout[0][0][1]", A)]],
        keycodes: { "layout[0][0][0]": W, "layout[0][0][1]": A },
      },
      [
        line({ section: "encoder", layer: 0, index: 0, field: "cw", old: "KC_VOLU", new: "KC_VOLD", oldDescription: "Vol+", newDescription: "Vol−" }),
      ],
    );
    const [group] = changeGroups(view);
    expect(group!.chips.map((chip) => chip.text)).toEqual(["W ⇄ A"]);
    expect(group!.rows.map((row) => row.text)).toEqual(["Knob 1 clockwise"]);
    expect(group!.count).toBe("2 changes");
    expect(changeCount(changeGroups(view))).toBe(2);
  });

  it("keeps the key and knob counts for a layer with no intent", () => {
    const [group] = changeGroups(
      makeView({
        diff: [
          layoutLine("r0c1", "KC_B", "KC_NO", "B", "No key"),
          line({ section: "encoder", layer: 0, index: 0, field: "cw", old: "KC_VOLU", new: "KC_VOLD", oldDescription: "Vol+", newDescription: "Vol−" }),
        ],
      }),
    );
    expect(group!.count).toBe("1 key · 1 knob");
  });

  it("reads the story's rest and reference by default, and the view's diff without a story", () => {
    const restLine = line({ section: "meta", field: "version", old: "1", new: "2" });
    const story = withStory([], {
      diff: [layoutLine("r0c1", "KC_B", "KC_NO", "B", "No key")],
    }, [restLine]);
    expect(changeGroups({ ...story, story: null } as ViewModel).length).toBe(1);
    const withRest = changeGroups(story);
    expect(withRest.map((group) => group.title)).toEqual(["File"]);
  });
});

describe("layer order", () => {
  const reorder = (order: number[], renumbered = true): IntentDto => ({ kind: "layersReordered", order, renumbered });
  const savedNames = (layers: Record<number, string>) => ({
    ...makeView().baseline,
    names: { layers, macros: {}, combos: {} },
  });

  it("words a swap of two layers as one row in a group ahead of the layers", () => {
    const view = withStory(
      [reorder([0, 1, 2, 3, 4, 5, 6, 8, 7, 9]), { kind: "keysSwapped", a: at(0, "r0c0"), b: at(0, "r0c1") }],
      { baseline: savedNames({ 7: "Symbols", 8: "Navigation" }), layers: [[cell("r0c0", "layout[0][0][0]", W), cell("r0c1", "layout[0][0][1]", A)]] },
    );
    const groups = changeGroups(view);
    expect(groups.map((group) => group.id)).toEqual(["layer-order", "layer-0"]);
    const [order] = groups;
    expect(order).toMatchObject({ title: "Layer order", count: "1 change", jump: { kind: "layer", layer: 7 } });
    expect(order!.rows[0]).toMatchObject({
      text: "Swapped Symbols (7) and Navigation (8)",
      detail: "Layer keys renumbered",
      title: "Swapped Symbols (7) and Navigation (8). Layer 7 is now layer 8. Layer 8 is now layer 7.",
    });
    expect(changeCount(groups)).toBe(2);
  });

  it("words a single move as the layer moved and its new position", () => {
    const view = withStory([reorder([0, 9, 1, 2, 3, 4, 5, 6, 7, 8])], {
      baseline: savedNames({ 9: "Functions" }),
    });
    const [group] = changeGroups(view);
    expect(group!.rows[0]).toMatchObject({
      text: "Moved Functions (9) to position 1",
      jump: { kind: "layer", layer: 1 },
    });
    expect(group!.rows[0]!.title).toContain("Layer 9 is now layer 1.");
    expect(group!.rows[0]!.title).toContain("Layer 1 is now layer 2.");
  });

  it("counts the layers that moved for a general order and names unnamed layers by number", () => {
    const view = withStory([reorder([0, 6, 2, 3, 4, 5, 1, 8, 7, 9], false)]);
    const [group] = changeGroups(view);
    expect(group!.rows[0]).toMatchObject({ text: "Reordered 4 layers", detail: "" });
    expect(group!.rows[0]!.jump).toEqual({ kind: "layer", layer: 1 });
    const swapped = withStory([reorder([0, 1, 2, 3, 4, 5, 6, 8, 7, 9])]);
    expect(changeGroups(swapped)[0]!.rows[0]!.text).toBe("Swapped layer 7 and layer 8");
  });
});

describe("helpers", () => {
  it("shortens labels to one line", () => {
    expect(shortLabel(null)).toBe("");
    expect(shortLabel({ code: 0, label: "" })).toBe("No key");
    expect(shortLabel({ code: 1, label: "" })).toBe("Transparent");
    expect(shortLabel({ code: 5, label: "Esc\nL8" })).toBe("Esc L8");
    expect(shortLabel({ code: 5, label: "S\n⌘" })).toBe("S ⌘");
  });

  it("marks changed combo and macro slots, including their names", () => {
    const view = makeView({
      diff: [
        line({ section: "combo", index: 0, line: "combo 0" }),
        line({ section: "names", index: 3, field: "macro", old: "", new: "Wake" }),
        line({ section: "layout", layer: 0, keyId: "r0c0", line: "" }),
      ],
    });
    const sets = changedSlots(view);
    expect([...sets.combo]).toEqual([0]);
    expect([...sets.macro]).toEqual([3]);
    expect([...sets.tapDance]).toEqual([]);
  });

  it("explains a failed comparison in plain words", () => {
    const file = "corne.vil";
    expect(compareErrorText("cannot read the git HEAD version: git failed: fatal: not a git repository", file)).toBe(
      "Can't compare with the last commit: corne.vil is not in a Git repository.",
    );
    expect(compareErrorText("fatal: invalid object name 'HEAD'.", file)).toBe("Can't compare with the last commit: corne.vil has no commit yet.");
    expect(compareErrorText("exists on disk, but not in 'HEAD'", file)).toBe("Can't compare with the last commit: corne.vil has no commit yet.");
    expect(compareErrorText("git is not installed", file)).toBe("Can't compare with the last commit: Git is not installed.");
    expect(compareErrorText("permission denied", file)).toBe("Can't compare with the last commit: permission denied");
  });
});
