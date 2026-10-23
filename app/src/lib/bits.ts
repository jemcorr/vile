export interface BitName {
  bit: number;
  name: string;
}

/** 8-bit modifier masks used by key overrides and alt repeat keys. */
export const MOD_BITS: BitName[] = [
  { bit: 0, name: "LCTL" },
  { bit: 1, name: "LSFT" },
  { bit: 2, name: "LALT" },
  { bit: 3, name: "LGUI" },
  { bit: 4, name: "RCTL" },
  { bit: 5, name: "RSFT" },
  { bit: 6, name: "RALT" },
  { bit: 7, name: "RGUI" },
];

export const KEY_OVERRIDE_OPTION_BITS: BitName[] = [
  { bit: 0, name: "activation: trigger down" },
  { bit: 1, name: "activation: required mod down" },
  { bit: 2, name: "activation: negative mod up" },
  { bit: 3, name: "one mod" },
  { bit: 4, name: "no reregister trigger" },
  { bit: 5, name: "no unregister on other key down" },
  { bit: 7, name: "enabled" },
];

export const ALT_REPEAT_OPTION_BITS: BitName[] = [
  { bit: 0, name: "default to this alt key" },
  { bit: 1, name: "bidirectional" },
  { bit: 2, name: "ignore mod handedness" },
  { bit: 3, name: "enabled" },
];

export function hasBit(value: number, bit: number): boolean {
  return (value & (1 << bit)) !== 0;
}

export function toggleBit(value: number, bit: number): number {
  return value ^ (1 << bit);
}

/** The names of the set modifier bits joined with "+", or "none". */
export function modNames(mask: number): string {
  const names = MOD_BITS.filter((m) => hasBit(mask, m.bit)).map((m) => m.name);
  return names.length > 0 ? names.join("+") : "none";
}
