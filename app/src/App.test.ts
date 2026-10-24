// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { mount, unmount, tick } from "svelte";
import App from "./App.svelte";
import { ui } from "./lib/state.svelte";
import type { ViewModel, KeycodeView, SearchHitDto } from "./lib/types";
const fake = vi.hoisted(() => ({
  getView: vi.fn(),
  applyOp: vi.fn(),
  save: vi.fn(),
  saveAsDialog: vi.fn(),
  exportLayers: vi.fn(),
  openDialog: vi.fn(),
  undo: vi.fn(),
  redo: vi.fn(),
  searchKeycodes: vi.fn(),
  resolveKeycode: vi.fn(),
  keycodeView: vi.fn(),
  composeKeycode: vi.fn(),
  macroUsage: vi.fn(),
  refreshDefinition: vi.fn(),
  smokeReport: vi.fn(),
  closeWindow: vi.fn(),
  setDiffBase: vi.fn(),
}));
const renderSheet = vi.hoisted(() => vi.fn());
vi.mock("./lib/layer-export", () => ({ renderLayerSheet: renderSheet }));
vi.mock("./lib/api", () => ({ api: fake, errorText: (e: unknown) => String(e) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: () => Promise.resolve(() => {}) }));
let app: ReturnType<typeof mount> | undefined;
async function settle() {
  for (let i = 0; i < 5; i++) {
    await Promise.resolve();
    await tick();
  }
}
const key: KeycodeView = {
  code: 4,
  wire: "KC_A",
  label: "A",
  description: "A",
  tooltip: "Send A to the host.",
  category: "Basic",
  parts: { kind: "tap", base: 4, wrapMods: 0, hold: { kind: "none" } },
};
function view(): ViewModel {
  return {
    revision: 1,
    path: "/test/layout.vil",
    uid: "9007199254740993",
    dirty: true,
    canUndo: true,
    canRedo: false,
    layerCount: 1,
    board: {
      id: "matrix",
      name: "Matrix grid",
      canMirror: false,
      encoderIndices: null,
      keys: [
        { id: "r0c0", row: 0, col: 0, x: 0, y: 0, w: 1, h: 1 },
        { id: "r0c1", row: 0, col: 1, x: 1, y: 0, w: 1, h: 1 },
      ],
    },
    layers: [
      [
        {
          id: "r0c0",
          wire: key.wire,
          label: key.label,
          kind: "basic",
          changed: false,
          path: "layout[0][0][0]",
          code: 4,
          description: "A",
          tooltip: "Send A to the host.",
          previous: "KC_A",
        },
        {
          id: "r0c1",
          wire: key.wire,
          label: key.label,
          kind: "basic",
          changed: false,
          path: "layout[0][0][1]",
          code: 4,
          description: "A",
          tooltip: "Send A to the host.",
          previous: "KC_A",
        },
      ],
    ],
    diff: [],
    diffBase: "disk",
    warnings: [],
    combos: [
      {
        index: 0,
        keys: ["KC_NO", "KC_NO", "KC_NO", "KC_NO"],
        output: "KC_ESCAPE",
        outputLabel: "Esc",
      },
    ],
    macros: [],
    tapDances: [],
    keyOverrides: [],
    altRepeatKeys: [],
    names: { layers: {}, macros: {}, combos: {} },
    keycodes: {
      "layout[0][0][0]": key,
      "layout[0][0][1]": key,
      "combo[0][0]": { ...key, code: 0, wire: "KC_NO" },
      "combo[0][1]": { ...key, code: 0, wire: "KC_NO" },
      "combo[0][2]": { ...key, code: 0, wire: "KC_NO" },
      "combo[0][3]": { ...key, code: 0, wire: "KC_NO" },
      "combo[0][4]": { ...key, code: 41, wire: "KC_ESCAPE" },
    },
    pendingDiff: [],
    macroBytes: 0,
    macroCapacity: null,
    geometrySource: "matrix",
  };
}
function button(text: string): HTMLButtonElement {
  const found = [...document.querySelectorAll("button")].find((b) =>
    b.textContent?.trim().startsWith(text),
  );
  if (!found) throw new Error(`Button missing: ${text}`);
  return found;
}
function pointer(target: EventTarget, type: string, x = 10, y = 10, id = 1): void {
  const event = new MouseEvent(type, { bubbles: true, button: 0, clientX: x, clientY: y });
  Object.defineProperty(event, "pointerId", { value: id });
  target.dispatchEvent(event);
}
beforeEach(() => {
  vi.clearAllMocks();
  ui.view = null;
  ui.layer = 0;
  ui.cursor = "";
  ui.selection = [];
  ui.inspectorPaths = [];
  ui.mode = "normal";
  ui.drafts = {};
  ui.panel = "diff";
  ui.comboEditing = null;
  ui.baseTarget = false;
  ui.sideVisible = true;
  ui.draftMacro = null;
  ui.revealMacro = null;
  ui.swapMark = null;
  fake.getView.mockResolvedValue(view());
  fake.searchKeycodes.mockResolvedValue([
    {
      name: "KC_A",
      insert: "KC_A",
      label: "A",
      pickerLabel: "A",
      summary: "Letter",
      pickerGroup: "Letters",
      complete: true,
      category: "Basic",
      description: "A",
      code: 4,
      group: "basic",
      matched: "KC_A",
    } satisfies SearchHitDto,
  ]);
  fake.keycodeView.mockResolvedValue(key);
  fake.composeKeycode.mockResolvedValue(key);
  fake.macroUsage.mockResolvedValue({ bytes: 0, actionBytes: [] });
  fake.refreshDefinition.mockRejectedValue("No device");
  fake.smokeReport.mockResolvedValue(undefined);
  fake.save.mockResolvedValue({ ...view(), dirty: false, revision: 2 });
  fake.applyOp.mockResolvedValue({ ...view(), revision: 2 });
  renderSheet.mockResolvedValue({ png: new Uint8Array([137, 80, 78, 71]), layerCount: 3 });
  fake.exportLayers.mockResolvedValue("/test/layout-layers.png");
});
afterEach(async () => {
  if (app) await unmount(app);
  app = undefined;
  document.body.innerHTML = "";
});
describe("desktop editing workflow", () => {
  it("shows only definition-confirmed knob bindings and keeps unknown slots accessible", async () => {
    const known = view();
    known.board = { ...known.board!, encoderIndices: [1] };
    for (const encoder of [0, 1]) {
      for (const direction of [0, 1]) {
        known.keycodes[`encoder_layout[0][${encoder}][${direction}]`] = key;
      }
    }
    fake.getView.mockResolvedValue(known);
    app = mount(App, { target: document.body });
    await settle();
    expect(document.body.textContent).toContain("Knob 2 clockwise");
    expect(document.body.textContent).toContain("Knob 2 counterclockwise");
    expect(document.body.textContent).not.toContain("Knob 1 clockwise");
    await unmount(app);
    app = undefined;

    const unknown = view();
    for (const encoder of [0, 1]) {
      for (const direction of [0, 1]) {
        unknown.keycodes[`encoder_layout[0][${encoder}][${direction}]`] = key;
      }
    }
    fake.getView.mockResolvedValue(unknown);
    app = mount(App, { target: document.body });
    await settle();
    expect(document.body.textContent).toContain("Knob 1 clockwise");
    expect(document.body.textContent).toContain("not confirmed as installed knobs");
  });

  it("hides firmware encoder bindings when a known definition selects no knobs", async () => {
    const noKnobs = view();
    noKnobs.board = { ...noKnobs.board!, encoderIndices: [] };
    noKnobs.keycodes["encoder_layout[0][0][0]"] = key;
    noKnobs.keycodes["encoder_layout[0][0][1]"] = key;
    fake.getView.mockResolvedValue(noKnobs);
    app = mount(App, { target: document.body });
    await settle();
    expect(document.body.textContent).not.toContain("Knobs");
  });

  it("exports the complete view without changing the current layer, selection or document", async () => {
    app = mount(App, { target: document.body });
    await settle();
    ui.selection = ["r0c0"];
    const original = ui.view;
    button("Export PNG").click();
    await settle();
    expect(renderSheet).toHaveBeenCalledWith(original);
    expect(fake.exportLayers).toHaveBeenCalledWith(new Uint8Array([137, 80, 78, 71]), 1);
    expect(ui.message).toBe("Exported 3 layers to /test/layout-layers.png");
    expect(ui.view).toBe(original);
    expect(ui.selection).toEqual(["r0c0"]);
    expect(fake.save).not.toHaveBeenCalled();
    expect(fake.applyOp).not.toHaveBeenCalled();
  });

  it("reports a canceled or failed PNG save and allows a retry", async () => {
    fake.exportLayers.mockResolvedValueOnce(null).mockRejectedValueOnce("Disk full");
    app = mount(App, { target: document.body });
    await settle();
    button("Export PNG").click();
    await settle();
    expect(ui.message).toBe("PNG export canceled");
    expect(button("Export PNG").disabled).toBe(false);
    button("Export PNG").click();
    await settle();
    expect(ui.message).toBe("Disk full");
    expect(button("Export PNG").disabled).toBe(false);
    expect(ui.view?.dirty).toBe(true);
  });

  it("reports a layer rendering failure before opening the native save dialog", async () => {
    renderSheet.mockRejectedValueOnce(new Error("No layers to export"));
    app = mount(App, { target: document.body });
    await settle();
    button("Export PNG").click();
    await settle();
    expect(ui.message).toContain("No layers to export");
    expect(fake.exportLayers).not.toHaveBeenCalled();
    expect(button("Export PNG").disabled).toBe(false);
  });

  it("makes a completed board background click clear the same targets as Escape", async () => {
    app = mount(App, { target: document.body });
    await settle();
    ui.selection = ["r0c0"];
    ui.cursor = "r0c0";
    ui.inspectorPaths = ["combo[0][4]"];
    ui.baseTarget = true;
    ui.comboEditing = 0;
    ui.swapMark = [{ layer: 0, key: "r0c0" }];
    ui.drafts = { "0": true };
    ui.draftMacro = 0;
    window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    const afterEscape = {
      selection: [...ui.selection],
      paths: [...ui.inspectorPaths],
      base: ui.baseTarget,
      combo: ui.comboEditing,
      swap: ui.swapMark,
      cursor: ui.cursor,
    };
    ui.selection = ["r0c0"];
    ui.cursor = "r0c0";
    ui.inspectorPaths = ["combo[0][4]"];
    ui.baseTarget = true;
    ui.comboEditing = 0;
    ui.swapMark = [{ layer: 0, key: "r0c0" }];
    const background = document.querySelector("main.board")!;
    pointer(background, "pointerdown");
    pointer(background, "pointerup");
    expect({
      selection: ui.selection,
      paths: ui.inspectorPaths,
      base: ui.baseTarget,
      combo: ui.comboEditing,
      swap: ui.swapMark,
      cursor: ui.cursor,
    }).toEqual(afterEscape);
    expect(ui.drafts).toEqual({ "0": true });
    expect(ui.draftMacro).toBe(0);
    expect(fake.applyOp).not.toHaveBeenCalled();
  });
  it("keeps selection through board controls, drag, and canceled background gestures", async () => {
    app = mount(App, { target: document.body });
    await settle();
    ui.selection = ["r0c0"];
    const background = document.querySelector("main.board")!;
    const control = button("Hide panel");
    pointer(control, "pointerdown");
    pointer(control, "pointerup");
    control.click();
    expect(ui.selection).toEqual(["r0c0"]);
    pointer(background, "pointerdown", 10, 10);
    pointer(background, "pointermove", 40, 10);
    pointer(background, "pointerup", 40, 10);
    expect(ui.selection).toEqual(["r0c0"]);
    pointer(background, "pointerdown");
    pointer(background, "pointerup", 10, 10, 2);
    expect(ui.selection).toEqual(["r0c0"]);
    pointer(background, "pointercancel");
    pointer(background, "pointerup");
    expect(ui.selection).toEqual(["r0c0"]);
    pointer(background, "pointerdown", 10, 10);
    pointer(background, "pointerup", 30, 10);
    expect(ui.selection).toEqual(["r0c0"]);
  });
  it("dismisses blank application areas without clearing control or field interactions", async () => {
    app = mount(App, { target: document.body });
    await settle();
    for (const selector of [
      "header .spacer",
      "nav.layers",
      "aside",
      ".picker-dock",
      ".picker",
      "footer",
    ]) {
      ui.selection = ["r0c0"];
      ui.swapMark = [{ layer: 0, key: "r0c0" }];
      ui.baseTarget = true;
      ui.comboEditing = 0;
      const area = document.querySelector(selector)!;
      pointer(area, "pointerdown");
      pointer(area, "pointerup");
      expect(ui.selection, selector).toEqual([]);
      expect(ui.swapMark, selector).toBeNull();
      expect(ui.baseTarget, selector).toBe(false);
      expect(ui.comboEditing, selector).toBeNull();
    }
    const picker = document.querySelector<HTMLElement>(".picker")!;
    Object.defineProperties(picker, {
      offsetWidth: { configurable: true, value: 100 },
      clientWidth: { configurable: true, value: 90 },
      getBoundingClientRect: {
        configurable: true,
        value: () => ({ left: 0, top: 0 }),
      },
    });
    ui.selection = ["r0c0"];
    pointer(picker, "pointerdown", 95, 10);
    pointer(picker, "pointerup", 95, 10);
    expect(ui.selection).toEqual(["r0c0"]);
    ui.selection = ["r0c0"];
    await settle();
    const value = document.querySelector(".inspector")!;
    pointer(value, "pointerdown");
    pointer(value, "pointerup");
    expect(ui.selection).toEqual(["r0c0"]);
    const control = button("Hide panel");
    pointer(control, "pointerdown");
    pointer(control, "pointerup");
    control.click();
    expect(ui.selection).toEqual(["r0c0"]);
    expect(fake.applyOp).not.toHaveBeenCalled();
    button("Save").click();
    await settle();
    const footer = document.querySelector("footer")!;
    pointer(footer, "pointerdown");
    pointer(footer, "pointerup");
    expect(document.querySelector('[aria-label="Review changes before save"]')).not.toBeNull();
  });
  it("does not swap a key when the pointer lands in its visible gutter", async () => {
    app = mount(App, { target: document.body });
    await settle();
    const svg = document.querySelector("svg.keyboard")!;
    Object.assign(svg, {
      getScreenCTM: () => ({ inverse: () => ({}) }),
      setPointerCapture: vi.fn(),
      releasePointerCapture: vi.fn(),
      hasPointerCapture: () => true,
    });
    Object.defineProperty(globalThis, "DOMPoint", {
      configurable: true,
      value: class {
        constructor(
          public x: number,
          public y: number,
        ) {}
        matrixTransform() {
          return this;
        }
      },
    });
    ui.selection = ["r0c0"];
    ui.swapMark = [{ layer: 0, key: "r0c0" }];
    pointer(svg, "pointerdown", 61, 30);
    pointer(svg, "pointerup", 61, 30);
    await settle();
    expect(ui.swapMark).toBeNull();
    expect(ui.selection).toEqual([]);
    expect(fake.applyOp).not.toHaveBeenCalled();
  });
  it("dismisses the context menu on an outside control click without canceling that control", async () => {
    app = mount(App, { target: document.body });
    await settle();
    document
      .querySelector('[data-key-id="r0c0"]')!
      .dispatchEvent(new MouseEvent("contextmenu", { bubbles: true, cancelable: true }));
    await settle();
    expect(document.querySelector('[role="menu"]')).not.toBeNull();
    button("Hide panel").click();
    await settle();
    expect(document.querySelector('[role="menu"]')).toBeNull();
    expect(button("Show panel")).toBeTruthy();
  });
  it("selects one key and Shift-click adds another", async () => {
    app = mount(App, { target: document.body });
    await settle();
    document
      .querySelector('[data-key-id="r0c0"]')!
      .dispatchEvent(new MouseEvent("click", { bubbles: true }));
    document
      .querySelector('[data-key-id="r0c1"]')!
      .dispatchEvent(new MouseEvent("click", { bubbles: true, shiftKey: true }));
    await settle();
    expect(ui.selection).toEqual(["r0c0", "r0c1"]);
    expect(document.querySelector(".inspector")?.textContent).toContain("2 keys selected");
  });
  it("requires diff review before saving", async () => {
    app = mount(App, { target: document.body });
    await settle();
    button("Save").click();
    await settle();
    expect(fake.save).not.toHaveBeenCalled();
    expect(document.querySelector('[aria-label="Review changes before save"]')).not.toBeNull();
    button("Save reviewed changes").click();
    await settle();
    expect(fake.save).toHaveBeenCalledOnce();
  });
  it("uses a base patch when the inspector base slot is active", async () => {
    app = mount(App, { target: document.body });
    await settle();
    document
      .querySelector('[data-key-id="r0c0"]')!
      .dispatchEvent(new MouseEvent("click", { bubbles: true }));
    await settle();
    document.querySelector<HTMLButtonElement>(".inspector .control button")!.click();
    document.querySelector<HTMLButtonElement>(".tiles .tile")!.click();
    await settle();
    expect(fake.applyOp).toHaveBeenCalledWith({
      type: "patchFields",
      paths: ["layout[0][0][0]"],
      patch: { base: 4 },
    });
  });
  it("edits combo triggers by code without changing its output", async () => {
    app = mount(App, { target: document.body });
    await settle();
    ui.comboEditing = 0;
    document
      .querySelector('[data-key-id="r0c1"]')!
      .dispatchEvent(new MouseEvent("click", { bubbles: true }));
    await settle();
    expect(fake.applyOp).toHaveBeenCalledWith({
      type: "setCombo",
      index: 0,
      keys: ["KC_A", "KC_NO", "KC_NO", "KC_NO"],
      output: "KC_ESCAPE",
    });
  });
  it("stops editing combo triggers when their details close", async () => {
    app = mount(App, { target: document.body });
    await settle();
    button("Combos").click();
    await settle();
    const details = document.querySelector<HTMLDetailsElement>("aside details")!;
    details.open = true;
    await settle();
    button("Choose triggers on board").click();
    expect(ui.comboEditing).toBe(0);
    details.open = false;
    details.dispatchEvent(new Event("toggle"));
    await settle();
    expect(ui.comboEditing).toBeNull();
    document
      .querySelector('[data-key-id="r0c1"]')!
      .dispatchEvent(new MouseEvent("click", { bubbles: true }));
    await settle();
    expect(ui.selection).toEqual(["r0c1"]);
    expect(fake.applyOp).not.toHaveBeenCalled();
  });
  it("lets Enter activate focused controls while SVG keys retain their own Enter action", async () => {
    app = mount(App, { target: document.body });
    await settle();
    const combos = button("Combos");
    combos.focus();
    const enter = new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true });
    combos.dispatchEvent(enter);
    expect(enter.defaultPrevented).toBe(false);
    expect(ui.mode).toBe("normal");
    combos.click(); // jsdom does not synthesize native button activation from Enter.
    await settle();
    expect(ui.panel).toBe("combos");
    const boardKey = document.querySelector<SVGGElement>('[data-key-id="r0c0"]')!;
    boardKey.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    await settle();
    expect(ui.mode).toBe("insert");
  });
  it("blocks board shortcuts in the context menu and activates its focused command", async () => {
    app = mount(App, { target: document.body });
    await settle();
    document
      .querySelector('[data-key-id="r0c0"]')!
      .dispatchEvent(new MouseEvent("contextmenu", { bubbles: true, cancelable: true }));
    await settle();
    const item = document.querySelector<HTMLButtonElement>('[role="menu"] [role="menuitem"]')!;
    expect(item.textContent).toContain("Clear selected keys");
    expect(document.activeElement).toBe(item);
    item.dispatchEvent(new KeyboardEvent("keydown", { key: "Delete", bubbles: true }));
    item.dispatchEvent(new KeyboardEvent("keydown", { key: "x", metaKey: true, bubbles: true }));
    await settle();
    expect(fake.applyOp).not.toHaveBeenCalled();
    const enter = new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true });
    item.dispatchEvent(enter);
    expect(enter.defaultPrevented).toBe(false);
    item.click(); // jsdom does not synthesize native button activation from Enter.
    await settle();
    expect(fake.applyOp).toHaveBeenCalledWith({
      type: "clearKeys",
      layer: 0,
      keys: ["r0c0"],
    });
    expect(document.querySelector('[role="menu"]')).toBeNull();
    document
      .querySelector('[data-key-id="r0c0"]')!
      .dispatchEvent(new MouseEvent("contextmenu", { bubbles: true, cancelable: true }));
    await settle();
    const first = document.querySelector<HTMLButtonElement>('[role="menu"] [role="menuitem"]')!;
    first.dispatchEvent(
      new KeyboardEvent("keydown", { key: "Tab", shiftKey: true, bubbles: true, cancelable: true }),
    );
    expect(document.activeElement?.textContent).toContain("Cancel");
    document.activeElement!.dispatchEvent(
      new KeyboardEvent("keydown", { key: "Escape", bubbles: true }),
    );
    await settle();
    expect(document.querySelector('[role="menu"]')).toBeNull();
  });
  it("does not assign to a field after changing panels or hiding the panel", async () => {
    app = mount(App, { target: document.body });
    await settle();
    button("Combos").click();
    await settle();
    const details = document.querySelector<HTMLDetailsElement>("aside details")!;
    details.open = true;
    await settle();
    button("Trigger 1").click();
    await settle();
    button("Macros").click();
    document.querySelector<HTMLButtonElement>(".tiles .tile")!.click();
    await settle();
    expect(fake.applyOp).not.toHaveBeenCalled();
    button("Combos").click();
    details.open = true;
    await settle();
    button("Trigger 1").click();
    button("Hide panel").click();
    document.querySelector<HTMLButtonElement>(".tiles .tile")!.click();
    await settle();
    expect(fake.applyOp).not.toHaveBeenCalled();
  });
  it("requires a selected combo output while collecting triggers", async () => {
    app = mount(App, { target: document.body });
    await settle();
    ui.comboEditing = 0;
    document.querySelector<HTMLButtonElement>(".tiles .tile")!.click();
    await settle();
    expect(fake.applyOp).not.toHaveBeenCalled();
    expect(ui.message).toContain("output field");
  });
  it("keeps review open after a save error and checks the refreshed revision", async () => {
    app = mount(App, { target: document.body });
    await settle();
    fake.save.mockRejectedValueOnce("File changed on disk");
    fake.getView.mockResolvedValue({ ...view(), revision: 3 });
    button("Save").click();
    await settle();
    button("Save reviewed changes").click();
    await settle();
    expect(document.querySelector('[aria-label="Review changes before save"]')).not.toBeNull();
    expect(document.querySelector('[role="alert"]')?.textContent).toContain("File changed on disk");
    button("Refresh review").click();
    await settle();
    expect(fake.save).toHaveBeenCalledTimes(1);
    expect(document.querySelector('[role="alert"]')?.textContent).toContain(
      "Review the current diff",
    );
    button("Save reviewed changes").click();
    await settle();
    expect(fake.save).toHaveBeenCalledTimes(2);
  });
  it("shows a macro draft across panels and projects its byte use", async () => {
    const withMacro = view();
    withMacro.macros = [{ index: 0, actions: [{ kind: "text", keys: [], text: "a", delay: 0 }] }];
    withMacro.macroBytes = 2;
    fake.getView.mockResolvedValue(withMacro);
    fake.macroUsage.mockImplementation(async (actions: { text: string }[]) => ({
      bytes: (actions[0]?.text.length ?? 0) + 1,
      actionBytes: [actions[0]?.text.length || 0],
    }));
    app = mount(App, { target: document.body });
    await settle();
    button("Macros").click();
    await settle();
    const input = document.querySelector<HTMLInputElement>('.action input:not([type="number"])')!;
    input.value = "longer";
    input.dispatchEvent(new Event("input", { bubbles: true }));
    await settle();
    expect(document.querySelector("aside")?.textContent).toContain(
      "7 bytes total projected from draft",
    );
    button("Combos").click();
    await settle();
    expect(button("Draft in macro M0")).toBeTruthy();
    button("Draft in macro M0").click();
    await settle();
    expect(ui.panel).toBe("macros");
    expect(
      document.querySelector<HTMLInputElement>('.action input:not([type="number"])')?.value,
    ).toBe("longer");
  });
  it("announces keyboard selection and returns focus after closing review", async () => {
    app = mount(App, { target: document.body });
    await settle();
    const keyButton = document.querySelector<SVGGElement>('[data-key-id="r0c0"]')!;
    keyButton.dispatchEvent(new KeyboardEvent("keydown", { key: " ", bubbles: true }));
    await settle();
    expect(keyButton.getAttribute("aria-pressed")).toBe("true");
    const save = button("Save");
    save.focus();
    save.click();
    await settle();
    expect(document.activeElement?.textContent).toContain("Save reviewed changes");
    document.activeElement!.dispatchEvent(
      new KeyboardEvent("keydown", { key: "Escape", bubbles: true }),
    );
    await settle();
    expect(document.querySelector('[aria-label="Review changes before save"]')).toBeNull();
    expect(document.activeElement).toBe(save);
  });
});
