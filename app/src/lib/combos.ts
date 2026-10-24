import type { BoardKeyDto, ComboDto, KeyCellDto } from "./types";
import { rotate } from "./interactions";

export interface ComboOverlay {
  index: number;
  keyIds: string[];
  label: string;
  cx: number; // key units
  cy: number;
}

const mean = (xs: number[]): number => xs.reduce((a, b) => a + b, 0) / xs.length;

/** Where each combo is drawn on the board: the centre of its input keys, labelled with its output. */
export function comboOverlays(
  combos: ComboDto[],
  cells: KeyCellDto[],
  keys: BoardKeyDto[],
): ComboOverlay[] {
  const out: ComboOverlay[] = [];
  for (const combo of combos) {
    const inputs = combo.keys.filter((k) => k !== "KC_NO");
    if (inputs.length < 2) continue;
    const found: BoardKeyDto[] = [];
    let complete = true;
    for (const input of inputs) {
      const matches = keys.filter((_, idx) => cells[idx]?.wire === input);
      if (!matches.length) {
        complete = false;
        break;
      }
      found.push(...matches);
    }
    if (!complete) continue;
    out.push({
      index: combo.index,
      keyIds: found.map((k) => k.id),
      label: combo.outputLabel,
      cx: mean(found.map((k) => rotate(k, { x: k.x + k.w / 2, y: k.y + k.h / 2 }).x)),
      cy: mean(found.map((k) => rotate(k, { x: k.x + k.w / 2, y: k.y + k.h / 2 }).y)),
    });
  }
  return out;
}
