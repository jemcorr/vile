// Pure geometry for bringing a key fully into view. Rectangles are client pixels, with y growing down.
export interface Rect {
  l: number;
  t: number;
  r: number;
  b: number;
}

/** --space-4: the least gap kept between a revealed key and the edge of the visible area or an occluder. */
export const REVEAL_MARGIN = 16;

function overlaps(a: Rect, b: Rect): boolean {
  return a.l < b.r && a.r > b.l && a.t < b.b && a.b > b.t;
}

function nonEmpty(a: Rect): boolean {
  return a.r > a.l && a.b > a.t;
}

/**
 * Rectangles of `area` that no occluder overlaps. Each occluder cuts every piece it touches into up to
 * four pieces (left, right, above, below). Any rectangle that avoids an occluder lies inside one of
 * those pieces, so the pieces cover every free placement.
 */
export function freeRects(area: Rect, occluders: Rect[]): Rect[] {
  let pieces = nonEmpty(area) ? [area] : [];
  for (const o of occluders) {
    const next: Rect[] = [];
    for (const p of pieces) {
      if (!overlaps(p, o)) {
        next.push(p);
        continue;
      }
      if (o.l > p.l) next.push({ l: p.l, t: p.t, r: Math.min(o.l, p.r), b: p.b });
      if (o.r < p.r) next.push({ l: Math.max(o.r, p.l), t: p.t, r: p.r, b: p.b });
      if (o.t > p.t) next.push({ l: p.l, t: p.t, r: p.r, b: Math.min(o.t, p.b) });
      if (o.b < p.b) next.push({ l: p.l, t: Math.max(o.b, p.t), r: p.r, b: p.b });
    }
    pieces = next;
  }
  return pieces.filter(nonEmpty);
}

/** Shrinks a rectangle by `margin` on every side, or null when nothing is left. */
export function insetRect(a: Rect, margin: number): Rect | null {
  const inner = { l: a.l + margin, t: a.t + margin, r: a.r - margin, b: a.b - margin };
  return nonEmpty(inner) ? inner : null;
}

/** The shift along one axis that puts [a0, a1] inside [b0, b1]; centred when [a0, a1] is wider than [b0, b1]. */
function axisShift(a0: number, a1: number, b0: number, b1: number): number {
  if (a1 - a0 > b1 - b0) return (b0 + b1) / 2 - (a0 + a1) / 2;
  if (a0 < b0) return b0 - a0;
  if (a1 > b1) return b1 - a1;
  return 0;
}

/**
 * The smallest move (dx, dy, client px) that puts `key` fully inside the free part of `area`, inset by
 * `margin` from the area's edges and from each occluder. A key already inside a free part gets (0, 0).
 * When no free part can hold the key, it is centred on the largest free part, along any axis where it
 * does not fit.
 */
export function revealShift(
  key: Rect,
  area: Rect,
  occluders: Rect[],
  margin = REVEAL_MARGIN,
): { dx: number; dy: number } {
  const w = key.r - key.l;
  const h = key.b - key.t;
  const inners = freeRects(area, occluders)
    .map((piece) => insetRect(piece, margin))
    .filter((inner): inner is Rect => inner !== null);
  let best: { dx: number; dy: number; cost: number } | null = null;
  for (const inner of inners) {
    if (inner.r - inner.l < w || inner.b - inner.t < h) continue;
    const dx = axisShift(key.l, key.r, inner.l, inner.r);
    const dy = axisShift(key.t, key.b, inner.t, inner.b);
    const cost = Math.abs(dx) + Math.abs(dy);
    if (!best || cost < best.cost) best = { dx, dy, cost };
  }
  if (best) return { dx: best.dx, dy: best.dy };
  // No free part holds the whole key: use the largest one and centre the key along the short axes.
  let largest: Rect | null = null;
  for (const inner of inners) {
    if (!largest || (inner.r - inner.l) * (inner.b - inner.t) > (largest.r - largest.l) * (largest.b - largest.t))
      largest = inner;
  }
  if (!largest) return { dx: 0, dy: 0 };
  return {
    dx: axisShift(key.l, key.r, largest.l, largest.r),
    dy: axisShift(key.t, key.b, largest.t, largest.b),
  };
}
