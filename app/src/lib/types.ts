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
  /** Null when matrix geometry cannot confirm installed knobs. */
  encoderIndices?: number[] | null;
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
  tooltip: string;
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
  tooltip: string;
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
export interface BaselineDto {
  /** Keycode fields whose baseline value differs, by path ("layout[0][1][2]", "combo[3][4]"). */
  keycodes: Record<string, KeycodeView>;
  combos: ComboDto[];
  macros: MacroDto[];
  tapDances: TapDanceDto[];
  keyOverrides: KeyOverrideDto[];
  altRepeatKeys: AltRepeatDto[];
  /** Names as last saved; names are always compared with the last save. */
  names: Names;
}
/** One key position: its layer and board key id. */
export interface KeyAtDto {
  layer: number;
  keyId: string;
}

/** One change as a user-level action. `value` is a keycode wire name; `feature` is a slot family. */
export type IntentDto =
  | { kind: "layersReordered"; order: number[]; renumbered: boolean }
  | { kind: "layerFilled"; layer: number; value: string; keys: number }
  | { kind: "layerCopied"; from: number; to: number; keys: number }
  | { kind: "keysSwapped"; a: KeyAtDto; b: KeyAtDto }
  | { kind: "keyMoved"; from: KeyAtDto; to: KeyAtDto }
  | { kind: "keysFilled"; layer: number; value: string; keys: string[] }
  | { kind: "slotsSwapped"; feature: string; a: number; b: number }
  | { kind: "slotCopied"; feature: string; from: number; to: number };

/** The changes explained as intents, with what the intents leave unexplained. */
export interface StoryDto {
  intents: IntentDto[];
  /** Changes the intents do not explain, compared with the reference after the intents. */
  rest: DiffLineDto[];
  /** Baseline values of that reference, for the rest rows. */
  reference: BaselineDto;
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
  baseline: BaselineDto;
  /** The changes as intents, or null when there is none. */
  story: StoryDto | null;
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
  | { type: "copyFeatureSlot"; kind: "combo" | "macro" | "tapDance" | "keyOverride" | "altRepeat"; index: number }
  | { type: "pasteFeatureSlot"; kind: "combo" | "macro" | "tapDance" | "keyOverride" | "altRepeat"; index: number }
  | { type: "swapFeatureSlots"; kind: "combo" | "macro" | "tapDance" | "keyOverride" | "altRepeat"; a: number; b: number }
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
