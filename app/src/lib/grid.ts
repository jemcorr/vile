import type { BoardKeyDto } from "./types";
import { rotate } from "./interactions";

export type Dir = "left" | "right" | "up" | "down";

const r2 = (v: number): number => Math.round(v * 100) / 100;

/** The key reached from `fromId` in direction `dir`, or `fromId` when there is none. */
export function neighbor(keys: BoardKeyDto[], fromId: string, dir: Dir): string {
  const start = keys.find((k) => k.id === fromId);
  if (!start) return fromId;
  const { x: cx, y: cy } = rotate(start, { x: start.x + start.w / 2, y: start.y + start.h / 2 });

  const candidates: { key: BoardKeyDto; index: number; p: number; s: number }[] = [];
  keys.forEach((key, index) => {
    if (key.id === fromId) return;
    const center = rotate(key, { x: key.x + key.w / 2, y: key.y + key.h / 2 });
    const dx = center.x - cx;
    const dy = center.y - cy;
    let p: number;
    let s: number;
    switch (dir) {
      case "right":
        p = dx;
        s = dy;
        break;
      case "left":
        p = -dx;
        s = dy;
        break;
      case "down":
        p = dy;
        s = dx;
        break;
      case "up":
        p = -dy;
        s = dx;
        break;
    }
    p = r2(p);
    s = r2(Math.abs(s));
    if (p > 0.5) candidates.push({ key, index, p, s });
  });
  if (candidates.length === 0) return fromId;

  const sameRow = (c: { key: BoardKeyDto }): number => (c.key.row === start.row ? 0 : 1);
  const aligned = candidates.filter((c) => c.s <= 0.75);
  if (aligned.length > 0) {
    const [best] = aligned.sort(
      (a, b) => a.p - b.p || a.s - b.s || sameRow(a) - sameRow(b) || a.index - b.index,
    );
    if (best) return best.key.id;
  }
  const scored = candidates.map((c) => ({ ...c, d: r2(c.p * c.p + c.s * c.s) }));
  const [best] = scored.sort((a, b) => a.d - b.d || sameRow(a) - sameRow(b) || a.index - b.index);
  return best ? best.key.id : fromId;
}
