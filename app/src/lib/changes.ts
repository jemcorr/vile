// Groups the changes against the comparison point in plain language. Rust sends the facts
// (diff lines, baseline values); this module only words and orders them.
import type { BaselineDto, DiffLineDto, IntentDto, KeyAtDto, KeyCellDto, ViewModel } from "./types";
import type { Panel } from "./modes";
import {
  ALT_REPEAT_OPTION_BITS,
  ALT_REPEAT_OPTION_LABELS,
  KEY_OVERRIDE_OPTION_BITS,
  KEY_OVERRIDE_OPTION_LABELS,
  MOD_BITS,
  MOD_LABELS,
  hasBit,
  type BitName,
} from "./bits";
import {
  altRepeatSet,
  altRepeatText,
  comboSet,
  comboText,
  currentLabels,
  flat,
  keyOverrideSet,
  keyOverrideText,
  macroSet,
  macroText,
  tapDanceSet,
  tapDanceText,
  type LabelOf,
} from "./slots";

export type ChangeJump =
  | { kind: "keys"; layer: number; keyIds: string[] }
  | { kind: "layer"; layer: number }
  | { kind: "slot"; panel: Panel; index: number };

export interface ChangeRow {
  id: string;
  text: string;
  detail: string;
  title: string;
  jump: ChangeJump | null;
}

export interface ChangeGroup {
  id: string;
  rank: number;
  title: string;
  count: string;
  jump: ChangeJump | null;
  /** Compact key changes, shown as wrapping chips. */
  chips: ChangeRow[];
  /** Everything else, one line each. */
  rows: ChangeRow[];
}

export type SlotSection = "combo" | "macro" | "tapDance" | "keyOverride" | "altRepeatKey";

const SLOT_SECTIONS: SlotSection[] = ["combo", "macro", "tapDance", "keyOverride", "altRepeatKey"];

const SLOT_TABLE: Record<SlotSection, { panel: Panel; title: string; group: string; rank: number; unit: [string, string] }> = {
  combo: { panel: "combos", title: "Combo", group: "Combos", rank: 1000, unit: ["combo", "combos"] },
  macro: { panel: "macros", title: "Macro", group: "Macros", rank: 1001, unit: ["macro", "macros"] },
  tapDance: { panel: "tapDances", title: "Tap dance", group: "Tap dances", rank: 1002, unit: ["tap dance", "tap dances"] },
  keyOverride: { panel: "keyOverrides", title: "Key override", group: "Key overrides", rank: 1003, unit: ["override", "overrides"] },
  altRepeatKey: { panel: "altRepeat", title: "Alt repeat", group: "Alt repeat", rank: 1004, unit: ["entry", "entries"] },
};

const SECTION_NAMES: Record<string, string> = {
  encoder: "Knobs",
  macro: "Macros",
  tapDance: "Tap dances",
  combo: "Combos",
  keyOverride: "Key overrides",
  altRepeatKey: "Alt repeat",
  settings: "QMK settings",
};

const META_LABELS: Record<string, string> = {
  version: "File format version",
  uid: "Keyboard ID",
  layout_options: "Layout options",
  vial_protocol: "Vial protocol",
  via_protocol: "VIA protocol",
  layers: "Number of layers",
  matrix: "Matrix size",
};

const TAP_DANCE_FIELDS: Record<string, { k: number; label: string }> = {
  tap: { k: 0, label: "Tap" },
  hold: { k: 1, label: "Hold" },
  double_tap: { k: 2, label: "Double tap" },
  tap_hold: { k: 3, label: "Tap and hold" },
};

const LAYER_BITS: BitName[] = Array.from({ length: 16 }, (_, bit) => ({ bit, name: `L${bit}` }));

const plural = (n: number, [one, many]: [string, string]): string => `${n} ${n === 1 ? one : many}`;

/** Key or keycap text on one line, without the curated picker words. */
export function shortLabel(key: { code: number; label: string } | null | undefined): string {
  if (!key) return "";
  if (key.code === 0) return "No key";
  if (key.code === 1) return "Transparent";
  return key.label.split("\n").join(" ").trim();
}

/** "+A +B −C" for the bits that turned on, then off. Masks are decimal strings. */
function maskDelta(before: string, after: string, bits: BitName[], label: (name: string) => string): string {
  const a = Number(before), b = Number(after);
  const on = bits.filter((bit) => !hasBit(a, bit.bit) && hasBit(b, bit.bit)).map((bit) => "+" + label(bit.name));
  const off = bits.filter((bit) => hasBit(a, bit.bit) && !hasBit(b, bit.bit)).map((bit) => "−" + label(bit.name));
  return [...on, ...off].join(" ");
}

/** "Enabled on, Replace off" for each option bit that changed. */
function optionDelta(before: string, after: string, bits: BitName[], labels: Record<string, string>): string {
  const a = Number(before), b = Number(after);
  return bits
    .filter((bit) => hasBit(a, bit.bit) !== hasBit(b, bit.bit))
    .map((bit) => `${labels[bit.name] ?? bit.name} ${hasBit(b, bit.bit) ? "on" : "off"}`)
    .join(", ");
}

function withRetryLine(lines: DiffLineDto[], view: ViewModel): DiffLineDto[] {
  const isRetry = (line: DiffLineDto) => line.section === "names" && line.index === null;
  if (lines.some(isRetry)) return lines;
  const retry = view.pendingDiff.find(isRetry);
  return retry ? [...lines, retry] : lines;
}

function isPresence(line: DiffLineDto): boolean {
  const state = (text: string) => text === "present" || text === "(absent)";
  return line.index === null && line.field === null && state(line.old) && state(line.new);
}

export function isSlotSection(section: string): section is SlotSection {
  return (SLOT_SECTIONS as string[]).includes(section);
}

/** Slot indices that differ at the comparison point, for the slot grid markers. */
export function changedSlots(view: ViewModel | null): Record<SlotSection, Set<number>> {
  const sets: Record<SlotSection, Set<number>> = {
    combo: new Set(),
    macro: new Set(),
    tapDance: new Set(),
    keyOverride: new Set(),
    altRepeatKey: new Set(),
  };
  for (const line of view?.diff ?? []) {
    if (line.index === null) continue;
    if (isSlotSection(line.section)) sets[line.section].add(line.index);
    if (line.section === "names" && (line.field === "combo" || line.field === "macro")) {
      sets[line.field].add(line.index);
    }
  }
  return sets;
}

export function changeCount(groups: ChangeGroup[]): number {
  return groups.reduce((total, group) => total + group.chips.length + group.rows.length, 0);
}

/**
 * Groups the changes by where they live. `lines` defaults to the story's rest (or the view's diff)
 * and `reference` to the story's reference (or the view's baseline). Each intent of the story is
 * one row or chip placed in front of the diff rows of its group.
 */
export function changeGroups(
  view: ViewModel,
  lines: DiffLineDto[] = view.story?.rest ?? view.diff,
  reference: BaselineDto = view.story?.reference ?? view.baseline,
): ChangeGroup[] {
  const input = withRetryLine(lines, view);
  const keys = view.board?.keys ?? [];
  const layerName = (layer: number) => view.names.layers[layer];
  const layerChips = new Map<number, ChangeRow[]>();
  const layerKnobs = new Map<number, ChangeRow[]>();
  const slotLines = new Map<SlotSection, Map<number, DiffLineDto[]>>();
  const names: ChangeRow[] = [];
  const settings: ChangeRow[] = [];
  const file: ChangeRow[] = [];
  const pushTo = <K, T>(map: Map<K, T[]>, key: K, value: T) => map.set(key, [...(map.get(key) ?? []), value]);

  for (const line of input) {
    if (line.section === "layout" && line.layer !== null && line.keyId !== null) {
      const index = keys.findIndex((key) => key.id === line.keyId);
      const key = keys[index];
      const cell: KeyCellDto | undefined = view.layers[line.layer]?.[index];
      if (!key || !cell) continue;
      const old = reference.keycodes[cell.path];
      const layer = line.layer;
      pushTo(layerChips, layer, {
        id: `key-${layer}-${line.keyId}`,
        text: `${shortLabel(old) || line.oldDescription} → ${shortLabel(cell)}`,
        detail: "",
        title: `${key.name ?? key.id}: ${old?.description ?? line.oldDescription} → ${cell.description} (${line.old} → ${line.new})`,
        jump: { kind: "keys", layer, keyIds: [line.keyId] },
      });
    } else if (line.section === "encoder" && line.layer !== null && line.index !== null && (line.field === "cw" || line.field === "ccw")) {
      const cw = line.field === "cw";
      const path = `encoder_layout[${line.layer}][${line.index}][${cw ? 0 : 1}]`;
      const text = `Knob ${line.index + 1} ${cw ? "clockwise" : "counterclockwise"}`;
      const detail = `${shortLabel(reference.keycodes[path]) || line.oldDescription} → ${shortLabel(view.keycodes[path]) || line.newDescription}`;
      pushTo(layerKnobs, line.layer, {
        id: `knob-${line.layer}-${line.index}-${line.field}`,
        text,
        detail,
        title: `${text}: ${detail}`,
        jump: { kind: "layer", layer: line.layer },
      });
    } else if (isSlotSection(line.section) && line.index !== null) {
      const sections = slotLines.get(line.section) ?? new Map<number, DiffLineDto[]>();
      slotLines.set(line.section, sections);
      pushTo(sections, line.index, line);
    } else if (line.section === "names") {
      names.push(nameRow(line));
    } else if (line.section === "settings" && line.field !== null) {
      const show = (value: string) => (value === "(none)" ? "not set" : value);
      const text = `QMK setting ${line.field}`;
      const detail = `${show(line.old)} → ${show(line.new)}`;
      settings.push({ id: `setting-${line.field}`, text, detail, title: `${text}: ${detail}`, jump: null });
    } else if (isPresence(line)) {
      const section = SECTION_NAMES[line.section] ?? line.section;
      const text = `${section} section`;
      const detail = line.new === "present" ? "Added" : "Removed";
      file.push({ id: `presence-${line.section}`, text, detail, title: `${text}: ${detail}`, jump: null });
    } else if (line.section === "meta" && line.field !== null) {
      const text = META_LABELS[line.field] ?? line.field;
      const detail = `${line.old} → ${line.new}`;
      file.push({ id: `meta-${line.field}`, text, detail, title: `${text}: ${detail}`, jump: null });
    }
  }

  const intentChips = new Map<number, ChangeRow[]>();
  const intentRows = new Map<number, ChangeRow[]>();
  const intentSlotRows = new Map<SlotSection, ChangeRow[]>();
  for (const [index, intent] of (view.story?.intents ?? []).entries()) {
    const placed = intentChange(view, intent, index);
    if (!placed) continue;
    if ("section" in placed.place) pushTo(intentSlotRows, placed.place.section, placed.row);
    else if (placed.chip) pushTo(intentChips, placed.place.layer, placed.row);
    else pushTo(intentRows, placed.place.layer, placed.row);
  }

  const groups: ChangeGroup[] = [];
  const layers = [...new Set([...layerChips.keys(), ...layerKnobs.keys(), ...intentChips.keys(), ...intentRows.keys()])].sort((a, b) => a - b);
  for (const layer of layers) {
    const chips = [...(intentChips.get(layer) ?? []), ...(layerChips.get(layer) ?? [])];
    const rows = [...(intentRows.get(layer) ?? []), ...(layerKnobs.get(layer) ?? [])];
    const name = layerName(layer);
    const holdsIntent = (intentChips.get(layer)?.length ?? 0) + (intentRows.get(layer)?.length ?? 0) > 0;
    const counts = holdsIntent
      ? [plural(chips.length + rows.length, ["change", "changes"])]
      : [
          chips.length ? plural(chips.length, ["key", "keys"]) : "",
          rows.length ? plural(rows.length, ["knob", "knobs"]) : "",
        ].filter(Boolean);
    groups.push({
      id: `layer-${layer}`,
      rank: layer,
      title: `Layer ${layer}${name ? ` · ${name}` : ""}`,
      count: counts.join(" · "),
      jump: { kind: "layer", layer },
      chips,
      rows,
    });
  }

  for (const section of SLOT_SECTIONS) {
    const bySlot = slotLines.get(section);
    const intentRowsHere = intentSlotRows.get(section) ?? [];
    if (!bySlot && !intentRowsHere.length) continue;
    const table = SLOT_TABLE[section];
    const rows = [
      ...intentRowsHere,
      ...[...(bySlot?.keys() ?? [])]
        .sort((a, b) => a - b)
        .map((index) => slotRow(section, index, bySlot?.get(index) ?? [], view, reference)),
    ];
    groups.push({
      id: `slot-${section}`,
      rank: table.rank,
      title: table.group,
      count: plural(rows.length, table.unit),
      jump: rows[0]?.jump ?? null,
      chips: [],
      rows,
    });
  }

  if (names.length) {
    groups.push({ id: "names", rank: 1005, title: "Names", count: plural(names.length, ["name", "names"]), jump: null, chips: [], rows: names });
  }
  if (settings.length) {
    groups.push({
      id: "settings",
      rank: 1006,
      title: "QMK settings",
      count: plural(settings.length, ["setting", "settings"]),
      jump: null,
      chips: [],
      rows: settings,
    });
  }
  if (file.length) {
    groups.push({ id: "file", rank: 1007, title: "File", count: plural(file.length, ["change", "changes"]), jump: null, chips: [], rows: file });
  }
  const reorder = view.story?.intents.find((intent) => intent.kind === "layersReordered");
  if (reorder?.kind === "layersReordered") groups.push(layerOrderGroup(view, reorder.order, reorder.renumbered));
  return groups.sort((a, b) => a.rank - b.rank);
}

/**
 * The layers in another order, as one row: a swap, a move, or a count of the layers that moved.
 * `order[now] = old`. Layer names come from the saved names, by the original layer number.
 */
function layerOrderGroup(view: ViewModel, order: number[], renumbered: boolean): ChangeGroup {
  const name = (layer: number) => {
    const saved = view.baseline.names.layers[layer];
    return saved ? `${saved} (${layer})` : `layer ${layer}`;
  };
  const moved = order.flatMap((old, now) => (old === now ? [] : [now]));
  const swapped = moved.length === 2 && order[moved[0]!] === moved[1] && order[moved[1]!] === moved[0];
  const single = swapped ? undefined : singleMove(order);
  const text = swapped
    ? `Swapped ${name(moved[0]!)} and ${name(moved[1]!)}`
    : single
      ? `Moved ${name(single.from)} to position ${single.to}`
      : `Reordered ${moved.length} layers`;
  const lines = moved
    .map((now) => order[now]!)
    .sort((a, b) => a - b)
    .map((old) => `Layer ${old} is now layer ${order.indexOf(old)}.`);
  const row: ChangeRow = {
    id: "layer-order-row",
    text,
    detail: renumbered ? "Layer keys renumbered" : "",
    title: `${text}. ${lines.join(" ")}`,
    jump: { kind: "layer", layer: moved[0]! },
  };
  return { id: "layer-order", rank: -1, title: "Layer order", count: plural(1, ["change", "changes"]), jump: row.jump, chips: [], rows: [row] };
}

/** The one move that gives `order`, as `from` (old layer) and `to` (new position), or null. */
function singleMove(order: number[]): { from: number; to: number } | null {
  const n = order.length;
  for (let from = 0; from < n; from++) {
    for (let to = 0; to < n; to++) {
      if (from === to) continue;
      const moved = [...order.keys()].filter((i) => i !== from);
      moved.splice(to, 0, from);
      if (moved.every((old, now) => old === order[now])) return { from, to };
    }
  }
  return null;
}

/** Feature names of the story (`IntentDto.feature`) to slot sections. */
const FEATURE_SECTIONS: Record<string, SlotSection> = {
  combo: "combo",
  macro: "macro",
  tapDance: "tapDance",
  keyOverride: "keyOverride",
  altRepeat: "altRepeatKey",
};

/** The current cell of a key, from its board key id. */
function cellOf(view: ViewModel, key: KeyAtDto): KeyCellDto | undefined {
  const index = view.board?.keys.findIndex((board) => board.id === key.keyId) ?? -1;
  return index < 0 ? undefined : view.layers[key.layer]?.[index];
}

/** The key's name when it has one, else its id ("r0c2"). */
function keyName(view: ViewModel, key: KeyAtDto): string {
  return view.board?.keys.find((board) => board.id === key.keyId)?.name || key.keyId;
}

/** The current label of a key, for "W ⇄ L" style text. */
function keyLabel(view: ViewModel, key: KeyAtDto): string {
  return shortLabel(cellOf(view, key)) || key.keyId;
}

/** Where an intent is shown, and how. A chip sits among the key chips of its layer. */
type IntentPlace = { layer: number } | { section: SlotSection };

/** One intent as a chip or row. Null for an intent the sheet cannot name. */
function intentChange(view: ViewModel, intent: IntentDto, index: number): { place: IntentPlace; chip: boolean; row: ChangeRow } | null {
  const id = `intent-${intent.kind}-${index}`;
  switch (intent.kind) {
    case "layersReordered":
      // Shown as the layer-order group, not as a row of a layer.
      return null;
    case "layerFilled": {
      const detail = plural(intent.keys, ["key", "keys"]);
      const text =
        intent.value === "KC_NO"
          ? "Cleared all keys"
          : intent.value === "KC_TRNS"
            ? "Made all keys transparent"
            : `Set all keys to ${intent.value}`;
      return { place: { layer: intent.layer }, chip: false, row: { id, text, detail, title: `${text}: ${detail}`, jump: { kind: "layer", layer: intent.layer } } };
    }
    case "layerCopied": {
      const name = view.names.layers[intent.from];
      const text = `Copied from layer ${intent.from}${name ? ` · ${name}` : ""}`;
      const detail = plural(intent.keys, ["key", "keys"]);
      return { place: { layer: intent.to }, chip: false, row: { id, text, detail, title: `${text}: ${detail}`, jump: { kind: "layer", layer: intent.to } } };
    }
    case "keysSwapped": {
      const { a, b } = intent;
      const sameLayer = a.layer === b.layer;
      const text = `${keyLabel(view, a)} ⇄ ${keyLabel(view, b)}${sameLayer ? "" : ` (layer ${b.layer})`}`;
      const title = `Swapped ${keyName(view, a)} and ${keyName(view, b)}${sameLayer ? "" : `, on layers ${a.layer} and ${b.layer}`}`;
      return {
        place: { layer: a.layer },
        chip: true,
        row: { id, text, detail: "", title, jump: { kind: "keys", layer: a.layer, keyIds: sameLayer ? [a.keyId, b.keyId] : [a.keyId] } },
      };
    }
    case "keyMoved": {
      const { from, to } = intent;
      const cell = cellOf(view, to);
      const replaced = cell ? view.baseline.keycodes[cell.path] : undefined;
      const replacedText = replaced && replaced.code !== 0 ? ` It replaced ${shortLabel(replaced)}.` : "";
      const text = `Moved ${keyLabel(view, to)}`;
      const title = `${text} from ${keyName(view, from)} to ${keyName(view, to)}. ${keyName(view, from)} is now empty.${replacedText}`;
      return { place: { layer: to.layer }, chip: true, row: { id, text, detail: "", title, jump: { kind: "keys", layer: to.layer, keyIds: [to.keyId] } } };
    }
    case "keysFilled": {
      const n = intent.keys.length;
      const text =
        intent.value === "KC_NO"
          ? `Cleared ${n} keys`
          : intent.value === "KC_TRNS"
            ? `Made ${n} keys transparent`
            : `Set ${n} keys to ${intent.value}`;
      return { place: { layer: intent.layer }, chip: false, row: { id, text, detail: "", title: text, jump: { kind: "keys", layer: intent.layer, keyIds: intent.keys } } };
    }
    case "slotsSwapped":
    case "slotCopied": {
      const section = FEATURE_SECTIONS[intent.feature];
      if (!section) return null;
      const table = SLOT_TABLE[section];
      const word = table.title.toLowerCase();
      if (intent.kind === "slotCopied") {
        const text = `Copied ${word} ${intent.from} to ${word} ${intent.to}`;
        return { place: { section }, chip: false, row: { id, text, detail: "", title: text, jump: { kind: "slot", panel: table.panel, index: intent.to } } };
      }
      const { a, b } = intent;
      let text = `Swapped ${word} ${a} and ${word} ${b}`;
      let index = a;
      if (!slotIsSet(view, section, a) && slotIsSet(view, section, b)) {
        text = `Moved ${word} ${a} to ${word} ${b}`;
        index = b;
      } else if (slotIsSet(view, section, a) && !slotIsSet(view, section, b)) {
        text = `Moved ${word} ${b} to ${word} ${a}`;
        index = a;
      }
      const detail = intent.renumbered ? "References renumbered" : "";
      return { place: { section }, chip: false, row: { id, text, detail, title: detail ? `${text}: ${detail}` : text, jump: { kind: "slot", panel: table.panel, index } } };
    }
  }
}

function nameRow(line: DiffLineDto): ChangeRow {
  if (line.index === null) {
    const detail = "Not saved yet. Saving retries it.";
    return { id: "names-file", text: "Names file", detail, title: `Names file: ${detail}`, jump: null };
  }
  const field = line.field ?? "";
  const text = `${field.charAt(0).toUpperCase()}${field.slice(1)} ${line.index}`;
  const detail = !line.old
    ? `Named “${line.new}”`
    : !line.new
      ? `Name removed (was “${line.old}”)`
      : `“${line.old}” → “${line.new}”`;
  const jump: ChangeJump | null =
    field === "layer"
      ? { kind: "layer", layer: line.index }
      : field === "macro"
        ? { kind: "slot", panel: "macros", index: line.index }
        : field === "combo"
          ? { kind: "slot", panel: "combos", index: line.index }
          : null;
  return {
    id: `name-${field}-${line.index}`,
    text,
    detail,
    title: `${detail}. Names are saved next to the layout and compared with the last save.`,
    jump,
  };
}

/** Current and comparison-point summaries of a combo or macro slot, without names. */
/** Whether slot `index` of `section` holds anything in the view. */
function slotIsSet(view: ViewModel, section: SlotSection, index: number): boolean {
  switch (section) {
    case "combo":
      return !!view.combos.find((slot) => slot.index === index && comboSet(slot));
    case "macro":
      return !!view.macros.find((slot) => slot.index === index && macroSet(slot));
    case "tapDance":
      return !!view.tapDances.find((slot) => slot.index === index && tapDanceSet(slot));
    case "keyOverride":
      return !!view.keyOverrides.find((slot) => slot.index === index && keyOverrideSet(slot));
    case "altRepeatKey":
      return !!view.altRepeatKeys.find((slot) => slot.index === index && altRepeatSet(slot));
  }
}

function slotState(
  section: SlotSection,
  index: number,
  view: ViewModel,
  reference: BaselineDto,
): { curSet: boolean; baseSet: boolean; curText: string; baseText: string } {
  const curLabel = currentLabels(view);
  const refLabel: LabelOf = (path) => reference.keycodes[path]?.label ?? view.keycodes[path]?.label;
  switch (section) {
    case "combo": {
      const cur = view.combos.find((slot) => slot.index === index);
      const base = reference.combos.find((slot) => slot.index === index);
      return {
        curSet: !!cur && comboSet(cur),
        baseSet: !!base && comboSet(base),
        curText: cur ? flat(comboText(cur, curLabel)) : "",
        baseText: base ? flat(comboText(base, refLabel)) : "",
      };
    }
    case "macro": {
      const cur = view.macros.find((slot) => slot.index === index);
      const base = reference.macros.find((slot) => slot.index === index);
      return {
        curSet: !!cur && macroSet(cur),
        baseSet: !!base && macroSet(base),
        curText: cur ? flat(macroText(cur, curLabel)) : "",
        baseText: base ? flat(macroText(base, refLabel)) : "",
      };
    }
    case "tapDance": {
      const cur = view.tapDances.find((slot) => slot.index === index);
      const base = reference.tapDances.find((slot) => slot.index === index);
      return {
        curSet: !!cur && tapDanceSet(cur),
        baseSet: !!base && tapDanceSet(base),
        curText: cur ? flat(tapDanceText(cur, curLabel)) : "",
        baseText: base ? flat(tapDanceText(base, refLabel)) : "",
      };
    }
    case "keyOverride": {
      const cur = view.keyOverrides.find((slot) => slot.index === index);
      const base = reference.keyOverrides.find((slot) => slot.index === index);
      return {
        curSet: !!cur && keyOverrideSet(cur),
        baseSet: !!base && keyOverrideSet(base),
        curText: cur ? flat(keyOverrideText(cur, curLabel)) : "",
        baseText: base ? flat(keyOverrideText(base, refLabel)) : "",
      };
    }
    case "altRepeatKey": {
      const cur = view.altRepeatKeys.find((slot) => slot.index === index);
      const base = reference.altRepeatKeys.find((slot) => slot.index === index);
      return {
        curSet: !!cur && altRepeatSet(cur),
        baseSet: !!base && altRepeatSet(base),
        curText: cur ? flat(altRepeatText(cur, curLabel)) : "",
        baseText: base ? flat(altRepeatText(base, refLabel)) : "",
      };
    }
  }
}

/** One row for a feature slot, with every field change of that slot in its detail. */
function slotRow(section: SlotSection, index: number, lines: DiffLineDto[], view: ViewModel, reference: BaselineDto): ChangeRow {
  const table = SLOT_TABLE[section];
  const jump: ChangeJump = { kind: "slot", panel: table.panel, index };
  let text = `${table.title} ${index}`;
  const name = section === "combo" ? view.names.combos[index] : section === "macro" ? view.names.macros[index] : undefined;
  if (name) text += ` · ${name}`;
  const { curSet, baseSet, curText, baseText } = slotState(section, index, view, reference);
  let detail: string;
  if (curSet && !baseSet) detail = `New: ${curText}`;
  else if (!curSet && baseSet) detail = `Removed: ${baseText}`;
  else if (section === "combo" || section === "macro") {
    detail = !curSet && !baseSet ? "Changed" : curText === baseText ? "Changed" : `${curText} · was ${baseText}`;
  } else {
    detail = lines.map((line) => slotFieldPhrase(section, index, line, view, reference)).join(" · ");
  }
  return { id: `slot-${section}-${index}`, text, detail, title: `${text}: ${detail}`, jump };
}

function slotFieldPhrase(section: SlotSection, index: number, line: DiffLineDto, view: ViewModel, reference: BaselineDto): string {
  const field = line.field ?? "";
  const pair = (path: string, label: string) =>
    `${label}: ${shortLabel(reference.keycodes[path]) || line.oldDescription} → ${shortLabel(view.keycodes[path]) || line.newDescription}`;
  if (section === "tapDance") {
    if (field === "term") return `Tapping term: ${line.old} → ${line.new} ms`;
    const spec = TAP_DANCE_FIELDS[field];
    return spec ? pair(`tap_dance[${index}][${spec.k}]`, spec.label) : field;
  }
  if (section === "keyOverride") {
    const path = (name: string) => `key_override[${index}].${name}`;
    switch (field) {
      case "trigger":
        return pair(path("trigger"), "Trigger");
      case "replacement":
        return pair(path("replacement"), "Replacement");
      case "layers":
        return `Layers: ${maskDelta(line.old, line.new, LAYER_BITS, (name) => name)}`;
      case "trigger_mods":
        return `Trigger mods: ${maskDelta(line.old, line.new, MOD_BITS, (name) => MOD_LABELS[name] ?? name)}`;
      case "negative_mod_mask":
        return `Negative mods: ${maskDelta(line.old, line.new, MOD_BITS, (name) => MOD_LABELS[name] ?? name)}`;
      case "suppressed_mods":
        return `Suppressed mods: ${maskDelta(line.old, line.new, MOD_BITS, (name) => MOD_LABELS[name] ?? name)}`;
      case "options":
        return optionDelta(line.old, line.new, KEY_OVERRIDE_OPTION_BITS, KEY_OVERRIDE_OPTION_LABELS);
      default:
        return field;
    }
  }
  if (section === "altRepeatKey") {
    switch (field) {
      case "keycode":
        return pair(`alt_repeat_key[${index}].keycode`, "Key");
      case "alt_keycode":
        return pair(`alt_repeat_key[${index}].alt_keycode`, "Alternate");
      case "allowed_mods":
        return `Allowed mods: ${maskDelta(line.old, line.new, MOD_BITS, (name) => MOD_LABELS[name] ?? name)}`;
      case "options":
        return optionDelta(line.old, line.new, ALT_REPEAT_OPTION_BITS, ALT_REPEAT_OPTION_LABELS);
      default:
        return field;
    }
  }
  return line.line;
}

/** The wording of a failed comparison with the last Git commit. */
export function compareErrorText(raw: string, file: string): string {
  const prefix = "Can't compare with the last commit: ";
  if (/not a git repository/i.test(raw)) return `${prefix}${file} is not in a Git repository.`;
  if (/does not exist in|exists on disk, but not in|invalid object name|bad revision|unknown revision/i.test(raw)) {
    return `${prefix}${file} has no commit yet.`;
  }
  if (/git is not installed/i.test(raw)) return `${prefix}Git is not installed.`;
  return prefix + raw;
}
