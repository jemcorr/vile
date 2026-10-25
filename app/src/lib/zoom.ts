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
