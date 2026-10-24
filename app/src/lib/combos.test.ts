import { describe, expect, it } from "vitest";
import { comboOverlays } from "./combos";
import type { BoardKeyDto, ComboDto, KeyCellDto } from "./types";

const keys: BoardKeyDto[] = [
  { id: "a", row: 0, col: 0, x: 0, y: 0, w: 1, h: 1 },
  { id: "b", row: 0, col: 1, x: 1, y: 0, w: 1, h: 1 },
  { id: "c", row: 0, col: 2, x: 2, y: 0, w: 1, h: 1 },
];

function cell(id: string, wire: string): KeyCellDto {
  return {
    id,
    wire,
    label: wire,
    kind: "basic",
    changed: false,
    path: id,
    code: 0,
    description: wire,
    previous: wire,
  };
}

const cells: KeyCellDto[] = [cell("a", "KC_J"), cell("b", "KC_K"), cell("c", "KC_L")];

function combo(index: number, inputs: string[], output: string): ComboDto {
  const padded = [...inputs, "KC_NO", "KC_NO", "KC_NO", "KC_NO"].slice(0, 4);
  return { index, keys: padded, output, outputLabel: output.replace("KC_", "") };
}

describe("comboOverlays", () => {
  it("includes every position carrying a trigger keycode", () => {
    const duplicate = [cell("a", "KC_J"), cell("b", "KC_K"), cell("c", "KC_J")];
    expect(
      comboOverlays([combo(0, ["KC_J", "KC_K"], "KC_ESCAPE")], duplicate, keys)[0]?.keyIds,
    ).toEqual(["a", "c", "b"]);
  });
  it("places a combo between its keys", () => {
    const out = comboOverlays([combo(0, ["KC_J", "KC_K"], "KC_ESCAPE")], cells, keys);
    expect(out).toEqual([{ index: 0, keyIds: ["a", "b"], label: "ESCAPE", cx: 1, cy: 0.5 }]);
  });

  it("skips empty combos and combos whose keys are not on the layer", () => {
    const out = comboOverlays(
      [combo(0, [], "KC_NO"), combo(1, ["KC_J", "KC_Z"], "KC_A"), combo(2, ["KC_J"], "KC_A")],
      cells,
      keys,
    );
    expect(out).toEqual([]);
  });

  it("handles three-key combos", () => {
    const out = comboOverlays([combo(5, ["KC_L", "KC_J", "KC_K"], "KC_B")], cells, keys);
    expect(out[0]?.keyIds).toEqual(["c", "a", "b"]);
    expect(out[0]?.cx).toBeCloseTo(1.5);
  });
});
