export interface Size {
  w: number;
  h: number;
}

export interface Box {
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface FitCap {
  maxWidth: number;
  maxHeight: number;
}

/** CSS px per SVG unit that fits content in the viewport, never larger than the caps. */
export function fitScale(content: Box, size: Size, cap: FitCap): number {
  if (content.w <= 0 || content.h <= 0 || size.w <= 0 || size.h <= 0) return 0;
  return Math.min(
    size.w / content.w,
    size.h / content.h,
    cap.maxWidth / content.w,
    cap.maxHeight / content.h,
  );
}

/** viewBox centred on `center`, filling the viewport at `scale`. */
export function viewBoxFor(center: { x: number; y: number }, size: Size, scale: number): Box {
  const w = size.w / scale;
  const h = size.h / scale;
  return { x: center.x - w / 2, y: center.y - h / 2, w, h };
}

/** "fit" scales the board to the viewport; a number is an absolute scale (1 = 100%, one 1u key is 60px). */
export type ZoomSetting = "fit" | number;

export const MIN_ZOOM = 0.25;
export const MAX_ZOOM = 4;
export const ZOOM_STEPS = [0.25, 0.33, 0.5, 0.67, 0.75, 0.9, 1, 1.1, 1.25, 1.5, 1.75, 2, 2.5, 3, 4];

export function clampScale(s: number): number {
  return Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, s));
}

/** The next preset above (dir 1) or below (dir -1) `s`, clamped to the range. */
export function nextStep(s: number, dir: 1 | -1): number {
  if (dir > 0) return ZOOM_STEPS.find((v) => v > s + 0.001) ?? MAX_ZOOM;
  return [...ZOOM_STEPS].reverse().find((v) => v < s - 0.001) ?? MIN_ZOOM;
}

/** Keeps the SVG point `p` under the pointer while the scale changes from `s0` to `s1`. */
export function zoomAt(
  c: { x: number; y: number },
  p: { x: number; y: number },
  s0: number,
  s1: number,
): { x: number; y: number } {
  return { x: p.x + ((c.x - p.x) * s0) / s1, y: p.y + ((c.y - p.y) * s0) / s1 };
}

/**
 * Keeps the view centre `c` inside the content. An axis where the content fits the viewport is
 * centred on the content. Otherwise the content edge may move at most `margin` CSS px inside the viewport.
 */
export function clampCenter(
  c: { x: number; y: number },
  content: Box,
  size: Size,
  s: number,
  margin = 24,
): { x: number; y: number } {
  const axis = (v: number, start: number, len: number, view: number) => {
    if (len * s <= view) return start + len / 2;
    const half = view / s / 2;
    const m = margin / s;
    return Math.min(Math.max(v, start + half - m), start + len - half + m);
  };
  return {
    x: axis(c.x, content.x, content.w, size.w),
    y: axis(c.y, content.y, content.h, size.h),
  };
}

export function readZoom(): ZoomSetting {
  try {
    const raw = localStorage.getItem("vile.boardZoom");
    if (raw === null || raw === "fit") return "fit";
    const n = Number(raw);
    return Number.isFinite(n) && n >= MIN_ZOOM && n <= MAX_ZOOM ? n : "fit";
  } catch {
    return "fit";
  }
}

export function writeZoom(z: ZoomSetting): void {
  try {
    localStorage.setItem("vile.boardZoom", String(z));
  } catch {
    /* viewing still works */
  }
}
