import { afterEach, describe, expect, it, vi } from "vitest";
import {
  clampCenter,
  clampScale,
  fitScale,
  nextStep,
  readZoom,
  viewBoxFor,
  writeZoom,
  zoomAt,
} from "./zoom";

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

describe("zoom steps", () => {
  it("moves one preset up or down and stops at the limits", () => {
    expect(nextStep(1, 1)).toBe(1.1);
    expect(nextStep(1, -1)).toBe(0.9);
    expect(nextStep(4, 1)).toBe(4);
    expect(nextStep(0.25, -1)).toBe(0.25);
    expect(nextStep(1.04, -1)).toBe(1);
  });
  it("clamps to the range", () => {
    expect(clampScale(9)).toBe(4);
    expect(clampScale(0.1)).toBe(0.25);
    expect(clampScale(1.5)).toBe(1.5);
  });
});

describe("zoomAt", () => {
  it("keeps the point under the pointer fixed", () => {
    const c = { x: 500, y: 160 };
    const p = { x: 300, y: 90 };
    const next = zoomAt(c, p, 1, 2.5);
    expect(p.x + (next.x - p.x) * 2.5).toBeCloseTo(p.x + (c.x - p.x) * 1);
    expect(p.y + (next.y - p.y) * 2.5).toBeCloseTo(p.y + (c.y - p.y) * 1);
  });
});

describe("clampCenter", () => {
  const content = { x: 0, y: 0, w: 1000, h: 320 };
  it("centres an axis where the content fits", () => {
    const c = clampCenter({ x: 10, y: 10 }, content, { w: 1084, h: 631 }, 1);
    expect(c.x).toBe(500);
    expect(c.y).toBe(160);
  });
  it("limits pan on an axis where the content overflows", () => {
    // Scale 2, viewport 400x300 px: visible 200x150 units. Half-views 100 and 75 units,
    // margin 12 units (24 px). The centre may reach 100 - 12 = 88 on x and 320 - 75 + 12 = 257 on y.
    const c = clampCenter({ x: -5000, y: 5000 }, content, { w: 400, h: 300 }, 2);
    expect(c.x).toBeCloseTo(88);
    expect(c.y).toBeCloseTo(257);
  });
});

describe("readZoom and writeZoom", () => {
  function stubStorage(value: string | null) {
    const store = new Map<string, string>(value === null ? [] : [["vile.boardZoom", value]]);
    vi.stubGlobal("localStorage", {
      getItem: (k: string) => store.get(k) ?? null,
      setItem: (k: string, v: string) => void store.set(k, v),
    });
    return store;
  }
  afterEach(() => vi.unstubAllGlobals());
  it("falls back to fit for missing or invalid values", () => {
    stubStorage(null);
    expect(readZoom()).toBe("fit");
    stubStorage("abc");
    expect(readZoom()).toBe("fit");
    stubStorage("9");
    expect(readZoom()).toBe("fit");
    stubStorage("fit");
    expect(readZoom()).toBe("fit");
  });
  it("reads a stored scale and writes the setting back", () => {
    const store = stubStorage("1.5");
    expect(readZoom()).toBe(1.5);
    writeZoom(0.5);
    expect(store.get("vile.boardZoom")).toBe("0.5");
  });
});
