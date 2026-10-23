import type { Dir } from "./grid";
import type { Mode } from "./modes";

export interface KeyLike {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
}

export type Action =
  | { kind: "move"; dir: Dir }
  | { kind: "layer"; layer: number }
  | { kind: "toggleSelect" }
  | { kind: "visual" }
  | { kind: "escape" }
  | { kind: "insert" }
  | { kind: "clear" }
  | { kind: "transparent" }
  | { kind: "copy" }
  | { kind: "paste" }
  | { kind: "swap" }
  | { kind: "undo" }
  | { kind: "redo" }
  | { kind: "copyLayer" }
  | { kind: "pasteLayer" }
  | { kind: "clearLayer" }
  | { kind: "transparentLayer" }
  | { kind: "palette" }
  | { kind: "save" }
  | { kind: "open" }
  | { kind: "toggleDiffBase" }
  | { kind: "nextPanel" }
  | { kind: "toggleCombos" };

const PLAIN: Record<string, Action> = {
  h: { kind: "move", dir: "left" },
  ArrowLeft: { kind: "move", dir: "left" },
  j: { kind: "move", dir: "down" },
  ArrowDown: { kind: "move", dir: "down" },
  k: { kind: "move", dir: "up" },
  ArrowUp: { kind: "move", dir: "up" },
  l: { kind: "move", dir: "right" },
  ArrowRight: { kind: "move", dir: "right" },
  " ": { kind: "toggleSelect" },
  v: { kind: "visual" },
  Escape: { kind: "escape" },
  i: { kind: "insert" },
  Enter: { kind: "insert" },
  x: { kind: "clear" },
  t: { kind: "transparent" },
  y: { kind: "copy" },
  p: { kind: "paste" },
  s: { kind: "swap" },
  u: { kind: "undo" },
  Y: { kind: "copyLayer" },
  P: { kind: "pasteLayer" },
  X: { kind: "clearLayer" },
  T: { kind: "transparentLayer" },
  ":": { kind: "palette" },
  d: { kind: "toggleDiffBase" },
  Tab: { kind: "nextPanel" },
  o: { kind: "toggleCombos" },
};

/** The action for a key press in the given mode, or null when the key does nothing. */
export function actionFor(mode: Mode, e: KeyLike): Action | null {
  if (mode !== "normal" && mode !== "visual") return null;
  if (e.metaKey && !e.ctrlKey && !e.altKey) {
    switch (e.key.toLowerCase()) {
      case "s":
        return { kind: "save" };
      case "o":
        return { kind: "open" };
      case "k":
        return { kind: "palette" };
      case "z":
        return e.shiftKey ? { kind: "redo" } : { kind: "undo" };
      default:
        return null;
    }
  }
  if (e.ctrlKey && !e.metaKey && !e.altKey) {
    return e.key === "r" ? { kind: "redo" } : null;
  }
  if (e.ctrlKey || e.metaKey || e.altKey) return null;
  if (/^[0-9]$/.test(e.key)) return { kind: "layer", layer: Number(e.key) };
  return PLAIN[e.key] ?? null;
}
