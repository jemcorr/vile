// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { mount, unmount, tick } from "svelte";
import FeatureSlotList from "./FeatureSlotList.svelte";
import DragPreview from "./DragPreview.svelte";
import { pointerDrag } from "../lib/pointer-drag.svelte";

const mocks = vi.hoisted(() => ({ notify: vi.fn() }));
vi.mock("../lib/state.svelte", () => ({ notify: mocks.notify }));

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
    expect(operation).toHaveBeenCalledWith({ type: "copyFeatureSlot", kind: "combo", index: 0 });
    first.dispatchEvent(
      new MouseEvent("contextmenu", { bubbles: true, cancelable: true, clientX: 20, clientY: 30 }),
    );
    await settle();
    document.querySelector<HTMLButtonElement>('[role="menuitem"]')!.click();
    await settle();
    expect(operation).toHaveBeenCalledWith({ type: "copyFeatureSlot", kind: "combo", index: 0 });
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
      [{ type: "swapFeatureSlots", kind: "combo", a: 0, b: 1 }],
      [{ type: "copyFeatureSlot", kind: "combo", index: 0 }],
      [{ type: "pasteFeatureSlot", kind: "combo", index: 1 }],
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
