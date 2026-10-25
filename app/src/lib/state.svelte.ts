// Shared UI state (Svelte 5 runes). Rust owns the document; this holds only
// the latest view model plus purely visual state such as cursor and selection.
import type { KeyRefDto, ViewModel } from "./types";
import type { Mode, Panel } from "./modes";

export const ui = $state({
  view: null as ViewModel | null,
  generation: 0,
  layer: 0,
  cursor: "",
  /** The key a Shift+arrow range grows from. null means the cursor. */
  anchor: null as string | null,
  selection: [] as string[],
  mode: "normal" as Mode,
  message: "",
  messageError: false,
  messageSeq: 0,
  panel: "diff" as Panel,
  showCombos: false,
  inspectorPaths: [] as string[],
  baseTarget: false,
  comboEditing: null as number | null,
  drafts: {} as Record<string, boolean>,
  draftMacro: null as number | null,
  revealMacro: null as number | null,
  sideVisible: true,
  vimKeys: false,
  vimRemaps: {} as Record<string, string>,
  swapMark: null as KeyRefDto[] | null,
});

/** Show a status message. Failures pass error=true and stay until dismissed. */
export function notify(text: string, error = false): void {
  ui.message = text;
  ui.messageError = error;
  ui.messageSeq += 1;
}

/** Replace the view model and keep layer, cursor and selection valid. */
export function setView(view: ViewModel | null): void {
  if (view && ui.view && view.revision < ui.view.revision) return;
  ui.view = view;
  if (view === null) {
    ui.layer = 0;
    ui.selection = [];
    ui.anchor = null;
    return;
  }
  if (ui.layer >= view.layerCount) {
    ui.layer = 0;
  }
  const ids = new Set(view.board?.keys.map((k) => k.id) ?? []);
  if (!ids.has(ui.cursor)) {
    ui.cursor = view.board?.keys[0]?.id ?? "";
  }
  if (ui.anchor !== null && !ids.has(ui.anchor)) ui.anchor = null;
  ui.selection = ui.selection.filter((id) => ids.has(id));
  ui.inspectorPaths = ui.inspectorPaths.filter((path) => view.keycodes[path] !== undefined);
}
