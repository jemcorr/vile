// Shared UI state (Svelte 5 runes). Rust owns the document; this holds only
// the latest view model plus purely visual state such as cursor and selection.
import type { KeyRefDto, ViewModel } from "./types";
import type { Mode, Panel } from "./modes";

export const ui = $state({
  view: null as ViewModel | null,
  layer: 0,
  cursor: "home-left-index",
  selection: [] as string[],
  mode: "normal" as Mode,
  message: "",
  panel: "diff" as Panel,
  showCombos: false,
  swapMark: null as KeyRefDto[] | null,
});

/** Replace the view model and keep layer, cursor and selection valid. */
export function setView(view: ViewModel | null): void {
  ui.view = view;
  if (view === null) {
    ui.layer = 0;
    ui.selection = [];
    return;
  }
  if (ui.layer >= view.layerCount) {
    ui.layer = 0;
  }
  const ids = new Set(view.board?.keys.map((k) => k.id) ?? []);
  if (!ids.has(ui.cursor)) {
    ui.cursor = view.board?.keys[0]?.id ?? "";
  }
  ui.selection = ui.selection.filter((id) => ids.has(id));
}
