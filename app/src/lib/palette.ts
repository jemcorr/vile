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
  { id: "diffDisk", title: "Diff: against file on disk", arg: "none" },
  { id: "diffGit", title: "Diff: against git HEAD", arg: "none" },
  { id: "panelDiff", title: "Panel: diff", arg: "none" },
  { id: "panelCombos", title: "Panel: combos", arg: "none" },
  { id: "panelMacros", title: "Panel: macros", arg: "none" },
  { id: "panelTapDances", title: "Panel: tap dances", arg: "none" },
  { id: "panelKeyOverrides", title: "Panel: key overrides", arg: "none" },
  { id: "panelAltRepeat", title: "Panel: alt repeat keys", arg: "none" },
  { id: "toggleComboOverlay", title: "View: toggle combo overlay", arg: "none" },
];

const SHORTCUTS: Record<string, string> = {
  open: "⌘O",
  save: "⌘S",
  undo: "⌘Z",
  redo: "⇧⌘Z",
  copyKeys: "⌘C",
  pasteKeys: "⌘V",
  cutKeys: "⌘X",
  clearKeys: "Delete",
  editKey: "Enter",
  palette: "⌘K",
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
for (const command of COMMANDS) command.shortcut = SHORTCUTS[command.id] ?? "";
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
