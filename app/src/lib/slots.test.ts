import { describe, expect, it } from "vitest";
import {
  altRepeatSet,
  altRepeatText,
  baselineLabels,
  comboSet,
  comboText,
  currentLabels,
  flat,
  keyOverrideText,
  macroParts,
  macroSet,
  macroText,
  tapDanceText,
  type LabelOf,
} from "./slots";
import type { ViewModel } from "./types";

const labels: Record<string, string> = {
  "combo[0][0]": "A",
  "combo[0][1]": "S",
  "macro[3][0][1]": "Shift",
  "macro[3][1][1]": "A",
  "tap_dance[2][0]": "A",
  "tap_dance[2][2]": "B",
  "key_override[1].trigger": ".",
  "key_override[1].replacement": "Del",
  "alt_repeat_key[4].keycode": ",",
  "alt_repeat_key[4].alt_keycode": ".",
};
const label: LabelOf = (path) => labels[path];

describe("combos", () => {
  const combo = { index: 0, keys: ["KC_A", "KC_S", "KC_NO", "KC_NO"], output: "KC_ESCAPE", outputLabel: "Esc" };

  it("joins trigger labels and the output label", () => {
    expect(comboSet(combo)).toBe(true);
    expect(comboText(combo, label)).toBe("A + S → Esc");
  });

  it("falls back to wire names when a label is missing", () => {
    expect(comboText({ ...combo, index: 7, keys: ["KC_X", "KC_NO", "KC_NO", "KC_NO"], outputLabel: "" }, label)).toBe("KC_X → KC_ESCAPE");
  });

  it("uses placeholders for a missing trigger or output", () => {
    const outputOnly = { index: 0, keys: ["KC_NO", "KC_NO", "KC_NO", "KC_NO"], output: "KC_ESCAPE", outputLabel: "Esc" };
    expect(comboText(outputOnly, label)).toBe("… → Esc");
    const triggersOnly = { ...combo, output: "KC_NO", outputLabel: "" };
    expect(comboText(triggersOnly, label)).toBe("A + S → …");
  });

  it("shows the name when set and nothing for an empty slot", () => {
    expect(comboText(combo, label, "Undo")).toBe("Undo");
    const empty = { index: 5, keys: ["KC_NO", "KC_NO", "KC_NO", "KC_NO"], output: "KC_NO", outputLabel: "" };
    expect(comboSet(empty)).toBe(false);
    expect(comboText(empty, label, "Unused")).toBe("");
  });
});

describe("macros", () => {
  const macro = {
    index: 3,
    actions: [
      { kind: "down" as const, keys: ["KC_LSFT"], text: "", delay: 0 },
      { kind: "tap" as const, keys: ["KC_A", "KC_NO"], text: "", delay: 0 },
      { kind: "text" as const, keys: [], text: "hi", delay: 0 },
      { kind: "delay" as const, keys: [], text: "", delay: 50 },
      { kind: "up" as const, keys: ["KC_LSFT"], text: "", delay: 0 },
    ],
  };

  it("prefixes down and up actions and keeps text and delays", () => {
    expect(macroSet(macro)).toBe(true);
    expect(macroParts({ ...macro, actions: macro.actions.slice(0, 3) }, label)).toEqual(["↓Shift", "A", '"hi"']);
    expect(macroParts({ ...macro, actions: macro.actions.slice(3, 4) }, label)).toEqual(["50 ms"]);
  });

  it("reads the first four actions only", () => {
    expect(macroParts(macro, label)).toEqual(["↓Shift", "A", '"hi"', "50 ms"]);
    expect(macroText(macro, label)).toBe("↓Shift A \"hi\" 50 ms");
  });

  it("prefers the macro name and hides an empty macro", () => {
    expect(macroText(macro, label, "Wake")).toBe("Wake");
    const empty = { index: 1, actions: [] };
    expect(macroSet(empty)).toBe(false);
    expect(macroText(empty, label, "Wake")).toBe("");
  });
});

describe("tap dances", () => {
  it("names the set actions and joins them with a middle dot", () => {
    const td = { index: 2, onTap: "KC_A", onHold: "KC_NO", onDoubleTap: "KC_B", onTapHold: "KC_NO", tappingTerm: 200 };
    expect(tapDanceText(td, label)).toBe("Tap A · Double tap B");
  });
});

describe("key overrides", () => {
  const base = { index: 1, trigger: "KC_DOT", replacement: "KC_DEL", layers: 65535, triggerMods: 1, negativeModMask: 0, suppressedMods: 0, options: 7 };

  it("shows trigger mods, trigger and replacement", () => {
    expect(keyOverrideText(base, label)).toBe("LCTL+. → Del");
  });

  it("uses placeholders and hides an override with no keys", () => {
    expect(keyOverrideText({ ...base, triggerMods: 0, trigger: "KC_NO" }, label)).toBe("… → Del");
    expect(keyOverrideText({ ...base, trigger: "KC_NO", replacement: "KC_NO" }, label)).toBe("");
  });
});

describe("alt repeat keys", () => {
  const entry = { index: 4, keycode: "KC_COMMA", altKeycode: "KC_DOT", allowedMods: 0, options: 0 };

  it("shows key and alternate", () => {
    expect(altRepeatSet(entry)).toBe(true);
    expect(altRepeatText(entry, label)).toBe(", → .");
  });

  it("counts a mask as set and hides an empty entry", () => {
    expect(altRepeatText({ ...entry, altKeycode: "KC_NO" }, label)).toBe(", → …");
    expect(altRepeatSet({ index: 0, keycode: "KC_NO", altKeycode: "KC_NO", allowedMods: 0, options: 1 })).toBe(true);
    expect(altRepeatText({ index: 0, keycode: "KC_NO", altKeycode: "KC_NO", allowedMods: 0, options: 0 }, label)).toBe("");
  });
});

describe("label sources", () => {
  const view = {
    keycodes: { "layout[0][0][1]": { label: "W" } },
    baseline: { keycodes: { "layout[0][0][1]": { label: "B" } } },
  } as unknown as ViewModel;

  it("reads current labels and prefers the baseline label at the comparison point", () => {
    expect(currentLabels(view)("layout[0][0][1]")).toBe("W");
    expect(baselineLabels(view)("layout[0][0][1]")).toBe("B");
    expect(baselineLabels(null)("layout[0][0][1]")).toBeUndefined();
  });

  it("flattens keycap line breaks", () => {
    expect(flat("Esc\nL8")).toBe("Esc L8");
  });
});
