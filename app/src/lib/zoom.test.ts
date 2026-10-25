import { describe, expect, it } from "vitest";
import { fitScale, viewBoxFor } from "./zoom";

describe("fitScale", () => {
  const content = { x: 0, y: 0, w: 1000, h: 320 };
  const cap = { maxWidth: 1040, maxHeight: 455 };
  it("fills the viewport up to the caps", () => {
    expect(fitScale(content, { w: 1084, h: 631 }, cap)).toBeCloseTo(1.04);
  });
  it("is limited by a narrow viewport", () => {
    expect(fitScale(content, { w: 500, h: 631 }, cap)).toBeCloseTo(0.5);
  });
  it("is zero before the viewport has a size", () => {
    expect(fitScale(content, { w: 0, h: 0 }, cap)).toBe(0);
  });
});

describe("viewBoxFor", () => {
  it("centres the view on the given point at the given scale", () => {
    const box = viewBoxFor({ x: 500, y: 160 }, { w: 1084, h: 631 }, 1.04);
    expect(box.x + box.w / 2).toBeCloseTo(500);
    expect(box.y + box.h / 2).toBeCloseTo(160);
    expect(box.w).toBeCloseTo(1084 / 1.04);
    expect(box.h).toBeCloseTo(631 / 1.04);
  });
});
