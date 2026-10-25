import { describe, expect, it } from "vitest";
import { hasBit, modNames, MOD_BITS, MOD_LABELS, toggleBit } from "./bits";

describe("bits", () => {
  it("tests and toggles bits", () => {
    expect(hasBit(0b101, 0)).toBe(true);
    expect(hasBit(0b101, 1)).toBe(false);
    expect(toggleBit(0b101, 1)).toBe(0b111);
    expect(toggleBit(0b111, 0)).toBe(0b110);
    expect(toggleBit(0xffff, 15)).toBe(0x7fff);
  });

  it("names modifier masks in bit order", () => {
    expect(MOD_BITS.map((m) => m.name)).toEqual(["LCTL", "LSFT", "LALT", "LGUI", "RCTL", "RSFT", "RALT", "RGUI"]);
    expect(modNames(0)).toBe("none");
    expect(modNames(0x02)).toBe("LSFT");
    expect(modNames(0x22)).toBe("LSFT+RSFT");
  });

  it("labels every modifier with a short name", () => {
    expect(MOD_LABELS.LCTL).toBe("L Ctrl");
    expect(MOD_BITS.every((m) => MOD_LABELS[m.name] !== undefined)).toBe(true);
  });
});
