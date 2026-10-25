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

/** Short names for modifier masks, shown next to the wire names. */
export const MOD_LABELS: Record<string, string> = {
  LCTL: "L Ctrl",
  LSFT: "L Shift",
  LALT: "L Alt",
  LGUI: "L Gui",
  RCTL: "R Ctrl",
  RSFT: "R Shift",
  RALT: "R Alt",
  RGUI: "R Gui",
};

export const KEY_OVERRIDE_OPTION_LABELS: Record<string, string> = {
  "activation: trigger down": "Activate on key press",
  "activation: required mod down": "Activate on modifier press",
  "activation: negative mod up": "Activate on excluded release",
  "one mod": "Any one modifier",
  "no reregister trigger": "Don't resend the key",
  "no unregister on other key down": "Stay on for other keys",
  enabled: "Enabled",
};

export const KEY_OVERRIDE_OPTION_TITLES: Record<string, string> = {
  "activation: trigger down": "Turns on when the trigger key is pressed while the modifiers are held",
  "activation: required mod down": "Turns on when a required modifier is pressed while the trigger is held",
  "activation: negative mod up": "Turns on when an excluded modifier is released",
  "one mod": "One of the trigger modifiers is enough, instead of all of them",
  "no reregister trigger": "When the override ends, don't press the trigger key again",
  "no unregister on other key down": "Pressing another key doesn't end the override",
  enabled: "Turn this override on or off",
};

export const ALT_REPEAT_OPTION_LABELS: Record<string, string> = {
  "default to this alt key": "Use as default alternate",
  bidirectional: "Works both ways",
  "ignore mod handedness": "Ignore left/right",
  enabled: "Enabled",
};

export const ALT_REPEAT_OPTION_TITLES: Record<string, string> = {
  "default to this alt key": "Used when no other alt repeat entry matches the last key",
  bidirectional: "Alt Repeat after the alternate key sends the original key",
  "ignore mod handedness": "Treat left and right modifiers as the same",
  enabled: "Turn this entry on or off",
};

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
