// Typed wrappers around the Tauri commands in app/src-tauri/src/main.rs.
import { invoke } from "@tauri-apps/api/core";
import type {
  DiffBase,
  OpDto,
  SearchHitDto,
  ViewModel,
  KeycodeView,
  KeyParts,
  MacroActionDto,
  MacroUsage,
} from "./types";

let revision = 0;
let queue: Promise<unknown> = Promise.resolve();
function observe<T extends ViewModel | null>(view: T): T {
  if (view) revision = Math.max(revision, view.revision);
  return view;
}
function mutation<T extends ViewModel | null>(
  command: string,
  args: Record<string, unknown> = {},
): Promise<T> {
  const expected = revision;
  const next = queue.then(() => invoke<T>(command, { ...args, revision: expected })).then(observe);
  queue = next.catch(() => undefined);
  return next;
}
export const api = {
  getView: (): Promise<ViewModel | null> => invoke<ViewModel | null>("get_view").then(observe),
  openPath: (path: string): Promise<ViewModel> =>
    mutation<ViewModel>("open_path", { path, discard: false }),
  openDialog: (discard = false): Promise<ViewModel | null> =>
    mutation<ViewModel | null>("open_dialog", { discard }),
  save: (): Promise<ViewModel> => mutation<ViewModel>("save"),
  saveAsDialog: (): Promise<ViewModel | null> => mutation<ViewModel | null>("save_as_dialog"),
  exportLayers: (png: Uint8Array, revision: number): Promise<string | null> =>
    invoke<string | null>("export_layers_dialog", { png: Array.from(png), revision }),
  applyOp: (op: OpDto): Promise<ViewModel> => mutation<ViewModel>("apply_op", { op }),
  undo: (): Promise<ViewModel> => mutation<ViewModel>("undo"),
  redo: (): Promise<ViewModel> => mutation<ViewModel>("redo"),
  searchKeycodes: (query: string, limit: number): Promise<SearchHitDto[]> =>
    invoke<SearchHitDto[]>("search_keycodes", { query, limit }),
  resolveKeycode: (text: string): Promise<string> => invoke<string>("resolve_keycode", { text }),
  setDiffBase: (base: DiffBase): Promise<ViewModel> =>
    mutation<ViewModel>("set_diff_base", { base }),
  keycodeView: (text: string): Promise<KeycodeView> => invoke("keycode_view", { text }),
  composeKeycode: (parts: KeyParts): Promise<KeycodeView> => invoke("compose_keycode", { parts }),
  macroUsage: (actions: MacroActionDto[]): Promise<MacroUsage> =>
    invoke("macro_usage", { actions }),
  refreshDefinition: (): Promise<ViewModel> =>
    invoke<ViewModel>("refresh_definition", { revision }).then(observe),
  closeWindow: (): Promise<void> => invoke("close_window", { revision }),
  updateSupport: (): Promise<void> => invoke<void>("update_support"),
  restartApp: (discard: boolean): Promise<void> =>
    invoke<void>("restart_app", { revision, discard }),
  smokeReport: (renderedKeys: number, layerCount: number, boardName: string): Promise<void> =>
    invoke<void>("smoke_report", { renderedKeys, layerCount, boardName }),
};

export function errorText(e: unknown): string {
  return typeof e === "string" ? e : e instanceof Error ? e.message : JSON.stringify(e);
}
