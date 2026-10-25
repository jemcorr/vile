import { describe, expect, it } from "vitest";
import { keyboardRange, matrixRange, nearestKey } from "./range";
import type { BoardKeyDto } from "./types";

/** A 3x4 grid of unit keys, row-major, id r{row}c{col}. */
function grid(rows = 3, cols = 4): BoardKeyDto[] {
  const keys: BoardKeyDto[] = [];
  for (let row = 0; row < rows; row++)
    for (let col = 0; col < cols; col++)
      keys.push({ id: `r${row}c${col}`, row, col, x: col, y: row, w: 1, h: 1 });
  return keys;
}

describe("keyboardRange", () => {
  it("grows to the block spanned by anchor and cursor", () => {
    const keys = grid();
    expect(keyboardRange(keys, "r0c0", "r0c0")).toEqual(["r0c0"]);
    expect(keyboardRange(keys, "r0c0", "r1c1").sort()).toEqual(["r0c0", "r0c1", "r1c0", "r1c1"]);
  });
  it("shrinks back when the cursor returns inside the anchor's side", () => {
    const keys = grid();
    expect(keyboardRange(keys, "r1c1", "r1c3").sort()).toEqual(["r1c1", "r1c2", "r1c3"]);
    expect(keyboardRange(keys, "r1c1", "r1c2").sort()).toEqual(["r1c1", "r1c2"]);
    expect(keyboardRange(keys, "r1c1", "r1c1")).toEqual(["r1c1"]);
  });
  it("covers the box on either side of the anchor", () => {
    const keys = grid();
    expect(keyboardRange(keys, "r1c2", "r0c0").sort()).toEqual(
      ["r0c0", "r0c1", "r0c2", "r1c0", "r1c1", "r1c2"].sort(),
    );
  });
  it("uses rotated centres for thumb keys", () => {
    // Rotated 90 degrees about its top-left corner (4, 0): the painted centre is (3.5, 0.5), not the
    // stored centre (4.5, 0.5). Key "u" sits at the stored centre, so it must stay out of the range.
    const thumb: BoardKeyDto = { id: "t", row: 3, col: 0, x: 4, y: 0, w: 1, h: 1, r: 90, rx: 4, ry: 0 };
    const unrotated: BoardKeyDto = { id: "u", row: 0, col: 4, x: 4, y: 0, w: 1, h: 1 };
    const keys = [...grid(1, 3), thumb, unrotated];
    expect(keyboardRange(keys, "r0c0", "t").sort()).toEqual(["r0c0", "r0c1", "r0c2", "t"].sort());
    expect(keyboardRange(keys, "t", "r0c0").sort()).toEqual(["r0c0", "r0c1", "r0c2", "t"].sort());
  });
  it("keeps the block on a staggered board, where neighbouring columns sit a little higher or lower", () => {
    // Columns 2 and 3 are raised by 0.27u, as on the corne. The block from r1c1 to r2c3 still holds all six keys.
    const keys = grid(3, 4).map((key) =>
      key.col >= 2 ? { ...key, y: key.y - 0.27 } : key,
    );
    expect(keyboardRange(keys, "r1c1", "r2c3").sort()).toEqual(
      ["r1c1", "r1c2", "r1c3", "r2c1", "r2c2", "r2c3"].sort(),
    );
    expect(keyboardRange(keys, "r1c1", "r2c2").sort()).toEqual(["r1c1", "r1c2", "r2c1", "r2c2"].sort());
  });
  it("returns nothing for an unknown cursor and the cursor alone for an unknown anchor", () => {
    const keys = grid();
    expect(keyboardRange(keys, "r0c0", "missing")).toEqual([]);
    expect(keyboardRange(keys, "missing", "r1c1")).toEqual(["r1c1"]);
  });
});

describe("matrixRange", () => {
  it("selects whole row and column ranges and shrinks past the anchor", () => {
    const keys = grid();
    expect(matrixRange(keys, "r1c1", "r2c3").sort()).toEqual(
      ["r1c1", "r1c2", "r1c3", "r2c1", "r2c2", "r2c3"].sort(),
    );
    expect(matrixRange(keys, "r1c1", "r2c0").sort()).toEqual(["r1c0", "r1c1", "r2c0", "r2c1"].sort());
    expect(matrixRange(keys, "r1c1", "r1c1")).toEqual(["r1c1"]);
  });
});

describe("nearestKey", () => {
  it("picks the candidate whose centre is closest to the point", () => {
    const keys = grid();
    expect(nearestKey(keys, { x: 2.9, y: 1.1 }, ["r0c0", "r1c3", "r2c3"])).toBe("r1c3");
    expect(nearestKey(keys, { x: 0.4, y: 0.4 })).toBe("r0c0");
    expect(nearestKey([], { x: 0, y: 0 })).toBeNull();
  });
});
