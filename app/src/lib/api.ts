// Typed wrappers around the Tauri commands in app/src-tauri/src/main.rs.
import { invoke } from "@tauri-apps/api/core";
import type { DiffBase, OpDto, SearchHitDto, ViewModel } from "./types";

export const api = {
  getView: (): Promise<ViewModel | null> => invoke<ViewModel | null>("get_view"),
  openPath: (path: string): Promise<ViewModel> => invoke<ViewModel>("open_path", { path }),
  openDialog: (): Promise<ViewModel | null> => invoke<ViewModel | null>("open_dialog"),
  save: (): Promise<ViewModel> => invoke<ViewModel>("save"),
  saveAsDialog: (): Promise<ViewModel | null> => invoke<ViewModel | null>("save_as_dialog"),
  applyOp: (op: OpDto): Promise<ViewModel> => invoke<ViewModel>("apply_op", { op }),
  undo: (): Promise<ViewModel> => invoke<ViewModel>("undo"),
  redo: (): Promise<ViewModel> => invoke<ViewModel>("redo"),
  searchKeycodes: (query: string, limit: number): Promise<SearchHitDto[]> =>
    invoke<SearchHitDto[]>("search_keycodes", { query, limit }),
  resolveKeycode: (text: string): Promise<string> => invoke<string>("resolve_keycode", { text }),
  setDiffBase: (base: DiffBase): Promise<ViewModel> => invoke<ViewModel>("set_diff_base", { base }),
  smokeReport: (renderedKeys: number, layerCount: number): Promise<void> =>
    invoke<void>("smoke_report", { renderedKeys, layerCount }),
};

export function errorText(e: unknown): string {
  return typeof e === "string" ? e : e instanceof Error ? e.message : JSON.stringify(e);
}
