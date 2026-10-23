import type { BoardKeyDto, ComboDto, KeyCellDto } from "./types";

export interface ComboOverlay {
  index: number;
  keyIds: string[];
  label: string;
  cx: number; // key units
  cy: number;
}

const mean = (xs: number[]): number => xs.reduce((a, b) => a + b, 0) / xs.length;

/** Where each combo is drawn on the board: the centre of its input keys, labelled with its output. */
export function comboOverlays(combos: ComboDto[], cells: KeyCellDto[], keys: BoardKeyDto[]): ComboOverlay[] {
  const out: ComboOverlay[] = [];
  for (const combo of combos) {
    const inputs = combo.keys.filter((k) => k !== "KC_NO");
    if (inputs.length < 2) continue;
    const found: BoardKeyDto[] = [];
    let complete = true;
    for (const input of inputs) {
      const i = keys.findIndex((_, idx) => cells[idx]?.wire === input);
      const key = i >= 0 ? keys[i] : undefined;
      if (!key) {
        complete = false;
        break;
      }
      found.push(key);
    }
    if (!complete) continue;
    out.push({
      index: combo.index,
      keyIds: found.map((k) => k.id),
      label: combo.outputLabel,
      cx: mean(found.map((k) => k.x + k.w / 2)),
      cy: mean(found.map((k) => k.y + k.h / 2)),
    });
  }
  return out;
}
