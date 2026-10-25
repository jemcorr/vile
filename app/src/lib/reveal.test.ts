import { describe, expect, it } from "vitest";
import { freeRects, insetRect, revealShift, type Rect } from "./reveal";

const area: Rect = { l: 0, t: 0, r: 1000, b: 600 };
const key = (l: number, t: number, w: number, h: number): Rect => ({ l, t, r: l + w, b: t + h });

describe("revealShift on an open area", () => {
  it("does not move a key already inside the margin", () => {
    expect(revealShift(key(100, 100, 60, 60), area, [])).toEqual({ dx: 0, dy: 0 });
  });
  it("moves a key off the left edge so its left edge sits one margin inside", () => {
    expect(revealShift(key(-40, 200, 60, 60), area, [])).toEqual({ dx: 56, dy: 0 });
  });
  it("moves a key off the right edge", () => {
    expect(revealShift(key(960, 200, 60, 60), area, [])).toEqual({ dx: -36, dy: 0 });
  });
  it("moves a key off the top edge", () => {
    expect(revealShift(key(300, -30, 60, 60), area, [])).toEqual({ dx: 0, dy: 46 });
  });
  it("moves a key off the bottom edge", () => {
    expect(revealShift(key(300, 570, 60, 60), area, [])).toEqual({ dx: 0, dy: -46 });
  });
  it("counts the selection ring margin: a key touching the edge still moves", () => {
    expect(revealShift(key(0, 200, 60, 60), area, [])).toEqual({ dx: 16, dy: 0 });
  });
});

describe("revealShift with occluders", () => {
  // A floating card covers x 700 to 1000 for the full height.
  const card: Rect = { l: 700, t: 0, r: 1000, b: 600 };
  it("moves a key left of an occluder on the right, keeping the margin", () => {
    expect(revealShift(key(650, 200, 100, 60), area, [card])).toEqual({ dx: -66, dy: 0 });
  });
  it("leaves a key alone when it already clears the occluder by the margin", () => {
    expect(revealShift(key(560, 200, 100, 60), area, [card])).toEqual({ dx: 0, dy: 0 });
  });
  it("moves a key up above a bottom bar", () => {
    // A bar across the bottom: 500 to 1000 wide, y 540 to 600. A key at y 500 to 560 overlaps it.
    const bar: Rect = { l: 0, t: 540, r: 1000, b: 600 };
    expect(revealShift(key(300, 500, 60, 60), area, [bar])).toEqual({ dx: 0, dy: -36 });
  });
  it("moves a key to the free part that is nearest", () => {
    // A block covers x 700 to 1000, y 300 to 600. The top right part (y up to 300) is 126 px away, the left part 136 px.
    const block: Rect = { l: 700, t: 300, r: 1000, b: 600 };
    expect(revealShift(key(720, 350, 100, 60), area, [block])).toEqual({ dx: 0, dy: -126 });
  });
  it("keeps a key that already sits in a free part", () => {
    // Two occluders leave one free part, above the lower bar and right of the left one. The key is in it.
    const left: Rect = { l: 0, t: 0, r: 200, b: 600 };
    const low: Rect = { l: 200, t: 300, r: 1000, b: 600 };
    expect(revealShift(key(600, 100, 60, 60), area, [left, low])).toEqual({ dx: 0, dy: 0 });
  });
});

describe("revealShift for a key larger than the free area", () => {
  it("centres a key wider than the view and leaves the other axis alone", () => {
    // A 1200 wide key on a 968 wide free part: its centre (600) moves to the part's centre (500).
    expect(revealShift(key(0, 200, 1200, 60), area, [])).toEqual({ dx: -100, dy: 0 });
  });
  it("centres a key taller than the view on the vertical axis", () => {
    expect(revealShift(key(300, -200, 60, 1200), area, [])).toEqual({ dx: 0, dy: -100 });
  });
  it("does not pick a sliver between occluders that cannot hold the key", () => {
    // The gap between the two occluders is 50 px wide, too narrow for a 100 px key: the key moves into the
    // wider free part on the left instead of centring on the sliver.
    const card: Rect = { l: 700, t: 0, r: 1000, b: 600 };
    const wall: Rect = { l: 600, t: 0, r: 650, b: 600 };
    const shift = revealShift(key(650, 300, 100, 60), area, [card, wall]);
    expect(shift).toEqual({ dx: -166, dy: 0 });
  });
});

describe("freeRects and insetRect", () => {
  it("returns the whole area when nothing occludes it", () => {
    expect(freeRects(area, [])).toEqual([area]);
  });
  it("keeps every free rectangle clear of the occluder", () => {
    const o: Rect = { l: 300, t: 100, r: 500, b: 200 };
    for (const piece of freeRects(area, [o])) {
      const clear = piece.r <= o.l || piece.l >= o.r || piece.b <= o.t || piece.t >= o.b;
      expect(clear).toBe(true);
    }
  });
  it("insets a rectangle and returns null when nothing is left", () => {
    expect(insetRect({ l: 0, t: 0, r: 100, b: 100 }, 16)).toEqual({ l: 16, t: 16, r: 84, b: 84 });
    expect(insetRect({ l: 0, t: 0, r: 20, b: 100 }, 16)).toBeNull();
  });
});
