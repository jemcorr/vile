import { describe, expect, it } from "vitest";
import { COMMANDS, filterCommands, parseLayerArg } from "./palette";

describe("filterCommands", () => {
  it("returns everything for an empty query", () => {
    expect(filterCommands("")).toEqual(COMMANDS);
    expect(filterCommands("   ")).toEqual(COMMANDS);
  });

  it("ranks prefix and word matches first", () => {
    expect(filterCommands("save")[0]?.id).toBe("save");
    expect(filterCommands("save")[1]?.id).toBe("saveAs");
    expect(filterCommands("swap")[0]?.id).toBe("swapLayer");
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
