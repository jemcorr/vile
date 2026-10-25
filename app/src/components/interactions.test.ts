// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { mount, unmount, tick } from "svelte";
import KeycodeInput from "./KeycodeInput.svelte";
import Keyboard from "./Keyboard.svelte";
import { startPointerDrag, suppressDragClick } from "../lib/pointer-drag.svelte";
import type { BoardKeyDto, KeyCellDto, SearchHitDto } from "../lib/types";
const mocks = vi.hoisted(() => ({ search: vi.fn(), resolve: vi.fn() }));
vi.mock("../lib/api", () => ({
  api: { searchKeycodes: mocks.search, resolveKeycode: mocks.resolve },
  errorText: (e: unknown) => String(e),
}));
let mounted: ReturnType<typeof mount> | undefined;
async function settle() {
  await Promise.resolve();
  await tick();
  await Promise.resolve();
  await tick();
}
afterEach(async () => {
  if (mounted) await unmount(mounted);
  mounted = undefined;
  document.body.innerHTML = "";
  vi.clearAllMocks();
});
const hit: SearchHitDto = {
  pickerLabel: "Esc",
  summary: "Escape",
  pickerGroup: "Editing and locks",
  name: "KC_ESCAPE",
  insert: "KC_ESCAPE",
  complete: true,
  label: "Esc",
  group: "basic",
  matched: "KC_ESCAPE",
  code: 41,
  description: "Escape",
  category: "Basic",
};
describe("explicit keycode completion", () => {
  beforeEach(() => {
    mocks.search.mockResolvedValue([hit]);
    mocks.resolve.mockImplementation((text: string) =>
      text === "KC_ESCAPE" ? Promise.resolve("KC_ESCAPE") : Promise.reject("Unknown keycode"),
    );
  });
  it("does not assign a fuzzy hit when typed input fails validation", async () => {
    const submit = vi.fn();
    mounted = mount(KeycodeInput, {
      target: document.body,
      props: { initial: "escpe", onsubmit: submit, oncancel: vi.fn() },
    });
    await settle();
    document
      .querySelector("input")!
      .dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    await settle();
    expect(submit).not.toHaveBeenCalled();
    expect(document.querySelector('[role="alert"]')?.textContent).toContain("Unknown keycode");
  });
  it("assigns a suggestion only after explicit selection", async () => {
    const submit = vi.fn();
    mounted = mount(KeycodeInput, {
      target: document.body,
      props: { initial: "escpe", onsubmit: submit, oncancel: vi.fn() },
    });
    await settle();
    document
      .querySelector(".hits button")!
      .dispatchEvent(new MouseEvent("click", { bubbles: true }));
    await settle();
    expect(document.querySelector<HTMLButtonElement>(".hits button")!.title).toBe(
      `${hit.description}\n${hit.name}`,
    );
    expect(document.activeElement).toBe(document.querySelector("input"));
    document
      .querySelector("input")!
      .dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    await settle();
    expect(submit).toHaveBeenCalledWith("KC_ESCAPE");
  });
});
const keys: BoardKeyDto[] = [
  { id: "a", row: 0, col: 0, x: 0, y: 0, w: 1, h: 1 },
  { id: "b", row: 0, col: 1, x: 2, y: 0, w: 1, h: 1 },
];
const cells: KeyCellDto[] = keys.map((key) => ({
  id: key.id,
  code: 4,
  wire: "KC_A",
  label: "A",
  kind: "basic",
  changed: false,
  path: `layout[0][0][${key.col}]`,
  description: "A",
  tooltip: "Detailed behavior while this key is held.",
  previous: "KC_A",
}));
function pointer(
  svg: EventTarget,
  type: string,
  x: number,
  y: number,
  alt = false,
  id = 1,
  modifiers: { shiftKey?: boolean; ctrlKey?: boolean } = {},
) {
  const event = new MouseEvent(type, {
    bubbles: true,
    clientX: x,
    clientY: y,
    button: 0,
    altKey: alt,
    shiftKey: modifiers.shiftKey ?? false,
    ctrlKey: modifiers.ctrlKey ?? false,
  });
  Object.defineProperty(event, "pointerId", { value: id });
  svg.dispatchEvent(event);
}
async function board(boardKeys = keys) {
  const drag = vi.fn(),
    marquee = vi.fn(),
    click = vi.fn(),
    blank = vi.fn(),
    assign = vi.fn(),
    edit = vi.fn();
  mounted = mount(Keyboard, {
    target: document.body,
    props: {
      keys: boardKeys,
      cells,
      cursor: "a",
      selection: [],
      onkeyclick: click,
      onmarquee: marquee,
      onblankclick: blank,
      ondragkey: drag,
      onassign: assign,
      onedit: edit,
      onmenu: vi.fn(),
    },
  });
  await settle();
  const svg = document.querySelector("svg")!;
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
  return { svg, drag, marquee, click, blank, assign, edit };
}
describe("board pointer controller", () => {
  it("uses detailed behavior in the board hover while keeping the drawn label compact", async () => {
    await board();
    const key = document.querySelector('[data-key-id="a"]')!;
    expect(key.querySelector("title")?.textContent).toBe(`${cells[0]!.tooltip} · KC_A`);
    expect(key.querySelector("text")?.textContent).toBe("A");
    expect(key.querySelector("text")?.textContent).not.toContain(cells[0]!.tooltip);
  });
  it("selects a captured pointer click once even when the browser retargets click to the SVG", async () => {
    const { svg, click } = await board();
    pointer(svg, "pointerdown", 30, 30);
    pointer(svg, "pointerup", 30, 30);
    svg.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(click).toHaveBeenCalledExactlyOnceWith("a", false);
  });
  it("passes Shift as the toggle modifier on key clicks", async () => {
    const { svg, click } = await board();
    pointer(svg, "pointerdown", 30, 30);
    pointer(svg, "pointerup", 30, 30);
    expect(click).toHaveBeenLastCalledWith("a", false);
    pointer(svg, "pointerdown", 30, 30, false, 1, { shiftKey: true });
    pointer(svg, "pointerup", 30, 30, false, 1, { shiftKey: true });
    expect(click).toHaveBeenLastCalledWith("a", true);
  });
  it("marks only selected keys, and keeps the cursor class without a selection", async () => {
    const props = (selection: string[]) => ({
      keys,
      cells,
      cursor: "a",
      selection,
      onkeyclick: vi.fn(),
      onmarquee: vi.fn(),
      onblankclick: vi.fn(),
      ondragkey: vi.fn(),
      onassign: vi.fn(),
      onedit: vi.fn(),
      onmenu: vi.fn(),
    });
    mounted = mount(Keyboard, { target: document.body, props: props(["a"]) });
    await settle();
    expect(document.querySelector('[data-key-id="a"]')!.classList).toContain("selected");
    await unmount(mounted);
    mounted = mount(Keyboard, { target: document.body, props: props([]) });
    await settle();
    const key = document.querySelector('[data-key-id="a"]')!;
    expect(key.classList).not.toContain("selected");
    expect(key.classList).toContain("cursor");
  });
  it("reports a completed empty click once but ignores marquee and canceled gestures", async () => {
    const { svg, blank, marquee } = await board();
    pointer(svg, "pointerdown", 70, -10);
    pointer(svg, "pointerup", 70, -10);
    svg.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(blank).toHaveBeenCalledTimes(1);
    pointer(svg, "pointerdown", 70, -10);
    pointer(svg, "pointermove", 190, 70);
    pointer(svg, "pointerup", 190, 70);
    svg.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(blank).toHaveBeenCalledTimes(1);
    expect(marquee).toHaveBeenCalledWith(["b"], false);
    pointer(svg, "pointerdown", 70, -10);
    pointer(svg, "pointercancel", 70, -10);
    pointer(svg, "pointerup", 70, -10);
    expect(blank).toHaveBeenCalledTimes(1);
  });
  it("starts a marquee from a gap between keys", async () => {
    const { svg, click, blank, marquee } = await board();
    pointer(svg, "pointerdown", 92, 20);
    pointer(svg, "pointermove", 160, 70);
    pointer(svg, "pointerup", 160, 70);
    expect(marquee).toHaveBeenCalledWith(["b"], false);
    expect(click).not.toHaveBeenCalled();
    expect(blank).not.toHaveBeenCalled();
  });
  it("treats the curve of a keycap corner as blank and the body as the key", async () => {
    const { svg, click, blank } = await board();
    pointer(svg, "pointerdown", 59, 59);
    pointer(svg, "pointerup", 59, 59);
    expect(blank).toHaveBeenCalledTimes(1);
    expect(click).not.toHaveBeenCalled();
    pointer(svg, "pointerdown", 50, 50);
    pointer(svg, "pointerup", 50, 50);
    expect(click).toHaveBeenCalledExactlyOnceWith("a", false);
  });
  it("treats an unpainted key gutter as blank, including rotated secondary keys", async () => {
    const { svg, click, blank } = await board([
      { ...keys[0]!, r: 90, rx: 0, ry: 0, secondary: { x: 1, y: 0, w: 1, h: 1 } },
    ]);
    pointer(svg, "pointerdown", -30, 61);
    pointer(svg, "pointerup", -30, 61);
    expect(blank).toHaveBeenCalledTimes(1);
    expect(click).not.toHaveBeenCalled();
    pointer(svg, "pointerdown", -30, 90);
    pointer(svg, "pointerup", -30, 90);
    expect(click).toHaveBeenCalledExactlyOnceWith("a", false);
  });
  it("ignores other pointers and checks the release position even without a move event", async () => {
    const { svg, click, blank, drag, marquee } = await board();
    pointer(svg, "pointerdown", 70, -10);
    pointer(svg, "pointerup", 70, -10, false, 2);
    expect(blank).not.toHaveBeenCalled();
    pointer(svg, "pointerup", 190, 70);
    expect(blank).not.toHaveBeenCalled();
    expect(marquee).toHaveBeenCalledWith(["b"], false);
    pointer(svg, "pointerdown", 30, 30);
    pointer(svg, "pointermove", 150, 30, false, 2);
    pointer(svg, "pointercancel", 30, 30, false, 2);
    pointer(svg, "pointerup", 150, 30);
    expect(drag).toHaveBeenCalledWith("a", "b", false);
    expect(click).not.toHaveBeenCalled();
  });
  it("opens inline editing when a captured double-click targets the SVG", async () => {
    const { svg, edit } = await board();
    svg.dispatchEvent(new MouseEvent("dblclick", { bubbles: true, clientX: 30, clientY: 30 }));
    expect(edit).toHaveBeenCalledExactlyOnceWith("a");
  });
  it("swaps a dragged key and suppresses its trailing click", async () => {
    const { svg, drag, click } = await board();
    pointer(svg, "pointerdown", 30, 30);
    pointer(svg, "pointermove", 150, 30);
    pointer(svg, "pointerup", 150, 30);
    document
      .querySelector('[data-key-id="b"]')!
      .dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(drag).toHaveBeenCalledWith("a", "b", false);
    expect(click).not.toHaveBeenCalled();
  });
  it("uses Alt at drop for copy", async () => {
    const { svg, drag } = await board();
    pointer(svg, "pointerdown", 30, 30);
    pointer(svg, "pointermove", 150, 30);
    pointer(svg, "pointerup", 150, 30, true);
    expect(drag).toHaveBeenCalledWith("a", "b", true);
  });
  it("cancels a drag on Escape without an edit", async () => {
    const { svg, drag, marquee } = await board();
    pointer(svg, "pointerdown", 30, 30);
    pointer(svg, "pointermove", 150, 30);
    window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    pointer(svg, "pointerup", 150, 30);
    expect(drag).not.toHaveBeenCalled();
    expect(marquee).not.toHaveBeenCalled();
  });
  it("starts marquee selection only in empty board space", async () => {
    const { svg, drag, marquee } = await board();
    pointer(svg, "pointerdown", 70, -10);
    pointer(svg, "pointermove", 190, 70);
    pointer(svg, "pointerup", 190, 70);
    expect(marquee).toHaveBeenCalledWith(["b"], false);
    expect(drag).not.toHaveBeenCalled();
  });
});

describe("picker pointer dragging", () => {
  async function source() {
    const state = await board();
    const tile = document.createElement("button");
    Object.assign(tile, {
      setPointerCapture: vi.fn(),
      releasePointerCapture: vi.fn(),
      hasPointerCapture: () => true,
    });
    tile.addEventListener("pointerdown", (event) =>
      startPointerDrag(event, { kind: "keycode", wire: "KC_B", code: 5 }, "B"),
    );
    document.body.append(tile);
    if (!document.elementFromPoint) document.elementFromPoint = () => null;
    vi.spyOn(document, "elementFromPoint").mockReturnValue(
      document.querySelector('[data-key-id="b"]'),
    );
    return { ...state, tile };
  }
  afterEach(() => vi.restoreAllMocks());
  it("assigns only the key under the drop and suppresses the source click", async () => {
    const { tile, assign, click } = await source();
    pointer(tile, "pointerdown", 10, 100);
    pointer(window, "pointermove", 150, 30);
    pointer(window, "pointerup", 150, 30);
    expect(assign).toHaveBeenCalledExactlyOnceWith("b", "KC_B");
    expect(click).not.toHaveBeenCalled();
    expect(suppressDragClick()).toBe(true);
  });
  it("cancels picker dragging on Escape without assignment", async () => {
    const { tile, assign } = await source();
    pointer(tile, "pointerdown", 10, 100);
    pointer(window, "pointermove", 150, 30);
    window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    pointer(window, "pointerup", 150, 30);
    expect(assign).not.toHaveBeenCalled();
  });
});
