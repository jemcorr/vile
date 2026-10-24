import { describe, expect, it } from "vitest";
import { actionFor as resolveAction } from "./keymap";
import type { KeyLike } from "./keymap";

function actionFor(mode: Parameters<typeof resolveAction>[0], event: KeyLike) {
  return resolveAction(mode, event, true);
}

function key(k: string, mods: Partial<KeyLike> = {}): KeyLike {
  return { key: k, ctrlKey: false, metaKey: false, shiftKey: false, altKey: false, ...mods };
}

describe("actionFor in normal mode", () => {
  it("moves with hjkl and arrows", () => {
    expect(actionFor("normal", key("h"))).toEqual({ kind: "move", dir: "left" });
    expect(actionFor("normal", key("j"))).toEqual({ kind: "move", dir: "down" });
    expect(actionFor("normal", key("k"))).toEqual({ kind: "move", dir: "up" });
    expect(actionFor("normal", key("l"))).toEqual({ kind: "move", dir: "right" });
    expect(actionFor("normal", key("ArrowLeft"))).toEqual({ kind: "move", dir: "left" });
    expect(actionFor("normal", key("ArrowDown"))).toEqual({ kind: "move", dir: "down" });
  });

  it("switches layers with digits", () => {
    expect(actionFor("normal", key("0"))).toEqual({ kind: "layer", layer: 0 });
    expect(actionFor("normal", key("9"))).toEqual({ kind: "layer", layer: 9 });
  });

  it("maps editing keys", () => {
    expect(actionFor("normal", key(" "))).toEqual({ kind: "toggleSelect" });
    expect(actionFor("normal", key("v"))).toEqual({ kind: "visual" });
    expect(actionFor("normal", key("Escape"))).toEqual({ kind: "escape" });
    expect(actionFor("normal", key("i"))).toEqual({ kind: "insert" });
    expect(actionFor("normal", key("Enter"))).toEqual({ kind: "insert" });
    expect(actionFor("normal", key("x"))).toEqual({ kind: "clear" });
    expect(actionFor("normal", key("t"))).toEqual({ kind: "transparent" });
    expect(actionFor("normal", key("y"))).toEqual({ kind: "copy" });
    expect(actionFor("normal", key("p"))).toEqual({ kind: "paste" });
    expect(actionFor("normal", key("s"))).toEqual({ kind: "swap" });
    expect(actionFor("normal", key("u"))).toEqual({ kind: "undo" });
    expect(actionFor("normal", key("r", { ctrlKey: true }))).toEqual({ kind: "redo" });
  });

  it("maps layer keys with shift", () => {
    expect(actionFor("normal", key("Y", { shiftKey: true }))).toEqual({ kind: "copyLayer" });
    expect(actionFor("normal", key("P", { shiftKey: true }))).toEqual({ kind: "pasteLayer" });
    expect(actionFor("normal", key("X", { shiftKey: true }))).toEqual({ kind: "clearLayer" });
    expect(actionFor("normal", key("T", { shiftKey: true }))).toEqual({ kind: "transparentLayer" });
  });

  it("maps command keys", () => {
    expect(actionFor("normal", key(":", { shiftKey: true }))).toEqual({ kind: "palette" });
    expect(actionFor("normal", key("k", { metaKey: true }))).toEqual({ kind: "palette" });
    expect(actionFor("normal", key("s", { metaKey: true }))).toEqual({ kind: "save" });
    expect(actionFor("normal", key("o", { metaKey: true }))).toEqual({ kind: "open" });
    expect(actionFor("normal", key("z", { metaKey: true }))).toEqual({ kind: "undo" });
    expect(actionFor("normal", key("z", { metaKey: true, shiftKey: true }))).toEqual({
      kind: "redo",
    });
    expect(actionFor("normal", key("Z", { metaKey: true, shiftKey: true }))).toEqual({
      kind: "redo",
    });
    expect(actionFor("normal", key("d"))).toEqual({ kind: "toggleDiffBase" });
    expect(actionFor("normal", key("Tab"))).toEqual({ kind: "nextPanel" });
    expect(actionFor("normal", key("o"))).toEqual({ kind: "toggleCombos" });
  });

  it("ignores unknown keys and modified letters", () => {
    expect(actionFor("normal", key("q"))).toBeNull();
    expect(actionFor("normal", key("h", { metaKey: true }))).toBeNull();
    expect(actionFor("normal", key("h", { altKey: true }))).toBeNull();
    expect(actionFor("normal", key("5", { ctrlKey: true }))).toBeNull();
  });
});

describe("actionFor in other modes", () => {
  it("visual mode uses the same bindings", () => {
    expect(actionFor("visual", key("l"))).toEqual({ kind: "move", dir: "right" });
    expect(actionFor("visual", key("Escape"))).toEqual({ kind: "escape" });
  });

  it("text modes leave keys to their inputs", () => {
    expect(actionFor("insert", key("h"))).toBeNull();
    expect(actionFor("palette", key("Escape"))).toBeNull();
    expect(actionFor("editor", key("x"))).toBeNull();
  });
});

describe("default layout-independent bindings", () => {
  it("requires opt-in for letters and permits remapping", () => {
    for (const letter of ["h", "j", "k", "l", "x", "t", "y", "p", "s", "u", "d", "o", "v", "i"])
      expect(resolveAction("normal", key(letter))).toBeNull();
    expect(resolveAction("normal", key("a"), true, { a: "h" })).toEqual({
      kind: "move",
      dir: "left",
    });
  });
  it("extends selections and supports conventional clipboard and Delete", () => {
    expect(resolveAction("normal", key("ArrowLeft", { shiftKey: true }))).toEqual({
      kind: "move",
      dir: "left",
      extend: true,
    });
    expect(resolveAction("normal", key("c", { metaKey: true }))).toEqual({ kind: "copy" });
    expect(resolveAction("normal", key("v", { metaKey: true }))).toEqual({ kind: "paste" });
    expect(resolveAction("normal", key("x", { metaKey: true }))).toEqual({ kind: "cut" });
    expect(resolveAction("normal", key("Delete"))).toEqual({ kind: "clear" });
  });
});
