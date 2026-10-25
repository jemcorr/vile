// Spreadsheet-style range selection. The anchor is the key a range grows from; the cursor is the
// key the arrows move. Pure functions only; the App decides which view's rule to use.
import type { BoardKeyDto } from "./types";
import { rotate, type Point } from "./interactions";

/** Keys whose centres lie within this many units of a row's first (highest) key share that row. */
const ROW_BAND = 0.9;
/** Keys whose centres lie within this many units of a column's first key share that column. */
const COL_BAND = 0.25;

function centre(key: BoardKeyDto): Point {
  return rotate(key, { x: key.x + key.w / 2, y: key.y + key.h / 2 });
}

/**
 * Index of the band each value falls in. Values are sorted, and a value starts a new band once it lies
 * more than `band` past the band's first value. Staggered keys stay in one band; neighbours a full row
 * or column apart do not.
 */
function bandIndices(values: number[], band: number): number[] {
  const order = values.map((_, i) => i).sort((a, b) => values[a]! - values[b]!);
  const out = new Array<number>(values.length).fill(0);
  let index = -1;
  let start = -Infinity;
  for (const i of order) {
    if (values[i]! > start + band) {
      index += 1;
      start = values[i]!;
    }
    out[i] = index;
  }
  return out;
}

/**
 * The visual grid of the physical layout, in keyboard view: rows are bands of rotated centre y (thumbs
 * fall in their own row) and columns are bands of rotated centre x, numbered left to right over the
 * whole board. Electrical matrix positions play no part here.
 */
export function gridCoordinates(keys: BoardKeyDto[]): Map<string, { row: number; col: number }> {
  const at = keys.map((key) => centre(key));
  const rows = bandIndices(at.map((p) => p.y), ROW_BAND);
  const cols = bandIndices(at.map((p) => p.x), COL_BAND);
  const grid = new Map<string, { row: number; col: number }>();
  keys.forEach((key, i) => grid.set(key.id, { row: rows[i]!, col: cols[i]! }));
  return grid;
}

/**
 * Keyboard view: every key whose visual grid row and column both lie in the ranges spanned by the
 * anchor's and the cursor's cells. Falls back to the cursor alone when the anchor is unknown.
 */
export function keyboardRange(keys: BoardKeyDto[], anchorId: string, cursorId: string): string[] {
  const grid = gridCoordinates(keys);
  const cursor = grid.get(cursorId);
  const anchor = grid.get(anchorId);
  if (!cursor) return [];
  if (!anchor) return [cursorId];
  const rows = [Math.min(anchor.row, cursor.row), Math.max(anchor.row, cursor.row)];
  const cols = [Math.min(anchor.col, cursor.col), Math.max(anchor.col, cursor.col)];
  return keys
    .filter((key) => {
      const cell = grid.get(key.id)!;
      return cell.row >= rows[0]! && cell.row <= rows[1]! && cell.col >= cols[0]! && cell.col <= cols[1]!;
    })
    .map((key) => key.id);
}

/** Matrix view: every key whose row and column both lie in the ranges spanned by anchor and cursor. */
export function matrixRange(keys: BoardKeyDto[], anchorId: string, cursorId: string): string[] {
  const cursor = keys.find((key) => key.id === cursorId);
  const anchor = keys.find((key) => key.id === anchorId);
  if (!cursor) return [];
  if (!anchor) return [cursor.id];
  const rows = [Math.min(anchor.row, cursor.row), Math.max(anchor.row, cursor.row)];
  const cols = [Math.min(anchor.col, cursor.col), Math.max(anchor.col, cursor.col)];
  return keys
    .filter((key) => key.row >= rows[0]! && key.row <= rows[1]! && key.col >= cols[0]! && key.col <= cols[1]!)
    .map((key) => key.id);
}

/**
 * The candidate whose centre is nearest the point, in key units. Candidates default to every key.
 * Returns null when there is nothing to choose from.
 */
export function nearestKey(keys: BoardKeyDto[], p: Point, candidateIds: string[] = []): string | null {
  const pool = candidateIds.length ? keys.filter((key) => candidateIds.includes(key.id)) : keys;
  let best: { id: string; d: number } | null = null;
  for (const key of pool) {
    const c = centre(key);
    const d = Math.hypot(c.x - p.x, c.y - p.y);
    if (!best || d < best.d) best = { id: key.id, d };
  }
  return best?.id ?? null;
}
