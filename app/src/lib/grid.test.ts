import { describe, expect, it } from "vitest";
import board from "../../../crates/vile-core/boards/corne_v4_1.json";
import { neighbor } from "./grid";
import type { Dir } from "./grid";
import type { BoardKeyDto } from "./types";

const keys: BoardKeyDto[] = board.keys;

describe("neighbor", () => {
  const cases: Array<[string, Dir, string]> = [
    ["home-left-index", "right", "home-left-inner"],
    ["home-left-inner", "right", "home-left-extra"],
    ["home-left-extra", "right", "home-right-extra"],
    ["home-right-extra", "left", "home-left-extra"],
    ["home-left-middle", "left", "home-left-ring"],
    ["home-right-inner", "right", "home-right-index"],
    ["home-right-inner", "left", "home-right-extra"],
    ["top-left-inner", "right", "top-left-extra"],
    ["top-left-outer", "down", "home-left-outer"],
    ["top-left-middle", "down", "home-left-middle"],
    ["top-left-extra", "down", "home-left-extra"],
    ["home-left-extra", "down", "thumb-left-inner"],
    ["bottom-left-index", "down", "thumb-left-outer"],
    ["bottom-left-inner", "down", "thumb-left-mid"],
    ["bottom-left-inner", "right", "home-left-extra"],
    ["thumb-left-outer", "up", "bottom-left-index"],
    ["thumb-left-inner", "up", "home-left-extra"],
    ["thumb-right-outer", "up", "bottom-right-index"],
    ["thumb-left-mid", "right", "thumb-left-inner"],
    ["thumb-left-inner", "right", "thumb-right-inner"],
    ["thumb-right-inner", "left", "thumb-left-inner"],
  ];
  for (const [from, dir, to] of cases) {
    it(`${from} ${dir} -> ${to}`, () => {
      expect(neighbor(keys, from, dir)).toBe(to);
    });
  }

  it("stays put at the edges", () => {
    expect(neighbor(keys, "top-left-outer", "up")).toBe("top-left-outer");
    expect(neighbor(keys, "top-left-outer", "left")).toBe("top-left-outer");
    expect(neighbor(keys, "top-right-outer", "right")).toBe("top-right-outer");
    expect(neighbor(keys, "thumb-left-outer", "down")).toBe("thumb-left-outer");
  });

  it("returns the input for an unknown id", () => {
    expect(neighbor(keys, "nope", "left")).toBe("nope");
  });
});
