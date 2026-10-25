import { describe, expect, it } from "vitest";
import { gridCoordinates, keyboardRange, matrixRange, nearestKey } from "./range";
import type { BoardKeyDto } from "./types";

/** A 3x4 grid of unit keys, row-major, id r{row}c{col} (matrix row and col match the grid). */
function grid(rows = 3, cols = 4): BoardKeyDto[] {
  const keys: BoardKeyDto[] = [];
  for (let row = 0; row < rows; row++)
    for (let col = 0; col < cols; col++)
      keys.push({ id: `r${row}c${col}`, row, col, x: col, y: row, w: 1, h: 1 });
  return keys;
}

/**
 * A corne-like board: two halves of three rows by seven columns. The outer columns drop by up to 0.75u
 * (stagger), the right half is mirrored in x, and the thumbs form their own row below.
 * Electrical rows are 0-2 on the left and 4-6 on the right, so they never match the visual grid.
 */
const STAGGER = [0.375, 0.125, 0, 0, 0.125, 0.25, 0.75];
function corne(): BoardKeyDto[] {
  const keys: BoardKeyDto[] = [];
  for (let row = 0; row < 3; row++)
    for (let col = 0; col < 7; col++) {
      keys.push({ id: `L${row}${col}`, row, col, x: col + 0.5, y: row + 1 + STAGGER[col]!, w: 1, h: 1 });
      keys.push({ id: `R${row}${col}`, row: row + 4, col, x: 15 - col, y: row + 1 + STAGGER[col]!, w: 1, h: 1 });
    }
  return keys;
}

describe("gridCoordinates", () => {
  it("numbers columns left to right over both halves, the right half reversed in x", () => {
    const g = gridCoordinates(corne());
    expect(g.get("L00")).toEqual({ row: 0, col: 0 });
    expect(g.get("L11")).toEqual({ row: 1, col: 1 });
    // The right half's electrical column 0 sits at the far right: visual column 13 of 14.
    expect(g.get("R00")).toEqual({ row: 0, col: 13 });
    // Electrical column 6 of the right half sits next to the left half's column 6 across the gap.
    expect(g.get("R06")).toEqual({ row: 0, col: 7 });
  });
  it("keeps staggered keys of one row in one row", () => {
    const g = gridCoordinates(corne());
    const rows = new Set(["L00", "L01", "L02", "L03", "L04", "L05", "L06"].map((id) => g.get(id)!.row));
    expect(rows).toEqual(new Set([0]));
  });
  it("puts a thumb in its own row, below the main rows", () => {
    const thumb: BoardKeyDto = { id: "t", row: 3, col: 0, x: 1.5, y: 4.25, w: 1, h: 1 };
    const g = gridCoordinates([...corne().filter((k) => k.id.startsWith("L")), thumb]);
    expect(g.get("t")!.row).toBe(3);
    expect(g.get("L21")!.row).toBe(2);
  });
  it("places a rotated thumb by its painted centre, not its stored one", () => {
    // Stored centre (3, 4.75). Rotated 90 degrees about its top-left corner (2.5, 4.25), it paints at (2, 4.75).
    const thumb: BoardKeyDto = { id: "t", row: 3, col: 0, x: 2.5, y: 4.25, w: 1, h: 1, r: 90, rx: 2.5, ry: 4.25 };
    const g = gridCoordinates([...corne().filter((k) => k.id.startsWith("L")), thumb]);
    expect(g.get("t")!.col).toBe(g.get("L01")!.col);
  });
});

describe("keyboardRange on a corne-like board", () => {
  it("selects the block inside the left half", () => {
    // Row 1 to row 2, columns 1 to 3: exactly six keys.
    expect(keyboardRange(corne(), "L11", "L23").sort()).toEqual(
      ["L11", "L12", "L13", "L21", "L22", "L23"].sort(),
    );
  });
  it("selects a block that spans both halves, the way it looks", () => {
    // From the left half's column 1 across the gap to the right half's electrical column 2 (visual column 11).
    expect(keyboardRange(corne(), "L11", "R12").sort()).toEqual(
      ["L11", "L12", "L13", "L14", "L15", "L16", "R16", "R15", "R14", "R13", "R12"].sort(),
    );
  });
  it("shrinks back when the cursor returns past the anchor", () => {
    const keys = corne();
    expect(keyboardRange(keys, "L11", "L13").sort()).toEqual(["L11", "L12", "L13"].sort());
    expect(keyboardRange(keys, "L11", "L12").sort()).toEqual(["L11", "L12"].sort());
    expect(keyboardRange(keys, "L11", "L10").sort()).toEqual(["L10", "L11"].sort());
    expect(keyboardRange(keys, "L11", "L11")).toEqual(["L11"]);
  });
  it("takes thumbs whose painted column lies in the block, and no others", () => {
    const thumbs: BoardKeyDto[] = [
      { id: "tA", row: 3, col: 0, x: 1.5, y: 4.25, w: 1, h: 1 },
      { id: "tB", row: 3, col: 1, x: 2.5, y: 4.25, w: 1, h: 1 },
      // Stored centre (3, 4.75); rotated 90 about its top-left corner (2.5, 4.25), it paints at (2, 4.75): column 1.
      { id: "tC", row: 3, col: 2, x: 2.5, y: 4.25, w: 1, h: 1, r: 90, rx: 2.5, ry: 4.25 },
    ];
    const keys = [...corne().filter((k) => k.id.startsWith("L")), ...thumbs];
    // Column 2 only (from L12 to tB): tB joins, tC (painted in column 1) does not.
    expect(keyboardRange(keys, "L12", "tB").sort()).toEqual(["L12", "L22", "tB"].sort());
    // Columns 1 to 2: all three thumbs join the block.
    expect(keyboardRange(keys, "L11", "tB").sort()).toEqual(
      ["L11", "L12", "L21", "L22", "tA", "tB", "tC"].sort(),
    );
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
