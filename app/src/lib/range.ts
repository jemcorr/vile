// Spreadsheet-style range selection. The anchor is the key a range grows from; the cursor is the
// key the arrows move. Pure functions only; the App decides which view's rule to use.
import type { BoardKeyDto } from "./types";
import { rotate, type Point } from "./interactions";

/**
 * Key units. A centre within half a key of the box counts as inside. On a uniform grid the next
 * centre out is a full key away, so nothing changes there; on a staggered board (corne columns sit
 * about 0.3u apart) the keys of a neighbouring row or column are still picked up.
 */
const TOLERANCE = 0.5;

function centre(key: BoardKeyDto): Point {
  return rotate(key, { x: key.x + key.w / 2, y: key.y + key.h / 2 });
}

/**
 * Keyboard view: every key whose (rotated) centre lies in the axis-aligned box spanned by the
 * anchor's and the cursor's centres, grown by TOLERANCE. Falls back to the cursor alone when the
 * anchor is unknown.
 */
export function keyboardRange(keys: BoardKeyDto[], anchorId: string, cursorId: string): string[] {
  const cursor = keys.find((key) => key.id === cursorId);
  const anchor = keys.find((key) => key.id === anchorId);
  if (!cursor) return [];
  if (!anchor) return [cursor.id];
  const a = centre(anchor);
  const c = centre(cursor);
  const left = Math.min(a.x, c.x) - TOLERANCE;
  const right = Math.max(a.x, c.x) + TOLERANCE;
  const top = Math.min(a.y, c.y) - TOLERANCE;
  const bottom = Math.max(a.y, c.y) + TOLERANCE;
  return keys
    .filter((key) => {
      const p = centre(key);
      return p.x >= left && p.x <= right && p.y >= top && p.y <= bottom;
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
