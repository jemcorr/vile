// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { bounds, rotate } from "./interactions";
import { planLayerSheet, renderLayerSheet, type ExportTheme } from "./layer-export";
import type { BoardKeyDto, KeyCellDto, KeycodeView } from "./types";

const keys: BoardKeyDto[] = [
  { id: "a", row: 0, col: 0, x: -1, y: 0, w: 1, h: 1 },
  {
    id: "b",
    row: 0,
    col: 1,
    x: 1,
    y: 1,
    w: 1.5,
    h: 1,
    r: 35,
    rx: 1,
    ry: 0,
    secondary: { x: -0.5, y: 0.5, w: 1, h: 1 },
  },
];
const cell = (id: string, code: number, label = "A"): KeyCellDto => ({
  id,
  code,
  label,
  kind: code === 0 ? "no" : code === 1 ? "trns" : "basic",
  wire: "KC_A",
  description: "A",
  tooltip: "Send A to the host.",
  path: "",
  previous: "KC_NO",
  changed: true,
});
type ExportView = Parameters<typeof planLayerSheet>[0];
function view(layers = [[cell("a", 4), cell("b", 5)]]): ExportView {
  return {
    board: { id: "test", name: "Test board", keys },
    layers,
    names: { layers: { "0": "Base", "3": "Symbols" }, macros: {}, combos: {} },
    keycodes: {},
    layerCount: layers.length,
    combos: [],
    macros: [],
    tapDances: [],
    keyOverrides: [],
    altRepeatKeys: [],
  };
}
function keycodeView(code: number, wire: string, label: string): KeycodeView {
  return { code, wire, label, description: label, tooltip: "", category: "Basic", parts: { kind: "raw", code } };
}
/** The corne fixture: combos 0-3 and macros 0, 2 and 3 are set; every other feature is empty. */
function fixtureFeatures(model: ExportView): ExportView {
  const combo = (index: number) => ({
    index,
    keys: ["KC_A", "KC_S", "KC_NO", "KC_NO"],
    output: "KC_ESC",
    outputLabel: "Esc",
  });
  const macro = (index: number) => ({
    index,
    actions: [
      { kind: "tap" as const, keys: ["KC_H"], text: "", delay: 0 },
      { kind: "text" as const, keys: [], text: "hi", delay: 0 },
      { kind: "delay" as const, keys: [], text: "", delay: 120 },
    ],
  });
  return {
    ...model,
    combos: [0, 1, 2, 3].map(combo),
    macros: [0, 2, 3].map(macro),
    keycodes: {
      "combo[0][0]": keycodeView(4, "KC_A", "A"),
      "combo[0][4]": keycodeView(41, "KC_ESC", "Esc"),
      "macro[0][0][1]": keycodeView(11, "KC_H", "H"),
    },
  };
}
const measure = (text: string) => text.length * 12;
const theme: ExportTheme = {
  background: "#1c1c1e",
  foreground: "#f2f2f7",
  muted: "#8e8e93",
  key: "#2c2c2e",
  keyForeground: "#f2f2f7",
  border: "#48484a",
  layer: "#233457",
  modifier: "#4a3b1f",
  macro: "#3b2d57",
  media: "#1f4440",
  font: "sans-serif",
};

afterEach(() => vi.restoreAllMocks());

describe("layer PNG composition", () => {
  it("skips blank, transparent and mixed blank/transparent layers while retaining original indexes", () => {
    const model = view([
      [cell("a", 4), cell("b", 0)],
      [cell("a", 0), cell("b", 0)],
      [cell("a", 1), cell("b", 0)],
      [cell("a", 1), cell("b", 5)],
      [cell("a", 1), cell("b", 1)],
    ]);
    const plan = planLayerSheet(model, measure);
    expect(plan.tiles.map((tile) => [tile.index, tile.title])).toEqual([
      [0, "Layer 0 · Base"],
      [3, "Layer 3 · Symbols"],
    ]);
    expect(planLayerSheet(model, measure)).toEqual(plan);
    expect(() => planLayerSheet(view([[cell("a", 0), cell("b", 1)]]), measure)).toThrow(
      "Every layer contains only blank or transparent keys",
    );
  });

  it("lays out every tile without overlap and contains rotated and secondary geometry", () => {
    const model = view(Array.from({ length: 9 }, () => [cell("a", 4), cell("b", 5)]));
    model.names.layers[0] = "A".repeat(100);
    const plan = planLayerSheet(model, measure);
    expect(plan.tiles).toHaveLength(9);
    expect(plan.tiles[0]!.lines.join("")).toBe("Layer 0 · " + "A".repeat(100));
    const geometry = bounds(keys);
    for (const [index, tile] of plan.tiles.entries()) {
      expect(tile.x).toBeGreaterThan(0);
      expect((tile.x + plan.tileWidth) * 2).toBeLessThan(plan.width);
      expect((tile.y + plan.tileHeight) * 2).toBeLessThan(plan.height);
      for (const other of plan.tiles.slice(index + 1))
        expect(
          tile.x + plan.tileWidth <= other.x ||
            other.x + plan.tileWidth <= tile.x ||
            tile.y + plan.tileHeight <= other.y ||
            other.y + plan.tileHeight <= tile.y,
        ).toBe(true);
      for (const key of keys) {
        for (const rect of [
          { x: 0, y: 0, w: key.w, h: key.h },
          ...(key.secondary ? [key.secondary] : []),
        ]) {
          for (const [dx, dy] of [
            [0, 0],
            [rect.w, 0],
            [0, rect.h],
            [rect.w, rect.h],
          ]) {
            const p = rotate(key, { x: key.x + rect.x + dx!, y: key.y + rect.y + dy! });
            expect(p.x).toBeGreaterThanOrEqual(geometry.x - 1e-10);
            expect(p.x).toBeLessThanOrEqual(geometry.x + geometry.w + 1e-10);
            expect(p.y).toBeGreaterThanOrEqual(geometry.y - 1e-10);
            expect(p.y).toBeLessThanOrEqual(geometry.y + geometry.h + 1e-10);
          }
        }
      }
    }
  });

  it("rejects image sizes before allocating a bitmap", () => {
    const model = view();
    model.board = { ...model.board!, keys: [{ ...keys[0]!, w: 1000 }] };
    expect(() => planLayerSheet(model, measure)).toThrow("too large to export");
  });
});

function canvasMock(failEncoding = false) {
  const fills: string[] = [];
  const text: { text: string; font: string; color: string }[] = [];
  const context = {
    fillStyle: "",
    font: "",
    strokeStyle: "",
    lineWidth: 1,
    textAlign: "",
    textBaseline: "",
    save: vi.fn(),
    restore: vi.fn(),
    translate: vi.fn(),
    rotate: vi.fn(),
    scale: vi.fn(),
    beginPath: vi.fn(),
    roundRect: vi.fn(),
    stroke: vi.fn(),
    fillRect: vi.fn(),
    fill() {
      fills.push(this.fillStyle);
    },
    fillText(value: string) {
      text.push({ text: value, font: this.font, color: this.fillStyle });
    },
    measureText: (value: string) => ({ width: measure(value) }),
  };
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue(
    context as unknown as CanvasRenderingContext2D,
  );
  const bytes = new Uint8Array([137, 80, 78, 71]);
  vi.spyOn(HTMLCanvasElement.prototype, "toBlob").mockImplementation((callback, type) => {
    expect(type).toBe("image/png");
    callback(failEncoding ? null : ({ arrayBuffer: async () => bytes.buffer } as Blob));
  });
  return { context, fills, text, bytes };
}

describe("layer PNG rendering", () => {
  it("renders all layers by key ID, with hold labels, geometry and theme colors but no edit markers", async () => {
    const mock = canvasMock();
    const model = view([
      [
        { ...cell("b", 5, "Mouse"), kind: "mouse" },
        { ...cell("a", 4, "S\nCtrl"), kind: "modTap" },
      ],
      [cell("a", 0, ""), cell("b", 1, "▽")],
      [cell("a", 4, "X"), cell("b", 5, "Y")],
    ]);
    const snapshot = JSON.stringify(model);
    const result = await renderLayerSheet(model, theme);
    expect(result).toEqual({ png: mock.bytes, layerCount: 2 });
    expect(mock.fills).toEqual([
      theme.modifier,
      theme.media,
      theme.media,
      theme.key,
      theme.key,
      theme.key,
    ]);
    expect(mock.text.map((entry) => entry.text)).toEqual([
      "Layer 0 · Base",
      "Ctrl",
      "S",
      "Mouse",
      "Layer 2",
      "X",
      "Y",
    ]);
    expect(mock.text.find((entry) => entry.text === "Ctrl")).toEqual({
      text: "Ctrl",
      font: "10px sans-serif",
      color: theme.muted,
    });
    expect(mock.text.find((entry) => entry.text === "Mouse")?.color).toBe(theme.keyForeground);
    expect(mock.context.rotate).toHaveBeenCalledWith((35 * Math.PI) / 180);
    expect(mock.context.roundRect).toHaveBeenCalledTimes(8); // Two tiles and three key rectangles per tile.
    expect(JSON.stringify(model)).toBe(snapshot);
  });

  it("paints compact two-line mouse labels without copying hover paragraphs into the image", async () => {
    const mock = canvasMock();
    const speed = {
      ...cell("a", 0xdd, "Speed\n0"),
      kind: "mouse" as const,
      tooltip: "Detailed speed behavior that belongs only in hover text.",
    };
    const click = { ...cell("b", 0xd1, "Left\nclick"), kind: "mouse" as const };
    await renderLayerSheet(view([[speed, click]]), theme);
    expect(mock.text.map((entry) => entry.text)).toEqual([
      "Layer 0 · Base",
      "Speed",
      "0",
      "Left",
      "click",
    ]);
    expect(
      mock.text.every(
        (entry) => entry.color === theme.keyForeground || entry.text.startsWith("Layer"),
      ),
    ).toBe(true);
  });

  it("reports encoding and canvas failures", async () => {
    canvasMock(true);
    await expect(renderLayerSheet(view(), theme)).rejects.toThrow("could not encode");
    vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue(null);
    await expect(renderLayerSheet(view(), theme)).rejects.toThrow("cannot render");
  });
});

describe("feature sections in the PNG", () => {
  it("adds Combos and Macros sections with exactly the set entries and no other feature sections", () => {
    const model = fixtureFeatures(view());
    const plan = planLayerSheet(model, measure);
    expect(plan.sections.map((section) => section.feature)).toEqual(["combo", "macro"]);
    expect(plan.sections.map((section) => section.heading)).toEqual(["Combos", "Macros"]);
    expect(plan.sections[0]!.entries.map((entry) => entry.index)).toEqual([0, 1, 2, 3]);
    expect(plan.sections[1]!.entries.map((entry) => entry.index)).toEqual([0, 2, 3]);
    expect(plan.sections[0]!.entries[0]!.title).toEqual(["Combo 0"]);
  });

  it("keeps the layer grid size and position identical with or without feature sections", () => {
    const base = view([[cell("a", 4), cell("b", 5)], [cell("a", 4), cell("b", 0)]]);
    const withoutFeatures = planLayerSheet(base, measure);
    const withFeatures = planLayerSheet(fixtureFeatures(base), measure);
    expect(withFeatures.tiles).toEqual(withoutFeatures.tiles);
    expect(withFeatures.width).toBe(withoutFeatures.width);
    expect(withFeatures.tileHeight).toBe(withoutFeatures.tileHeight);
    expect(withFeatures.height).toBeGreaterThan(withoutFeatures.height);
    const layerBottom = Math.max(...withFeatures.tiles.map((tile) => tile.y + withFeatures.tileHeight));
    for (const section of withFeatures.sections) expect(section.y).toBeGreaterThanOrEqual(layerBottom);
    expect(withFeatures.sections[0]!.y).toBe(withoutFeatures.height / 2);
  });

  it("skips features with no set entries and does not change the plan for an empty model", () => {
    const plan = planLayerSheet(view(), measure);
    expect(plan.sections).toEqual([]);
    const withTapDance = { ...view(), tapDances: [{ index: 0, onTap: "KC_NO", onHold: "KC_NO", onDoubleTap: "KC_NO", onTapHold: "KC_NO", tappingTerm: 200 }] };
    expect(planLayerSheet(withTapDance, measure).sections).toEqual([]);
  });

  it("renders the feature text in dark and light themes without changing the layer tiles", async () => {
    const mock = canvasMock();
    const model = fixtureFeatures(view());
    const dark = await renderLayerSheet(model, theme);
    const darkText = mock.text.map((entry) => entry.text);
    expect(darkText).toEqual(expect.arrayContaining(["Combos", "Macros", "Combo 0", "A", "Esc", "Macro 3", '"hi"', "120 ms"]));
    expect(darkText).not.toEqual(expect.arrayContaining(["Tap dances", "Key overrides", "Alt repeat"]));
    expect(mock.fills).toContain(theme.key);
    const light: ExportTheme = { ...theme, background: "#ffffff", foreground: "#1c1c1e", key: "#f2f2f7", keyForeground: "#1c1c1e" };
    mock.text.length = 0;
    const lightResult = await renderLayerSheet(model, light);
    expect(lightResult.layerCount).toBe(dark.layerCount);
    expect(mock.text.map((entry) => entry.text)).toEqual(darkText);
    expect(mock.text.find((entry) => entry.text === "Esc")?.color).toBe(light.keyForeground);
  });
});
