import { primaryModifierLabel } from "./keymap";

export interface PaletteCommand {
  id: string;
  title: string;
  arg: "none" | "layer";
  shortcut?: string;
}

export const COMMANDS: PaletteCommand[] = [
  { id: "open", title: "Open file…", arg: "none" },
  { id: "save", title: "Save", arg: "none" },
  { id: "saveAs", title: "Save as…", arg: "none" },
  { id: "exportPng", title: "Export PNG…", arg: "none" },
  { id: "undo", title: "Undo", arg: "none" },
  { id: "redo", title: "Redo", arg: "none" },
  { id: "clearLayer", title: "Layer: clear to KC_NO", arg: "none" },
  { id: "transparentLayer", title: "Layer: make transparent", arg: "none" },
  { id: "copyLayer", title: "Layer: copy", arg: "none" },
  { id: "pasteLayer", title: "Layer: paste", arg: "none" },
  { id: "swapLayer", title: "Layer: swap with…", arg: "layer" },
  { id: "moveLayer", title: "Layer: move to…", arg: "layer" },
  { id: "mirrorLeftToRight", title: "Layer: mirror left to right", arg: "none" },
  { id: "mirrorRightToLeft", title: "Layer: mirror right to left", arg: "none" },
  { id: "mirrorSwap", title: "Layer: swap halves", arg: "none" },
  { id: "diffDisk", title: "Compare with the last save", arg: "none" },
  { id: "diffGit", title: "Compare with the last Git commit", arg: "none" },
  { id: "panelDiff", title: "Review changes", arg: "none" },
  { id: "panelCombos", title: "Panel: combos", arg: "none" },
  { id: "panelMacros", title: "Panel: macros", arg: "none" },
  { id: "panelTapDances", title: "Panel: tap dances", arg: "none" },
  { id: "panelKeyOverrides", title: "Panel: key overrides", arg: "none" },
  { id: "panelAltRepeat", title: "Panel: alt repeat keys", arg: "none" },
  { id: "toggleComboOverlay", title: "View: toggle combo overlay", arg: "none" },
  { id: "zoomIn", title: "View: zoom in", arg: "none" },
  { id: "zoomOut", title: "View: zoom out", arg: "none" },
  { id: "zoomFit", title: "View: fit keyboard", arg: "none" },
];

const SHORTCUTS: Record<string, string | ((modifier: string) => string)> = {
  open: (modifier) => `${modifier}O`,
  save: (modifier) => `${modifier}S`,
  undo: (modifier) => `${modifier}Z`,
  redo: (modifier) => (isMacModifier(modifier) ? `⇧${modifier}Z` : `${modifier}Shift+Z`),
  copyKeys: (modifier) => `${modifier}C`,
  pasteKeys: (modifier) => `${modifier}V`,
  cutKeys: (modifier) => `${modifier}X`,
  clearKeys: "Delete",
  editKey: "Enter",
  palette: (modifier) => `${modifier}K`,
  zoomIn: (modifier) => `${modifier}=`,
  zoomOut: (modifier) => `${modifier}−`,
  zoomFit: (modifier) => `${modifier}0`,
};
COMMANDS.push(
  { id: "clearKeys", title: "Clear selected keys", arg: "none" },
  { id: "transparentKeys", title: "Make selected keys transparent", arg: "none" },
  { id: "copyKeys", title: "Copy selected keys", arg: "none" },
  { id: "cutKeys", title: "Cut selected keys", arg: "none" },
  { id: "pasteKeys", title: "Paste keys", arg: "none" },
  { id: "swapKeys", title: "Swap selected keys with…", arg: "none" },
  { id: "editKey", title: "Type keycode", arg: "none" },
);
function isMacModifier(modifier: string): boolean {
  return modifier === "⌘";
}

export function updateShortcutLabels(platform: string): void {
  const modifier = primaryModifierLabel(platform);
  for (const command of COMMANDS) {
    const shortcut = SHORTCUTS[command.id];
    command.shortcut = typeof shortcut === "function" ? shortcut(modifier) : (shortcut ?? "");
  }
}

updateShortcutLabels(typeof navigator === "undefined" ? "" : navigator.platform);
export function commandReason(
  id: string,
  view: { canUndo: boolean; canRedo: boolean; board: { canMirror?: boolean } | null } | null,
  hasSelection: boolean,
): string {
  if (id === "open") return "";
  if (!view) return "Open a layout first";
  if (id === "undo" && !view.canUndo) return "Nothing to undo";
  if (id === "redo" && !view.canRedo) return "Nothing to redo";
  if (id.startsWith("mirror") && !view.board?.canMirror)
    return "The definition does not supply validated left/right pairs";
  if (
    [
      "clearKeys",
      "transparentKeys",
      "copyKeys",
      "cutKeys",
      "pasteKeys",
      "swapKeys",
      "editKey",
    ].includes(id) &&
    !hasSelection
  )
    return "Select a key first";
  return "";
}

/** How well a title matches a query: 3 prefix, 2 word prefix, 1 substring, 0.5 subsequence, else null. */
function score(title: string, q: string): number | null {
  const t = title.toLowerCase();
  if (t.startsWith(q)) return 3;
  if (t.split(/[\s:]+/).some((word) => word.startsWith(q))) return 2;
  if (t.includes(q)) return 1;
  let i = 0;
  for (const ch of t) {
    if (i < q.length && ch === q[i]) i++;
  }
  return i === q.length ? 0.5 : null;
}

export function filterCommands(
  query: string,
  commands: PaletteCommand[] = COMMANDS,
): PaletteCommand[] {
  const q = query.trim().toLowerCase();
  if (q === "") return [...commands];
  const scored: { command: PaletteCommand; score: number }[] = [];
  for (const command of commands) {
    const s = score(command.title, q);
    if (s !== null) scored.push({ command, score: s });
  }
  return scored.sort((a, b) => b.score - a.score).map((entry) => entry.command);
}

/** A layer number typed by the user, or null when it is not a number below layerCount. */
export function parseLayerArg(text: string, layerCount: number): number | null {
  const trimmed = text.trim();
  if (!/^\d+$/.test(trimmed)) return null;
  const n = Number(trimmed);
  return n < layerCount ? n : null;
}
