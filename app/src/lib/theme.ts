// Theme preference: follow the system, or force light or dark.

export type ThemeMode = "system" | "light" | "dark";
export type EffectiveTheme = "light" | "dark";

export const THEME_STORAGE_KEY = "vile.theme";

const MODES: readonly ThemeMode[] = ["system", "light", "dark"];

/** System -> Light -> Dark -> System. */
export function nextThemeMode(mode: ThemeMode): ThemeMode {
  return MODES[(MODES.indexOf(mode) + 1) % MODES.length] ?? "system";
}

/** Resolves a stored value. Anything unknown, or a missing value, means system. */
export function parseThemeMode(value: string | null | undefined): ThemeMode {
  return value === "light" || value === "dark" ? value : "system";
}

export function effectiveTheme(mode: ThemeMode, prefersDark: boolean): EffectiveTheme {
  if (mode === "system") return prefersDark ? "dark" : "light";
  return mode;
}

/** Reads the stored mode. Blocked or throwing storage means system. */
export function readThemeMode(storage: Pick<Storage, "getItem"> | null = safeStorage()): ThemeMode {
  try {
    return parseThemeMode(storage?.getItem(THEME_STORAGE_KEY) ?? null);
  } catch {
    return "system";
  }
}

/** Saves the mode. Storage that throws is ignored; the choice then lasts for this session only. */
export function writeThemeMode(
  mode: ThemeMode,
  storage: Pick<Storage, "setItem"> | null = safeStorage(),
): void {
  try {
    storage?.setItem(THEME_STORAGE_KEY, mode);
  } catch {
    /* Preference storage is optional. */
  }
}

function safeStorage(): Storage | null {
  try {
    return typeof localStorage === "undefined" ? null : localStorage;
  } catch {
    return null;
  }
}
