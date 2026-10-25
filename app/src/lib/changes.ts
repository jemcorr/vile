// Groups the changes against the comparison point in plain language. Rust sends the facts
// (diff lines, baseline values); this module only words and orders them.
import type { BaselineDto, DiffLineDto, KeyCellDto, ViewModel } from "./types";
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
import { comboSet, comboText, currentLabels, flat, macroSet, macroText, type LabelOf } from "./slots";

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
 * Groups the diff lines by where the change lives. `lines` defaults to the view's diff and
 * `reference` to its baseline values. Stories (Phase 3) pass their own rest lines and reference.
 */
export function changeGroups(view: ViewModel, lines: DiffLineDto[] = view.diff, reference: BaselineDto = view.baseline): ChangeGroup[] {
  const input = withRetryLine(lines, view);
  const keys = view.board?.keys ?? [];
  const layerName = (layer: number) => view.names.layers[layer];
  const layerChips = new Map<number, ChangeRow[]>();
  const layerKnobs = new Map<number, ChangeRow[]>();
  const slotLines = new Map<SlotSection, Map<number, DiffLineDto[]>>();
  const names: ChangeRow[] = [];
  const settings: ChangeRow[] = [];
  const file: ChangeRow[] = [];
  const pushTo = <T>(map: Map<number, T[]>, key: number, value: T) => map.set(key, [...(map.get(key) ?? []), value]);

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

  const groups: ChangeGroup[] = [];
  const layers = [...new Set([...layerChips.keys(), ...layerKnobs.keys()])].sort((a, b) => a - b);
  for (const layer of layers) {
    const chips = layerChips.get(layer) ?? [];
    const rows = layerKnobs.get(layer) ?? [];
    const name = layerName(layer);
    const counts = [
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
    if (!bySlot) continue;
    const table = SLOT_TABLE[section];
    const rows = [...bySlot.keys()]
      .sort((a, b) => a - b)
      .map((index) => slotRow(section, index, bySlot.get(index) ?? [], view, reference));
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
  return groups.sort((a, b) => a.rank - b.rank);
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
function slotState(
  section: "combo" | "macro",
  index: number,
  view: ViewModel,
  reference: BaselineDto,
): { curSet: boolean; baseSet: boolean; curText: string; baseText: string } {
  const curLabel = currentLabels(view);
  const refLabel: LabelOf = (path) => reference.keycodes[path]?.label ?? view.keycodes[path]?.label;
  if (section === "combo") {
    const cur = view.combos.find((slot) => slot.index === index);
    const base = reference.combos.find((slot) => slot.index === index);
    return {
      curSet: !!cur && comboSet(cur),
      baseSet: !!base && comboSet(base),
      curText: cur ? flat(comboText(cur, curLabel)) : "",
      baseText: base ? flat(comboText(base, refLabel)) : "",
    };
  }
  const cur = view.macros.find((slot) => slot.index === index);
  const base = reference.macros.find((slot) => slot.index === index);
  return {
    curSet: !!cur && macroSet(cur),
    baseSet: !!base && macroSet(base),
    curText: cur ? flat(macroText(cur, curLabel)) : "",
    baseText: base ? flat(macroText(base, refLabel)) : "",
  };
}

/** One row for a feature slot, with every field change of that slot in its detail. */
function slotRow(section: SlotSection, index: number, lines: DiffLineDto[], view: ViewModel, reference: BaselineDto): ChangeRow {
  const table = SLOT_TABLE[section];
  const jump: ChangeJump = { kind: "slot", panel: table.panel, index };
  let text = `${table.title} ${index}`;
  let detail: string;
  if (section === "combo" || section === "macro") {
    const name = section === "combo" ? view.names.combos[index] : view.names.macros[index];
    if (name) text += ` · ${name}`;
    const { curSet, baseSet, curText, baseText } = slotState(section, index, view, reference);
    if (curSet && !baseSet) detail = `New: ${curText}`;
    else if (!curSet && baseSet) detail = `Removed: ${baseText}`;
    else if (!curSet && !baseSet) detail = "Changed";
    else detail = curText === baseText ? "Changed" : `${curText} · was ${baseText}`;
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
