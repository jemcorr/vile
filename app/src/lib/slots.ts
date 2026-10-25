// Feature slot summaries for the slot grids, the changes sheet and the Revert labels.
import type { AltRepeatDto, ComboDto, KeyOverrideDto, MacroActionDto, MacroDto, TapDanceDto, ViewModel } from "./types";
import { MOD_BITS, hasBit } from "./bits";

/** The keycap label at a keycode path ("combo[0][1]"), or undefined when there is none. */
export type LabelOf = (path: string) => string | undefined;

export function currentLabels(view: ViewModel | null): LabelOf {
  return (path) => view?.keycodes[path]?.label;
}

/** Labels at the comparison point: the baseline value where it differs, else the current one. */
export function baselineLabels(view: ViewModel | null): LabelOf {
  return (path) => view?.baseline?.keycodes[path]?.label ?? view?.keycodes[path]?.label;
}

/** Keycap labels hold "\n" between lines. Text and tooltips outside a keycap use one line. */
export const flat = (text: string): string => text.split("\n").join(" ");

export function comboSet(combo: ComboDto): boolean {
  return combo.keys.some((key) => key !== "KC_NO") || combo.output !== "KC_NO";
}

/** "triggers → output", or the combo name when it has one. Empty for an empty slot. */
export function comboText(combo: ComboDto, label: LabelOf, name?: string): string {
  if (!comboSet(combo)) return "";
  if (name) return name;
  const triggers = combo.keys
    .map((key, i) => (key === "KC_NO" ? "" : label(`combo[${combo.index}][${i}]`) || key))
    .filter(Boolean)
    .join(" + ");
  const output = combo.output === "KC_NO" ? "" : combo.outputLabel || combo.output;
  return (triggers || "…") + " → " + (output || "…");
}

export function macroSet(macro: MacroDto): boolean {
  return macro.actions.length > 0;
}

/** The first four actions joined with spaces, or the macro name when it has one. */
export function macroText(macro: MacroDto, label: LabelOf, name?: string): string {
  if (!macroSet(macro)) return "";
  if (name) return name;
  return macroParts(macro, label).join(" ");
}

export function macroParts(macro: MacroDto, label: LabelOf): string[] {
  if (!macroSet(macro)) return [];
  return macro.actions.slice(0, 4).flatMap((action, i) => macroPart(macro.index, action, i, label));
}

function macroPart(index: number, action: MacroActionDto, i: number, label: LabelOf): string[] {
  if (action.kind === "text") return ['"' + action.text + '"'];
  if (action.kind === "delay") return [action.delay + " ms"];
  const prefix = action.kind === "down" ? "↓" : action.kind === "up" ? "↑" : "";
  return action.keys
    .map((key, k) => (key === "KC_NO" ? "" : label(`macro[${index}][${i}][${k + 1}]`) || key))
    .filter(Boolean)
    .map((text) => prefix + text);
}

export function tapDanceSet(td: TapDanceDto): boolean {
  return [td.onTap, td.onHold, td.onDoubleTap, td.onTapHold].some((key) => key !== "KC_NO");
}

/** "Tap x · Hold y" for the four actions that are set. */
export function tapDanceText(td: TapDanceDto, label: LabelOf): string {
  const keys = [td.onTap, td.onHold, td.onDoubleTap, td.onTapHold];
  const names = ["Tap", "Hold", "Double tap", "Tap and hold"];
  return keys
    .map((key, i) => (key === "KC_NO" ? "" : names[i] + " " + (label(`tap_dance[${td.index}][${i}]`) || key)))
    .filter(Boolean)
    .join(" · ");
}

export function keyOverrideSet(ko: KeyOverrideDto): boolean {
  return ko.trigger !== "KC_NO" || ko.replacement !== "KC_NO";
}

/** "Mods+trigger → replacement". Trigger modifiers use their wire names. */
export function keyOverrideText(ko: KeyOverrideDto, label: LabelOf): string {
  if (!keyOverrideSet(ko)) return "";
  const mods = MOD_BITS.filter((bit) => hasBit(ko.triggerMods, bit.bit))
    .map((bit) => bit.name)
    .join("+");
  const trigger = ko.trigger === "KC_NO" ? "" : label(`key_override[${ko.index}].trigger`) || ko.trigger;
  const replacement = ko.replacement === "KC_NO" ? "" : label(`key_override[${ko.index}].replacement`) || ko.replacement;
  const left = [mods, trigger].filter(Boolean).join("+");
  return (left || "…") + " → " + (replacement || "…");
}

export function altRepeatSet(ar: AltRepeatDto): boolean {
  return ar.keycode !== "KC_NO" || ar.altKeycode !== "KC_NO" || ar.allowedMods !== 0 || ar.options !== 0;
}

/** "key → alternate". */
export function altRepeatText(ar: AltRepeatDto, label: LabelOf): string {
  if (!altRepeatSet(ar)) return "";
  const key = ar.keycode === "KC_NO" ? "" : label(`alt_repeat_key[${ar.index}].keycode`) || ar.keycode;
  const alt = ar.altKeycode === "KC_NO" ? "" : label(`alt_repeat_key[${ar.index}].alt_keycode`) || ar.altKeycode;
  return (key || "…") + " → " + (alt || "…");
}
