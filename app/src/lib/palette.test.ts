import { describe, expect, it } from "vitest";
import {
  COMMANDS,
  commandReason,
  filterCommands,
  parseLayerArg,
  updateShortcutLabels,
} from "./palette";

describe("filterCommands", () => {
  it("returns everything for an empty query", () => {
    expect(filterCommands("")).toEqual(COMMANDS);
    expect(filterCommands("   ")).toEqual(COMMANDS);
  });

  it("ranks prefix and word matches first", () => {
    expect(filterCommands("save")[0]?.id).toBe("save");
    expect(filterCommands("save")[1]?.id).toBe("saveAs");
    expect(filterCommands("swap")[0]?.id).toBe("swapKeys");
    expect(filterCommands("git")[0]?.id).toBe("diffGit");
    expect(filterCommands("UNDO")[0]?.id).toBe("undo");
  });

  it("matches subsequences and drops non-matches", () => {
    expect(filterCommands("mrr").map((c) => c.id)).toContain("mirrorLeftToRight");
    expect(filterCommands("xyzzy")).toEqual([]);
  });

  it("has unique ids", () => {
    const ids = COMMANDS.map((c) => c.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("shows the platform primary modifier", () => {
    updateShortcutLabels("Win32");
    expect(COMMANDS.find((command) => command.id === "save")?.shortcut).toBe("Ctrl+S");
    expect(COMMANDS.find((command) => command.id === "redo")?.shortcut).toBe("Ctrl+Shift+Z");
    updateShortcutLabels("MacIntel");
    expect(COMMANDS.find((command) => command.id === "save")?.shortcut).toBe("⌘S");
    expect(COMMANDS.find((command) => command.id === "redo")?.shortcut).toBe("⇧⌘Z");
  });
});

describe("parseLayerArg", () => {
  it("accepts layer numbers in range", () => {
    expect(parseLayerArg("0", 10)).toBe(0);
    expect(parseLayerArg(" 9 ", 10)).toBe(9);
  });

  it("rejects everything else", () => {
    expect(parseLayerArg("10", 10)).toBeNull();
    expect(parseLayerArg("-1", 10)).toBeNull();
    expect(parseLayerArg("1.5", 10)).toBeNull();
    expect(parseLayerArg("", 10)).toBeNull();
    expect(parseLayerArg("two", 10)).toBeNull();
  });
});

describe("zoom commands", () => {
  it("lists the three view commands with their shortcuts", () => {
    updateShortcutLabels("MacIntel");
    const byId = (id: string) => COMMANDS.find((command) => command.id === id);
    expect(byId("zoomIn")).toMatchObject({ title: "View: zoom in", shortcut: "⌘=" });
    expect(byId("zoomOut")).toMatchObject({ title: "View: zoom out", shortcut: "⌘−" });
    expect(byId("zoomFit")).toMatchObject({ title: "View: fit keyboard", shortcut: "⌘0" });
    expect(filterCommands("zoom").map((command) => command.id)).toEqual(
      expect.arrayContaining(["zoomIn", "zoomOut"]),
    );
  });
});

describe("update commands", () => {
  const updateIds = ["checkUpdates", "restartToUpdate", "autoUpdatesOff", "autoUpdatesOn"];

  it("list four commands with exact titles, no argument and no shortcut", () => {
    const found = COMMANDS.filter((command) => updateIds.includes(command.id));
    expect(
      found.map((command) => [command.id, command.title, command.arg, command.shortcut]),
    ).toEqual([
      ["checkUpdates", "Check for updates", "none", ""],
      ["restartToUpdate", "Restart to update", "none", ""],
      ["autoUpdatesOff", "Updates: turn off automatic checks", "none", ""],
      ["autoUpdatesOn", "Updates: turn on automatic checks", "none", ""],
    ]);
  });

  it("do not blame a missing layout", () => {
    for (const id of updateIds) {
      expect(commandReason(id, null, false)).toBe("");
    }
    expect(commandReason("save", null, false)).toBe("Open a layout first");
  });
});
