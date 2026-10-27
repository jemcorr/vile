// Shared UI state (Svelte 5 runes). Rust owns the document; this holds only
// the latest view model plus purely visual state such as cursor and selection.
import type { KeyRefDto, ViewModel } from "./types";
import type { Mode, Panel } from "./modes";
import type { ChangeJump } from "./changes";

/** A button on a toast. A jump opens what the message names; a run calls back into the app. */
export type ToastAction = { label: string; jump: ChangeJump } | { label: string; run: () => void };

export const ui = $state({
  view: null as ViewModel | null,
  generation: 0,
  layer: 0,
  cursor: "",
  /** Counts arrow moves, so a zoomed board reveals the cursor key. */
  revealTick: 0,
  /** The key a Shift+arrow range grows from. null means the cursor. */
  anchor: null as string | null,
  selection: [] as string[],
  mode: "normal" as Mode,
  message: "",
  messageError: false,
  /** Full text behind a short message, shown on hover. */
  messageDetail: "",
  messageSeq: 0,
  /** A button on the toast that jumps to what the message names, or runs an action. */
  messageAction: null as ToastAction | null,
  /** A sticky toast stays until dismissed or replaced. */
  messageSticky: false,
  /** A jump a toast action asked for; the app performs it once and clears it. */
  jump: null as ChangeJump | null,
  panel: "combos" as Panel,
  /** A slot a feature editor selects once, then clears (jumps from the changes sheet). */
  revealSlot: null as { panel: Panel; index: number } | null,
  showCombos: false,
  inspectorPaths: [] as string[],
  baseTarget: false,
  comboEditing: null as number | null,
  drafts: {} as Record<string, boolean>,
  draftMacro: null as number | null,
  revealMacro: null as number | null,
  sideVisible: true,
  swapMark: null as KeyRefDto[] | null,
});

/** Show a status message. Failures pass error=true and stay until dismissed. */
export function notify(
  text: string,
  error = false,
  action: ToastAction | null = null,
  detail = "",
  sticky = false,
): void {
  ui.message = text;
  ui.messageError = error;
  ui.messageAction = action;
  ui.messageDetail = detail;
  ui.messageSticky = sticky;
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
