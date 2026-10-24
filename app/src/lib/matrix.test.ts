import { describe, expect, it } from "vitest";
import type { BoardKeyDto } from "./types";
import {
  BOARD_VIEW_STORAGE_KEY,
  emptyMatrixCells,
  matrixGeometry,
  matrixGrid,
  matrixLabel,
  readBoardView,
  writeBoardView,
} from "./matrix";

function key(id: string, row: number, col: number, x = col * 1.5, y = row * 1.2): BoardKeyDto {
  return { id, row, col, x, y, w: 1, h: 1, r: 0.5, rx: 2, ry: 3, secondary: null, name: id } as BoardKeyDto;
}

describe("matrix cells", () => {
  const keys = [key("r0c0", 0, 0), key("r0c2", 0, 2), key("r1c1", 1, 1)];

  it("sizes the grid from the largest row and column", () => {
    expect(matrixGrid(keys)).toEqual({ rows: 2, cols: 3 });
    expect(matrixGrid([])).toEqual({ rows: 0, cols: 0 });
  });

  it("lists unoccupied positions in row-major order, including the empty cells", () => {
    expect(emptyMatrixCells(keys)).toEqual([
      { row: 0, col: 1 },
      { row: 1, col: 0 },
      { row: 1, col: 2 },
    ]);
  });

  it("reports no empty cells for a full grid", () => {
    expect(emptyMatrixCells([key("a", 0, 0), key("b", 0, 1), key("c", 1, 0), key("d", 1, 1)])).toEqual([]);
  });

  it("places each key at its column and row as a unit cell, keeping ids and order", () => {
    const geometry = matrixGeometry(keys);
    expect(geometry.map((k) => k.id)).toEqual(["r0c0", "r0c2", "r1c1"]);
    expect(geometry.map((k) => [k.x, k.y, k.w, k.h])).toEqual([
      [0, 0, 1, 1],
      [2, 0, 1, 1],
      [1, 1, 1, 1],
    ]);
    expect(geometry[0]).toMatchObject({ r: 0, rx: 0, ry: 0, secondary: null, row: 0, col: 0 });
  });

  it("labels a cell as row,col", () => {
    expect(matrixLabel({ row: 3, col: 6 })).toBe("3,6");
  });
});

describe("board view preference", () => {
  it("defaults to the physical keyboard, including when storage is missing or throws", () => {
    expect(readBoardView(null)).toBe("keyboard");
    expect(
      readBoardView({
        getItem: () => {
          throw new Error("blocked");
        },
      }),
    ).toBe("keyboard");
  });

  it("round-trips the stored view under its own key", () => {
    const store = new Map<string, string>();
    const storage = {
      getItem: (k: string) => store.get(k) ?? null,
      setItem: (k: string, v: string) => void store.set(k, v),
    };
    writeBoardView("matrix", storage);
    expect(store.get(BOARD_VIEW_STORAGE_KEY)).toBe("matrix");
    expect(readBoardView(storage)).toBe("matrix");
    expect(() =>
      writeBoardView("keyboard", {
        setItem: () => {
          throw new Error("full");
        },
      }),
    ).not.toThrow();
  });
});
