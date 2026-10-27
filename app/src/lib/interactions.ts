import type { BoardKeyDto } from "./types";
export interface Point {
  x: number;
  y: number;
}
export interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}
export function unrotate(key: BoardKeyDto, p: Point): Point {
  const a = (-(key.r ?? 0) * Math.PI) / 180;
  const x = p.x - (key.rx ?? 0),
    y = p.y - (key.ry ?? 0);
  return {
    x: x * Math.cos(a) - y * Math.sin(a) + (key.rx ?? 0),
    y: x * Math.sin(a) + y * Math.cos(a) + (key.ry ?? 0),
  };
}
export function hitKey(keys: BoardKeyDto[], p: Point): string | null {
  return (
    [...keys].reverse().find((key) => {
      const q = unrotate(key, p);
      return (
        (q.x >= key.x && q.x <= key.x + key.w && q.y >= key.y && q.y <= key.y + key.h) ||
        (!!key.secondary &&
          q.x >= key.x + key.secondary.x &&
          q.x <= key.x + key.secondary.x + key.secondary.w &&
          q.y >= key.y + key.secondary.y &&
          q.y <= key.y + key.secondary.y + key.secondary.h)
      );
    })?.id ?? null
  );
}
export function rotate(key: BoardKeyDto, p: Point): Point {
  return unrotate({ ...key, r: -(key.r ?? 0) }, p);
}
export function marqueeKeys(keys: BoardKeyDto[], a: Point, b: Point): string[] {
  const left = Math.min(a.x, b.x),
    right = Math.max(a.x, b.x),
    top = Math.min(a.y, b.y),
    bottom = Math.max(a.y, b.y);
  const box: Point[] = [
    { x: left, y: top },
    { x: right, y: top },
    { x: right, y: bottom },
    { x: left, y: bottom },
  ];
  return keys
    .filter((key) =>
      keyRects(key).some((rect) => {
        const corners = (
          [
            [0, 0],
            [rect.w, 0],
            [rect.w, rect.h],
            [0, rect.h],
          ] as [number, number][]
        ).map(([dx, dy]) => rotate(key, { x: key.x + rect.x + dx, y: key.y + rect.y + dy }));
        const a = ((key.r ?? 0) * Math.PI) / 180;
        // Separating axes: the marquee's own and the key's rotated ones.
        const axes: Point[] = [
          { x: 1, y: 0 },
          { x: 0, y: 1 },
          { x: Math.cos(a), y: Math.sin(a) },
          { x: -Math.sin(a), y: Math.cos(a) },
        ];
        return axes.every((axis) => {
          const project = (points: Point[]) => points.map((p) => p.x * axis.x + p.y * axis.y);
          const k = project(corners),
            m = project(box);
          return Math.min(...m) < Math.max(...k) && Math.max(...m) > Math.min(...k);
        });
      }),
    )
    .map((key) => key.id);
}
function keyRects(key: BoardKeyDto): Rect[] {
  return [{ x: 0, y: 0, w: key.w, h: key.h }, ...(key.secondary ? [key.secondary] : [])];
}
export function bounds(keys: BoardKeyDto[]): Rect {
  const points = keys.flatMap((key) => {
    const rectangles = [
      { x: 0, y: 0, w: key.w, h: key.h },
      ...(key.secondary ? [key.secondary] : []),
    ];
    return rectangles.flatMap((rect) =>
      (
        [
          [0, 0],
          [rect.w, 0],
          [0, rect.h],
          [rect.w, rect.h],
        ] as [number, number][]
      ).map(([dx, dy]) => rotate(key, { x: key.x + rect.x + dx, y: key.y + rect.y + dy })),
    );
  });
  const x = Math.min(0, ...points.map((p) => p.x)),
    y = Math.min(0, ...points.map((p) => p.y));
  return {
    x,
    y,
    w: Math.max(1, ...points.map((p) => p.x)) - x,
    h: Math.max(1, ...points.map((p) => p.y)) - y,
  };
}
