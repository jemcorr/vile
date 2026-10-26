// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { mount, unmount, tick } from "svelte";
import FeatureSlotList from "./FeatureSlotList.svelte";
import DragPreview from "./DragPreview.svelte";
import { pointerDrag } from "../lib/pointer-drag.svelte";

const mocks = vi.hoisted(() => ({ notify: vi.fn() }));
vi.mock("../lib/state.svelte", () => ({ notify: mocks.notify, ui: { view: null } }));

let mounted: ReturnType<typeof mount> | undefined;
let previewMounted: ReturnType<typeof mount> | undefined;
const originalElementFromPoint = document.elementFromPoint;
async function settle() {
  await Promise.resolve();
  await tick();
  await Promise.resolve();
  await tick();
}
afterEach(async () => {
  if (mounted) await unmount(mounted);
  if (previewMounted) await unmount(previewMounted);
  mounted = undefined;
  previewMounted = undefined;
  pointerDrag.current = null;
  Object.defineProperty(document, "elementFromPoint", {
    configurable: true,
    value: originalElementFromPoint,
  });
  document.body.innerHTML = "";
  vi.clearAllMocks();
});
it("marks a changed slot with a dot and says so in the hover", async () => {
  mounted = mount(FeatureSlotList, {
    target: document.body,
    props: {
      kind: "combo",
      label: "Combo",
      perColumn: 16,
      selected: 0,
      onselect: vi.fn(),
      onoperation: vi.fn(async () => true),
      items: [
        { index: 0, set: true, text: "first", changed: true },
        { index: 1, set: false, text: "", changed: false },
      ],
    },
  });
  await settle();
  const changedCell = document.querySelector<HTMLButtonElement>('[data-slot-index="0"]')!;
  expect(changedCell.querySelector(".slot-change")).not.toBeNull();
  expect(changedCell.title).toContain("Changed");
  expect(document.querySelector('[data-slot-index="1"]')!.querySelector(".slot-change")).toBeNull();
});
function render(kind: "combo" | "macro" = "combo", dirty = false) {
  const operation = vi.fn(async () => true);
  mounted = mount(FeatureSlotList, {
    target: document.body,
    props: {
      kind,
      items: [
        { index: 0, set: true, text: "first" },
        { index: 1, set: true, text: "second" },
      ],
      selected: 0,
      perColumn: 8,
      label: kind,
      dirty,
      onselect: vi.fn(),
      onoperation: operation,
    },
  });
  return operation;
}
function pointer(target: EventTarget, type: string, x: number, altKey = false) {
  const event = new MouseEvent(type, { bubbles: true, clientX: x, clientY: 0, button: 0, altKey });
  Object.defineProperty(event, "pointerId", { value: 1 });
  target.dispatchEvent(event);
}
function drag(source: HTMLButtonElement, target: HTMLButtonElement, altKey = false) {
  Object.defineProperty(document, "elementFromPoint", { configurable: true, value: () => target });
  source.setPointerCapture = vi.fn();
  source.hasPointerCapture = vi.fn(() => false);
  source.releasePointerCapture = vi.fn();
  pointer(source, "pointerdown", 0);
  pointer(window, "pointermove", 20);
  pointer(window, "pointerup", 20, altKey);
}

describe("feature slot actions", () => {
  it("uses same-type slot shortcuts and context menu operations", async () => {
    const operation = render();
    const first = document.querySelector<HTMLButtonElement>('[data-slot-index="0"]')!;
    first.dispatchEvent(new KeyboardEvent("keydown", { key: "c", ctrlKey: true, bubbles: true }));
    await settle();
    expect(operation).toHaveBeenCalledWith({ type: "copyFeatureSlots", kind: "combo", indices: [0] }, "Copied combo 0");
    first.dispatchEvent(
      new MouseEvent("contextmenu", { bubbles: true, cancelable: true, clientX: 20, clientY: 30 }),
    );
    await settle();
    [...document.querySelectorAll<HTMLButtonElement>('[role="menuitem"]')]
      .find((button) => button.textContent!.includes("Copy"))!
      .click();
    await settle();
    expect(operation).toHaveBeenLastCalledWith({ type: "copyFeatureSlots", kind: "combo", indices: [0] }, "Copied combo 0");
  });

  it("opens the slot menu without a backdrop and closes it on outside click or Escape", async () => {
    render();
    await settle();
    const first = document.querySelector<HTMLButtonElement>('[data-slot-index="0"]')!;
    const open = () =>
      first.dispatchEvent(
        new MouseEvent("contextmenu", { bubbles: true, cancelable: true, clientX: 20, clientY: 30 }),
      );
    open();
    await settle();
    const menu = document.querySelector<HTMLElement>(".feature-slot-menu")!;
    expect(menu).not.toBeNull();
    // A full-viewport button under the menu would take the hover fill across the whole workspace.
    expect(document.querySelector(".feature-slot-dismiss")).toBeNull();
    expect(menu.contains(document.activeElement)).toBe(true);
    const outside = document.createElement("input");
    document.body.append(outside);
    outside.click();
    await settle();
    expect(document.querySelector(".feature-slot-menu")).toBeNull();
    open();
    await settle();
    document.activeElement!.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    await settle();
    expect(document.querySelector(".feature-slot-menu")).toBeNull();
  });

  it("swaps on same-type drag and copies on Alt-drop", async () => {
    const operation = render();
    await settle();
    const source = document.querySelector<HTMLButtonElement>('[data-slot-index="0"]')!;
    const target = document.querySelector<HTMLButtonElement>('[data-slot-index="1"]')!;
    drag(source, target);
    drag(source, target, true);
    await settle();
    expect(operation.mock.calls).toEqual([
      [{ type: "swapFeatureSlots", kind: "combo", a: 0, b: 1 }, "Swapped combo 0 and combo 1"],
      [{ type: "copyFeatureSlotsTo", kind: "combo", from: [0], to: [1] }, "Copied combo 0 to combo 1"],
    ]);
  });

  it("shows the held slot, valid target and live copy mode, then clears on cancel and blur", async () => {
    const operation = render();
    previewMounted = mount(DragPreview, { target: document.body });
    await settle();
    const source = document.querySelector<HTMLButtonElement>('[data-slot-index="0"]')!;
    const target = document.querySelector<HTMLButtonElement>('[data-slot-index="1"]')!;
    source.setPointerCapture = vi.fn();
    source.hasPointerCapture = vi.fn(() => false);
    source.releasePointerCapture = vi.fn();
    Object.defineProperty(document, "elementFromPoint", { configurable: true, value: () => target });

    pointer(source, "pointerdown", 0);
    pointer(window, "pointermove", 20, true);
    await settle();
    expect(source.classList.contains("drag-source")).toBe(true);
    expect(target.classList.contains("drop-target")).toBe(true);
    expect(target.classList.contains("copy-target")).toBe(true);
    expect(document.querySelector(".drag-preview")?.textContent).toContain("Combo C0 · first");
    expect(document.querySelector(".drag-preview")?.textContent).toContain("Copy");

    window.dispatchEvent(new KeyboardEvent("keyup", { key: "Alt" }));
    await settle();
    expect(target.classList.contains("drop-target")).toBe(true);
    expect(target.classList.contains("copy-target")).toBe(false);
    expect(document.querySelector(".drag-preview")?.textContent).toContain("Swap");

    const outside = document.createElement("input");
    document.body.append(outside);
    Object.defineProperty(document, "elementFromPoint", { configurable: true, value: () => outside });
    pointer(window, "pointermove", 28);
    await settle();
    expect(target.classList.contains("drop-target")).toBe(false);
    expect(source.classList.contains("drag-source")).toBe(true);
    window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    await settle();
    expect(document.querySelector(".drag-preview")).toBeNull();
    expect(source.classList.contains("drag-source")).toBe(false);
    expect(operation).not.toHaveBeenCalled();

    pointer(source, "pointerdown", 40);
    pointer(window, "pointermove", 60);
    pointer(window, "pointercancel", 60);
    await settle();
    expect(document.querySelector(".drag-preview")).toBeNull();
    expect(source.classList.contains("drag-source")).toBe(false);

    Object.defineProperty(document, "elementFromPoint", { configurable: true, value: () => target });
    pointer(source, "pointerdown", 80);
    pointer(window, "pointermove", 100);
    window.dispatchEvent(new Event("blur"));
    await settle();
    expect(document.querySelector(".drag-preview")).toBeNull();
    expect(source.classList.contains("drag-source")).toBe(false);
  });

  it("ignores cross-type drags and blocks macro slot operations while a draft is dirty", async () => {
    const operation = render("macro", true);
    await settle();
    const source = document.querySelector<HTMLButtonElement>('[data-slot-index="0"]')!;
    const target = document.querySelector<HTMLButtonElement>('[data-slot-index="1"]')!;
    target.dispatchEvent(
      new CustomEvent("vile-pointer-drop", {
        bubbles: true,
        detail: {
          payload: { kind: "featureSlot", feature: "combo", index: 0 },
          clientX: 0,
          clientY: 0,
        },
      }),
    );
    drag(source, target);
    target.dispatchEvent(new KeyboardEvent("keydown", { key: "c", metaKey: true, bubbles: true }));
    await settle();
    expect(operation).not.toHaveBeenCalled();
    expect(mocks.notify).toHaveBeenCalledWith("Apply or revert the macro draft first");
  });

  it("does not advertise a macro drop target while the draft is dirty", async () => {
    render("macro", true);
    await settle();
    const source = document.querySelector<HTMLButtonElement>('[data-slot-index="0"]')!;
    const target = document.querySelector<HTMLButtonElement>('[data-slot-index="1"]')!;
    source.setPointerCapture = vi.fn();
    source.hasPointerCapture = vi.fn(() => false);
    source.releasePointerCapture = vi.fn();
    Object.defineProperty(document, "elementFromPoint", { configurable: true, value: () => target });
    pointer(source, "pointerdown", 0);
    pointer(window, "pointermove", 20);
    await settle();
    expect(target.dataset.featureSlotReady).toBe("false");
    expect(target.classList.contains("drop-target")).toBe(false);
    window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
  });

  it("leaves Cmd+C and Cmd+V alone in feature text inputs", async () => {
    const operation = render("macro");
    const input = document.createElement("input");
    document.body.append(input);
    const copy = new KeyboardEvent("keydown", { key: "c", metaKey: true, bubbles: true, cancelable: true });
    const paste = new KeyboardEvent("keydown", { key: "v", metaKey: true, bubbles: true, cancelable: true });
    input.dispatchEvent(copy);
    input.dispatchEvent(paste);
    await settle();
    expect(operation).not.toHaveBeenCalled();
    expect(copy.defaultPrevented).toBe(false);
    expect(paste.defaultPrevented).toBe(false);
  });
});

function mountList(count: number, perColumn: number, start = 0) {
  // The active slot is read through a getter, so each key sees the slot the last one selected.
  let current = start;
  const onselect = vi.fn((index: number) => {
    current = index;
  });
  mounted = mount(FeatureSlotList, {
    target: document.body,
    props: {
      kind: "combo",
      label: "Combo",
      perColumn,
      get selected() {
        return current;
      },
      onselect,
      onoperation: vi.fn(async () => true),
      items: Array.from({ length: count }, (_, index) => ({ index, set: true, text: `c${index}` })),
    },
  });
  return onselect;
}
// A drop arms a zero-delay timer that ends click suppression; let it run before clicking.
async function clickCell(index: number, modifiers: MouseEventInit = {}) {
  await new Promise((resolve) => setTimeout(resolve, 0));
  document
    .querySelector<HTMLButtonElement>(`[data-slot-index="${index}"]`)!
    .dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true, ...modifiers }));
}
function pressOn(index: number, key: string, shiftKey = false) {
  document
    .querySelector<HTMLButtonElement>(`[data-slot-index="${index}"]`)!
    .dispatchEvent(new KeyboardEvent("keydown", { key, shiftKey, bubbles: true, cancelable: true }));
}
function pickedIndices(): number[] {
  return [...document.querySelectorAll<HTMLElement>(".slot-cell.picked")].map((cell) =>
    Number(cell.dataset.slotIndex),
  );
}

describe("feature slot selection", () => {
  it("toggles a picked set with Shift and Command clicks, seeded with the active slot", async () => {
    mountList(8, 4, 0);
    await settle();
    await clickCell(2, { shiftKey: true });
    await settle();
    await clickCell(3, { metaKey: true });
    await settle();
    expect(pickedIndices()).toEqual([0, 2, 3]);
    await clickCell(2, { metaKey: true });
    await settle();
    expect(pickedIndices()).toEqual([0, 3]);
  });

  it("moves with arrows within a column and jumps between the two columns", async () => {
    const onselect = mountList(32, 16, 0);
    await settle();
    pressOn(0, "ArrowDown");
    expect(onselect).toHaveBeenLastCalledWith(1);
    pressOn(1, "ArrowRight");
    expect(onselect).toHaveBeenLastCalledWith(17);
    expect(pickedIndices()).toEqual([]);
  });

  it("extends the picked range with Shift+arrows from the anchor", async () => {
    const onselect = mountList(32, 16, 1);
    await settle();
    pressOn(1, "ArrowDown", true);
    await settle();
    pressOn(2, "ArrowDown", true);
    await settle();
    expect(onselect.mock.calls.map(([index]) => index)).toEqual([2, 3]);
    expect(pickedIndices()).toEqual([1, 2, 3]);
  });

  it("leaves arrow keys alone inside a text field", async () => {
    const onselect = mountList(4, 2, 0);
    await settle();
    const input = document.createElement("input");
    document.querySelector(".slot-list")!.appendChild(input);
    input.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
    expect(onselect).not.toHaveBeenCalled();
  });

  it("refuses Shift-clicks while a macro draft is unapplied", async () => {
    render("macro", true);
    await settle();
    await clickCell(1, { shiftKey: true });
    await settle();
    expect(mocks.notify).toHaveBeenCalledWith("Apply or revert the macro draft first");
    expect(pickedIndices()).toEqual([]);
  });
});

function mountSlots(items: { index: number; set: boolean; text: string }[], selected = 0) {
  const operation = vi.fn(async () => true);
  const onselect = vi.fn();
  mounted = mount(FeatureSlotList, {
    target: document.body,
    props: { kind: "combo", label: "Combo", perColumn: 8, selected, onselect, onoperation: operation, items },
  });
  return { operation, onselect };
}
function commandKey(index: number, key: string) {
  document
    .querySelector<HTMLButtonElement>(`[data-slot-index="${index}"]`)!
    .dispatchEvent(new KeyboardEvent("keydown", { key, metaKey: true, bubbles: true, cancelable: true }));
}

describe("feature slot menu", () => {
  it("clears the picked slots with Delete and Backspace", async () => {
    const { operation } = mountSlots(Array.from({ length: 4 }, (_, index) => ({ index, set: true, text: `c${index}` })));
    await settle();
    await clickCell(2, { shiftKey: true });
    await settle();
    pressOn(0, "Delete");
    await settle();
    expect(operation).toHaveBeenLastCalledWith(
      { type: "clearFeatureSlots", kind: "combo", indices: [0, 2] },
      "Cleared 2 combos",
    );
    pressOn(0, "Backspace");
    await settle();
    expect(operation).toHaveBeenCalledTimes(2);
  });

  it("duplicates into the next empty slot with Command-D and cuts with Command-X", async () => {
    const { operation } = mountSlots([
      { index: 0, set: true, text: "a" },
      { index: 1, set: true, text: "b" },
      { index: 2, set: false, text: "" },
      { index: 3, set: false, text: "" },
    ]);
    await settle();
    commandKey(0, "d");
    await settle();
    expect(operation).toHaveBeenLastCalledWith(
      { type: "copyFeatureSlotsTo", kind: "combo", from: [0], to: [2] },
      "Duplicated combo 0 to combo 2",
    );
    commandKey(0, "x");
    await settle();
    expect(operation).toHaveBeenLastCalledWith(
      { type: "cutFeatureSlots", kind: "combo", indices: [0] },
      "Cut combo 0",
    );
  });

  it("says why paste is off and runs nothing when the clipboard is empty", async () => {
    const { operation } = mountSlots([{ index: 0, set: true, text: "a" }]);
    await settle();
    commandKey(0, "v");
    await settle();
    expect(mocks.notify).toHaveBeenCalledWith("Clipboard is empty");
    expect(operation).not.toHaveBeenCalled();
  });

  it("opens a menu of the family's items with their reasons", async () => {
    mountSlots([{ index: 0, set: true, text: "a" }]);
    await settle();
    document
      .querySelector<HTMLButtonElement>('[data-slot-index="0"]')!
      .dispatchEvent(new MouseEvent("contextmenu", { bubbles: true, cancelable: true, clientX: 20, clientY: 30 }));
    await settle();
    const menu = document.querySelector<HTMLElement>(".feature-slot-menu")!;
    expect(menu.classList.contains("menu")).toBe(true);
    const paste = [...menu.querySelectorAll<HTMLButtonElement>('[role="menuitem"]')].find((button) =>
      button.textContent!.includes("Paste"),
    )!;
    expect(paste.disabled).toBe(true);
    expect(paste.title).toBe("Clipboard is empty");
    expect(paste.querySelector(".menu-reason")!.textContent).toBe("Clipboard is empty");
  });
});

describe("feature slot new entries", () => {
  it("shows + New in the first empty slot, and in others only on hover or focus", async () => {
    mountSlots([
      { index: 0, set: true, text: "a" },
      { index: 1, set: false, text: "" },
      { index: 2, set: false, text: "" },
    ]);
    await settle();
    expect(document.querySelector('[data-slot-index="1"]')!.classList.contains("first-empty")).toBe(true);
    expect(document.querySelector('[data-slot-index="2"]')!.classList.contains("first-empty")).toBe(false);
    expect(document.querySelector('[data-slot-index="1"] .slot-new')!.textContent).toBe("+ New");
    expect(document.querySelector('[data-slot-index="0"] .slot-new')).toBeNull();
    expect(document.querySelector('[data-slot-index="2"]')!.getAttribute("aria-label")).toBe("Combo slot C2, empty");
  });

  it("selects an empty slot and then starts a new entry, without a write", async () => {
    const operation = vi.fn(async () => true);
    const onselect = vi.fn();
    const onnew = vi.fn();
    mounted = mount(FeatureSlotList, {
      target: document.body,
      props: {
        kind: "combo",
        label: "Combo",
        perColumn: 8,
        selected: 0,
        onselect,
        onnew,
        onoperation: operation,
        items: [
          { index: 0, set: true, text: "a" },
          { index: 1, set: false, text: "" },
        ],
      },
    });
    await settle();
    await clickCell(1);
    await settle();
    expect(onselect).toHaveBeenCalledWith(1);
    expect(onnew).toHaveBeenCalledWith(1);
    expect(onselect.mock.invocationCallOrder[0]).toBeLessThan(onnew.mock.invocationCallOrder[0]!);
    expect(operation).not.toHaveBeenCalled();
    await clickCell(0);
    await settle();
    expect(onnew).toHaveBeenCalledTimes(1);
  });
});
