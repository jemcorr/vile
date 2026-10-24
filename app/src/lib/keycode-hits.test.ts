import { describe, expect, it } from "vitest";
import { hitLabel } from "./keycode-hits";

describe("hitLabel", () => {
  it("omits a label that only repeats the code name without its prefix", () => {
    expect(hitLabel("KC_UNDO", "UNDO")).toBe("");
    expect(hitLabel("MAGIC_CAPSLOCK_TO_CONTROL", "CAPSLOCK_TO_CONTROL")).toBe("");
    expect(hitLabel("KC_A", "A")).toBe("");
  });

  it("keeps labels that add meaning", () => {
    expect(hitLabel("KC_NUM_LOCK", "NumLk")).toBe("NumLk");
    expect(hitLabel("KC_KP_MINUS", "Bri-")).toBe("Bri-");
    expect(hitLabel("KC_HASH", "#")).toBe("#");
  });

  it("collapses line breaks and treats an empty label as absent", () => {
    expect(hitLabel("KC_LSFT", "Shift\nHold")).toBe("Shift Hold");
    expect(hitLabel("KC_NO", "")).toBe("");
    expect(hitLabel("KC_NO", "  \n ")).toBe("");
  });
});
