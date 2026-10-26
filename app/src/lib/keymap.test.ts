import { describe, expect, it } from "vitest";
import { actionFor as resolveAction } from "./keymap";
import type { KeyLike } from "./keymap";

function actionFor(mode: Parameters<typeof resolveAction>[0], event: KeyLike) {
  return resolveAction(mode, event, "MacIntel");
}

function key(k: string, mods: Partial<KeyLike> = {}): KeyLike {
  return { key: k, ctrlKey: false, metaKey: false, shiftKey: false, altKey: false, ...mods };
}

describe("actionFor in normal mode", () => {
  it("moves with arrows, extending with Shift", () => {
    expect(actionFor("normal", key("ArrowLeft"))).toEqual({ kind: "move", dir: "left" });
    expect(actionFor("normal", key("ArrowDown"))).toEqual({ kind: "move", dir: "down" });
    expect(actionFor("normal", key("ArrowRight", { shiftKey: true }))).toEqual({
      kind: "move",
      dir: "right",
      extend: true,
    });
  });

  it("switches layers with digits", () => {
    expect(actionFor("normal", key("0"))).toEqual({ kind: "layer", layer: 0 });
    expect(actionFor("normal", key("9"))).toEqual({ kind: "layer", layer: 9 });
  });

  it("maps Enter, Escape, Delete and Backspace", () => {
    expect(actionFor("normal", key("Enter"))).toEqual({ kind: "insert" });
    expect(actionFor("normal", key("Escape"))).toEqual({ kind: "escape" });
    expect(actionFor("normal", key("Delete"))).toEqual({ kind: "clear" });
    expect(actionFor("normal", key("Backspace"))).toEqual({ kind: "clear" });
  });

  it("maps command keys", () => {
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
  });

  it("ignores unknown keys and modified letters", () => {
    expect(actionFor("normal", key("q"))).toBeNull();
    expect(actionFor("normal", key("h", { metaKey: true }))).toBeNull();
    expect(actionFor("normal", key("h", { altKey: true }))).toBeNull();
    expect(actionFor("normal", key("5", { ctrlKey: true }))).toBeNull();
  });
});

describe("actionFor in other modes", () => {
  it("text modes leave keys to their inputs", () => {
    expect(actionFor("insert", key("h"))).toBeNull();
    expect(actionFor("insert", key("Enter"))).toBeNull();
    expect(actionFor("palette", key("Escape"))).toBeNull();
    expect(actionFor("editor", key("x"))).toBeNull();
  });
});

describe("no letter-key bindings", () => {
  it("binds no bare letter, symbol or Tab key", () => {
    for (const letter of "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ")
      expect(actionFor("normal", key(letter))).toBeNull();
    expect(actionFor("normal", key(":", { shiftKey: true }))).toBeNull();
    expect(actionFor("normal", key(" "))).toBeNull();
    expect(actionFor("normal", key("Tab"))).toBeNull();
  });

  it("binds no Ctrl+R or Cmd+R redo, so redo stays on Z", () => {
    expect(resolveAction("normal", key("r", { metaKey: true }), "MacIntel")).toBeNull();
    expect(resolveAction("normal", key("r", { ctrlKey: true }), "Win32")).toBeNull();
  });
});

describe("default layout-independent bindings", () => {
  it("extends selections and supports conventional clipboard and Delete", () => {
    expect(resolveAction("normal", key("ArrowLeft", { shiftKey: true }), "MacIntel")).toEqual({
      kind: "move",
      dir: "left",
      extend: true,
    });
    expect(resolveAction("normal", key("c", { metaKey: true }), "MacIntel")).toEqual({
      kind: "copy",
    });
    expect(resolveAction("normal", key("v", { metaKey: true }), "MacIntel")).toEqual({
      kind: "paste",
    });
    expect(resolveAction("normal", key("x", { metaKey: true }), "MacIntel")).toEqual({
      kind: "cut",
    });
    expect(resolveAction("normal", key("Delete"), "MacIntel")).toEqual({ kind: "clear" });
  });

  it("uses Ctrl shortcuts on Windows and Linux, while leaving AltGr input alone", () => {
    for (const platform of ["Win32", "Linux x86_64"]) {
      expect(resolveAction("normal", key("s", { ctrlKey: true }), platform)).toEqual({
        kind: "save",
      });
      expect(
        resolveAction("normal", key("z", { ctrlKey: true, shiftKey: true }), platform),
      ).toEqual({ kind: "redo" });
      expect(resolveAction("normal", key("s", { metaKey: true }), platform)).toBeNull();
    }
    expect(
      resolveAction("normal", key("s", { ctrlKey: true, altKey: true }), "Win32"),
    ).toBeNull();
    expect(
      resolveAction("normal", key("s", { ctrlKey: true, altKey: true }), "Linux x86_64"),
    ).toBeNull();
  });
});

describe("zoom shortcuts", () => {
  it("maps Cmd (macOS) with =, -, 0 to zoom actions, and Shift+= to zoom in", () => {
    expect(actionFor("normal", key("=", { metaKey: true }))).toEqual({ kind: "zoomIn" });
    expect(actionFor("normal", key("+", { metaKey: true, shiftKey: true }))).toEqual({ kind: "zoomIn" });
    expect(actionFor("normal", key("-", { metaKey: true }))).toEqual({ kind: "zoomOut" });
    expect(actionFor("normal", key("0", { metaKey: true }))).toEqual({ kind: "zoomFit" });
  });
  it("uses Ctrl on Windows and Linux", () => {
    expect(resolveAction("normal", key("=", { ctrlKey: true }), "Win32")).toEqual({
      kind: "zoomIn",
    });
    expect(resolveAction("normal", key("0", { ctrlKey: true }), "Linux x86_64")).toEqual({
      kind: "zoomFit",
    });
  });
  it("keeps plain digits as layer switches", () => {
    expect(actionFor("normal", key("0"))).toEqual({ kind: "layer", layer: 0 });
  });
});
