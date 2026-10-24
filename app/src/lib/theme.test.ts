import { describe, expect, it } from "vitest";
import {
  effectiveTheme,
  nextThemeMode,
  parseThemeMode,
  readThemeMode,
  THEME_STORAGE_KEY,
  writeThemeMode,
} from "./theme";

describe("theme mode", () => {
  it("cycles system, light, dark, then system", () => {
    expect(nextThemeMode("system")).toBe("light");
    expect(nextThemeMode("light")).toBe("dark");
    expect(nextThemeMode("dark")).toBe("system");
  });

  it("defaults to system for missing, unknown or invalid stored values", () => {
    expect(parseThemeMode(null)).toBe("system");
    expect(parseThemeMode(undefined)).toBe("system");
    expect(parseThemeMode("sepia")).toBe("system");
    expect(parseThemeMode("dark")).toBe("dark");
    expect(parseThemeMode("light")).toBe("light");
  });

  it("resolves system from the color scheme and keeps explicit choices", () => {
    expect(effectiveTheme("system", true)).toBe("dark");
    expect(effectiveTheme("system", false)).toBe("light");
    expect(effectiveTheme("light", true)).toBe("light");
    expect(effectiveTheme("dark", false)).toBe("dark");
  });

  it("reads and writes the stored choice", () => {
    const store = new Map<string, string>();
    const storage = {
      getItem: (key: string) => store.get(key) ?? null,
      setItem: (key: string, value: string) => void store.set(key, value),
    };
    expect(readThemeMode(storage)).toBe("system");
    writeThemeMode("dark", storage);
    expect(store.get(THEME_STORAGE_KEY)).toBe("dark");
    expect(readThemeMode(storage)).toBe("dark");
  });

  it("falls back to system and does not throw when storage throws", () => {
    const broken = {
      getItem: () => {
        throw new Error("denied");
      },
      setItem: () => {
        throw new Error("denied");
      },
    };
    expect(readThemeMode(broken)).toBe("system");
    expect(() => writeThemeMode("light", broken)).not.toThrow();
    expect(readThemeMode(null)).toBe("system");
  });
});
