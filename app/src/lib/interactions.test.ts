import { describe, it, expect } from "vitest";
import { hitKey, marqueeKeys, bounds } from "./interactions";
import { moveAction } from "./macros";
import type { BoardKeyDto, MacroActionDto } from "./types";
const keys: BoardKeyDto[] = [
  { id: "a", row: 0, col: 0, x: 0, y: 0, w: 1, h: 1 },
  { id: "b", row: 0, col: 1, x: 2, y: 0, w: 1, h: 1 },
];
describe("board geometry interactions", () => {
  it("keeps holes empty and selects key centers inside the marquee", () => {
    expect(hitKey(keys, { x: 1.5, y: 0.5 })).toBeNull();
    expect(marqueeKeys(keys, { x: -0.1, y: -0.1 }, { x: 1.1, y: 1.1 })).toEqual(["a"]);
  });
  it("hits rotated and secondary key rectangles", () => {
    const rotated = { ...keys[0]!, r: 90, rx: 0, ry: 0, secondary: { x: 1, y: 0, w: 1, h: 1 } };
    expect(hitKey([rotated], { x: -0.5, y: 1.5 })).toBe("a");
    expect(hitKey([rotated], { x: 0.5, y: 0.5 })).toBeNull();
    expect(bounds([rotated]).x).toBeCloseTo(-1);
  });
});
describe("macro action reorder", () => {
  it("preserves action contents and rejects invalid moves", () => {
    const actions: MacroActionDto[] = [
      { kind: "text", text: "hello", keys: [], delay: 0 },
      { kind: "tap", text: "", keys: ["LGUI_T(KC_S)"], delay: 0 },
    ];
    expect(moveAction(actions, 0, 1)).toEqual([actions[1], actions[0]]);
    expect(actions[0]?.text).toBe("hello");
    expect(moveAction(actions, -1, 0)).toBe(actions);
  });
});
