// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { mount, tick, unmount } from "svelte";
import Field from "./Field.svelte";
import Inspector from "./Inspector.svelte";
import { ui } from "../lib/state.svelte";
import type { KeycodeView, ViewModel } from "../lib/types";
const api = vi.hoisted(() => ({ keycodeView: vi.fn() }));
vi.mock("../lib/api", () => ({ api }));
const key: KeycodeView = {
  code: 0xdd,
  wire: "KC_ACL0",
  label: "Speed\n0",
  description: "Mouse speed preset 0",
  tooltip: "Hold with a movement key; firmware may instead latch the preset after a tap.",
  category: "Mouse",
  parts: { kind: "tap", base: 0xdd, wrapMods: 0, hold: { kind: "none" } },
};
let component: ReturnType<typeof mount> | undefined;
afterEach(async () => {
  if (component) await unmount(component);
  component = undefined;
  ui.view = null;
  ui.inspectorPaths = [];
  document.body.innerHTML = "";
  vi.clearAllMocks();
});
async function settle() {
  for (let i = 0; i < 4; i++) {
    await Promise.resolve();
    await tick();
  }
}

describe("shared behavior hovers", () => {
  it("uses the detailed field tooltip while preserving the compact visible description", async () => {
    ui.view = { keycodes: { "macro[0]": key } } as unknown as ViewModel;
    component = mount(Field, {
      target: document.body,
      props: { path: "macro[0]", label: "Output" },
    });
    await settle();
    const field = document.querySelector<HTMLButtonElement>(".field")!;
    expect(field.title).toBe(`${key.tooltip}\n${key.wire}`);
    expect(field.getAttribute("aria-label")).toBe(`Output: ${key.description}`);
    expect(field.textContent).not.toContain(key.tooltip);
  });
  it("marks a keycap changed from the comparison point and shows the earlier value on hover", async () => {
    const before = { ...key, code: 5, wire: "KC_B", label: "B", description: "B", tooltip: key.tooltip };
    ui.view = {
      keycodes: { "macro[0]": { ...key, code: 0xdd } },
      baseline: { keycodes: { "macro[0]": before } },
    } as unknown as ViewModel;
    component = mount(Field, {
      target: document.body,
      props: { path: "macro[0]", label: "Output" },
    });
    await settle();
    const field = document.querySelector<HTMLButtonElement>(".keycap-field")!;
    expect(field.getAttribute("aria-label")).toBe(`Output: ${key.description}, changed`);
    expect(field.title).toContain("\nWas B");
    expect(field.querySelector(".change-mark")).not.toBeNull();
  });
  it("keeps the inspector heading compact and explains both the heading and base on hover", async () => {
    api.keycodeView.mockResolvedValue(key);
    component = mount(Inspector, {
      target: document.body,
      props: { keys: [key], layerCount: 4, onpatch: vi.fn(), onbase: vi.fn() },
    });
    await settle();
    const heading = document.querySelector<HTMLHeadingElement>("h2")!;
    expect(heading.textContent).toBe(key.description);
    expect(heading.title).toBe(`${key.tooltip}\n${key.wire}`);
    expect(document.querySelector<HTMLButtonElement>(".control button")!.title).toContain(
      key.tooltip,
    );
    expect(document.querySelector("details.keycode-details p")!.textContent).toBe(key.tooltip);
  });
});
