import type { BoardKeyDto } from "./types";

// Matrix view: each key sits at its electrical grid position (x = column, y = row)
// as a unit cell. Keys keep their ids, so selection, drag, marquee and menus work
// unchanged. Only the main keyboard uses this view; the mini board and PNG export stay physical.

export type BoardView = "keyboard" | "matrix";
export const BOARD_VIEW_STORAGE_KEY = "vile.boardView";

export interface GridPosition {
  row: number;
  col: number;
}

export interface MatrixGrid {
  rows: number;
  cols: number;
}

export function matrixGrid(keys: BoardKeyDto[]): MatrixGrid {
  let rows = 0;
  let cols = 0;
  for (const key of keys) {
    rows = Math.max(rows, key.row + 1);
    cols = Math.max(cols, key.col + 1);
  }
  return { rows, cols };
}

/** Grid positions inside the matrix that no key occupies, in row-major order. */
export function emptyMatrixCells(keys: BoardKeyDto[]): GridPosition[] {
  const { rows, cols } = matrixGrid(keys);
  const taken = new Set(keys.map((key) => `${key.row},${key.col}`));
  const empty: GridPosition[] = [];
  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      if (!taken.has(`${row},${col}`)) empty.push({ row, col });
    }
  }
  return empty;
}

/** The same keys in matrix geometry, in the same order, so index-paired arrays still line up. */
export function matrixGeometry(keys: BoardKeyDto[]): BoardKeyDto[] {
  return keys.map((key) => ({
    ...key,
    x: key.col,
    y: key.row,
    w: 1,
    h: 1,
    r: 0,
    rx: 0,
    ry: 0,
    secondary: null,
  }));
}

export function matrixLabel(position: GridPosition): string {
  return `${position.row},${position.col}`;
}

/** Reads the stored view. Missing, unknown or blocked storage means the physical keyboard. */
export function readBoardView(storage: Pick<Storage, "getItem"> | null = safeStorage()): BoardView {
  try {
    return storage?.getItem(BOARD_VIEW_STORAGE_KEY) === "matrix" ? "matrix" : "keyboard";
  } catch {
    return "keyboard";
  }
}

/** Saves the view. Storage that throws is ignored. */
export function writeBoardView(
  view: BoardView,
  storage: Pick<Storage, "setItem"> | null = safeStorage(),
): void {
  try {
    storage?.setItem(BOARD_VIEW_STORAGE_KEY, view);
  } catch {
    /* Preference storage is optional. */
  }
}

function safeStorage(): Storage | null {
  try {
    return typeof localStorage === "undefined" ? null : localStorage;
  } catch {
    return null;
  }
}
