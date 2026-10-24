// Mirrors crates/vile-session/src/dto.rs. Keep the two files in sync.

export type DiffBase = "disk" | "git";

export type KeyKind =
  | "no"
  | "trns"
  | "basic"
  | "modifier"
  | "mouse"
  | "media"
  | "mods"
  | "modTap"
  | "layerTap"
  | "layerMod"
  | "layer"
  | "oneShotMod"
  | "tapDance"
  | "macro"
  | "user"
  | "other";

export interface BoardKeyDto {
  id: string;
  row: number;
  col: number;
  x: number;
  y: number;
  w: number;
  h: number;
  name?: string | null;
  r?: number;
  rx?: number;
  ry?: number;
  secondary?: { x: number; y: number; w: number; h: number } | null;
}

export interface BoardDto {
  canMirror?: boolean;
  id: string;
  name: string;
  keys: BoardKeyDto[];
}

export interface KeyCellDto {
  id: string;
  wire: string;
  label: string;
  kind: KeyKind;
  changed: boolean;
  path: string;
  code: number;
  description: string;
  previous: string;
}

export interface DiffLineDto {
  section: string;
  line: string;
  layer: number | null;
  keyId: string | null;
  index: number | null;
  field: string | null;
  old: string;
  new: string;
  oldDescription: string;
  newDescription: string;
}

export interface ComboDto {
  index: number;
  keys: string[];
  output: string;
  outputLabel: string;
}

export type MacroActionKind = "tap" | "down" | "up" | "text" | "delay";

export interface MacroActionDto {
  kind: MacroActionKind;
  keys: string[];
  text: string;
  delay: number;
}

export interface MacroDto {
  index: number;
  actions: MacroActionDto[];
}

export interface TapDanceDto {
  index: number;
  onTap: string;
  onHold: string;
  onDoubleTap: string;
  onTapHold: string;
  tappingTerm: number;
}

export interface KeyOverrideDto {
  index: number;
  trigger: string;
  replacement: string;
  layers: number;
  triggerMods: number;
  negativeModMask: number;
  suppressedMods: number;
  options: number;
}

export interface AltRepeatDto {
  index: number;
  keycode: string;
  altKeycode: string;
  allowedMods: number;
  options: number;
}

/** uid is a string on purpose: it exceeds 2^53 and must never become a JS number. */
export type HoldBehavior =
  { kind: "none" } | { kind: "mod"; mods: number } | { kind: "layer"; layer: number };
export type LayerOp = "MO" | "TO" | "TG" | "TT" | "OSL" | "DF" | "PDF";
export type KeyParts =
  | { kind: "tap"; base: number; wrapMods: number; hold: HoldBehavior }
  | { kind: "layer"; op: "Mo" | "To" | "Tg" | "Tt" | "Osl" | "Df" | "Pdf"; layer: number }
  | { kind: "oneShot"; mods: number }
  | { kind: "layerMod"; layer: number; mods: number }
  | { kind: "raw"; code: number };
export interface KeyPartsPatch {
  base?: number;
  wrapMods?: number;
  hold?: HoldBehavior;
  layer?: number;
  mods?: number;
}
export interface KeycodeView {
  code: number;
  wire: string;
  label: string;
  description: string;
  category: string;
  parts: KeyParts;
}
export interface Names {
  layers: Record<string, string>;
  macros: Record<string, string>;
  combos: Record<string, string>;
}
export interface MacroUsage {
  bytes: number;
  actionBytes: number[];
}
export interface ViewModel {
  revision: number;
  names: Names;
  keycodes: Record<string, KeycodeView>;
  pendingDiff: DiffLineDto[];
  macroBytes: number;
  macroCapacity: number | null;
  geometrySource: "matrix" | "cache" | "device";
  path: string | null;
  uid: string;
  dirty: boolean;
  canUndo: boolean;
  canRedo: boolean;
  layerCount: number;
  board: BoardDto | null;
  layers: KeyCellDto[][];
  diff: DiffLineDto[];
  diffBase: DiffBase;
  warnings: string[];
  combos: ComboDto[];
  macros: MacroDto[];
  tapDances: TapDanceDto[];
  keyOverrides: KeyOverrideDto[];
  altRepeatKeys: AltRepeatDto[];
}

export interface SearchHitDto {
  pickerLabel: string;
  summary: string;
  pickerGroup: string;
  code: number | null;
  description: string;
  category: string;
  name: string;
  insert: string;
  complete: boolean;
  label: string;
  group: string;
  matched: string;
}

export interface KeyRefDto {
  layer: number;
  key: string;
}

export type MirrorModeDto = "swap" | "leftToRight" | "rightToLeft";

export type OpDto =
  | { type: "patchFields"; paths: string[]; patch: KeyPartsPatch }
  | { type: "replaceFields"; paths: string[]; keycode: string }
  | { type: "rename"; kind: "layer" | "macro" | "combo"; index: number; name: string }
  | { type: "setKeys"; layer: number; keys: string[]; keycode: string }
  | { type: "clearKeys"; layer: number; keys: string[] }
  | { type: "transparentKeys"; layer: number; keys: string[] }
  | { type: "copyKeys"; layer: number; keys: string[] }
  | { type: "pasteKeys"; layer: number; keys: string[] }
  | { type: "swapKeys"; a: KeyRefDto[]; b: KeyRefDto[] }
  | { type: "clearLayer"; layer: number; transparent: boolean }
  | { type: "copyLayer"; layer: number }
  | { type: "pasteLayer"; layer: number }
  | { type: "swapLayers"; a: number; b: number }
  | { type: "moveLayer"; from: number; to: number }
  | { type: "mirrorLayer"; layer: number; mode: MirrorModeDto }
  | { type: "setCombo"; index: number; keys: string[]; output: string }
  | { type: "setMacro"; index: number; actions: MacroActionDto[] }
  | {
      type: "setTapDance";
      index: number;
      onTap: string;
      onHold: string;
      onDoubleTap: string;
      onTapHold: string;
      tappingTerm: number;
    }
  | {
      type: "setKeyOverride";
      index: number;
      trigger: string;
      replacement: string;
      layers: number;
      triggerMods: number;
      negativeModMask: number;
      suppressedMods: number;
      options: number;
    }
  | {
      type: "setAltRepeat";
      index: number;
      keycode: string;
      altKeycode: string;
      allowedMods: number;
      options: number;
    };
