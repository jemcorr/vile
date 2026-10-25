import type { Dir } from "./grid";
import type { Mode } from "./modes";

export interface KeyLike {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
}

export function isMacPlatform(platform: string): boolean {
  return /Mac|iPhone|iPad|iPod/i.test(platform);
}

export function primaryModifierDown(e: KeyLike, platform: string): boolean {
  return isMacPlatform(platform)
    ? e.metaKey && !e.ctrlKey && !e.altKey
    : e.ctrlKey && !e.metaKey && !e.altKey;
}

export function primaryModifierLabel(platform: string): string {
  return isMacPlatform(platform) ? "⌘" : "Ctrl+";
}

export type Action =
  | { kind: "move"; dir: Dir; extend?: boolean }
  | { kind: "layer"; layer: number }
  | { kind: "toggleSelect" }
  | { kind: "visual" }
  | { kind: "escape" }
  | { kind: "insert" }
  | { kind: "clear" }
  | { kind: "transparent" }
  | { kind: "copy" }
  | { kind: "cut" }
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
  | { kind: "toggleCombos" }
  | { kind: "zoomIn" }
  | { kind: "zoomOut" }
  | { kind: "zoomFit" };

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
export function actionFor(
  mode: Mode,
  e: KeyLike,
  vim = false,
  remaps: Record<string, string> = {},
  platform = typeof navigator === "undefined" ? "" : navigator.platform,
): Action | null {
  if (mode !== "normal" && mode !== "visual") return null;
  if (primaryModifierDown(e, platform)) {
    switch (e.key.toLowerCase()) {
      case "c":
        return { kind: "copy" };
      case "v":
        return { kind: "paste" };
      case "x":
        return { kind: "cut" };
      case "s":
        return { kind: "save" };
      case "o":
        return { kind: "open" };
      case "k":
        return { kind: "palette" };
      case "z":
        return e.shiftKey ? { kind: "redo" } : { kind: "undo" };
      case "r":
        return vim ? { kind: "redo" } : null;
      case "=":
      case "+":
        return { kind: "zoomIn" };
      case "-":
      case "_":
        return { kind: "zoomOut" };
      case "0":
        return { kind: "zoomFit" };
      default:
        return null;
    }
  }
  if (
    (isMacPlatform(platform) && e.ctrlKey && !e.metaKey && !e.altKey) ||
    (!isMacPlatform(platform) && e.metaKey && !e.ctrlKey && !e.altKey)
  ) {
    return vim && e.key === "r" ? { kind: "redo" } : null;
  }
  if (e.ctrlKey || e.metaKey || e.altKey) return null;
  if (/^[0-9]$/.test(e.key)) return { kind: "layer", layer: Number(e.key) };
  const directions: Record<string, Dir> = {
    ArrowLeft: "left",
    ArrowRight: "right",
    ArrowUp: "up",
    ArrowDown: "down",
  };
  if (directions[e.key])
    return e.shiftKey
      ? { kind: "move", dir: directions[e.key]!, extend: true }
      : { kind: "move", dir: directions[e.key]! };
  if (e.key === "Delete" || e.key === "Backspace") return { kind: "clear" };
  if (["Enter", "Escape"].includes(e.key)) return PLAIN[e.key] ?? null;
  return vim ? (PLAIN[remaps[e.key] ?? e.key] ?? null) : null;
}
