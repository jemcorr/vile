// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { mount, tick, unmount } from "svelte";
import Picker from "./Picker.svelte";
import type { KeycodeView, SearchHitDto, ViewModel } from "../lib/types";

const api = vi.hoisted(() => ({
  searchKeycodes: vi.fn(),
  composeKeycode: vi.fn(),
  keycodeView: vi.fn(),
}));
vi.mock("../lib/api", () => ({ api, errorText: String }));
let component: ReturnType<typeof mount> | undefined;
afterEach(async () => {
  if (component) await unmount(component);
  component = undefined;
  document.body.innerHTML = "";
  vi.clearAllMocks();
});
async function settle() {
  for (let i = 0; i < 5; i++) {
    await Promise.resolve();
    await tick();
  }
}
function hit(
  name: string,
  pickerLabel: string,
  summary: string,
  pickerGroup: string,
): SearchHitDto {
  return {
    name,
    pickerLabel,
    summary,
    pickerGroup,
    description: `Full behavior for ${name}.`,
    insert: name,
    complete: true,
    code: 0x7000,
    label: name,
    category: "Quantum",
    group: "quantum",
    matched: name,
  };
}
const swap = hit("MAGIC_SWAP_CONTROL_CAPSLOCK", "Ctrl / Caps", "Swap keys", "Key remapping");
const audio = hit("AU_ON", "Audio on", "Keyboard sound", "Audio and click sounds");
const restore = hit(
  "MAGIC_UNSWAP_CONTROL_CAPSLOCK",
  "Ctrl / Caps",
  "Restore keys",
  "Key remapping",
);
async function picker() {
  const key: KeycodeView = {
    code: 0x7000,
    wire: swap.name,
    label: swap.label,
    description: "Compact inspector heading",
    tooltip: swap.description,
    category: "Quantum",
    parts: { kind: "raw", code: 0x7000 },
  };
  api.searchKeycodes.mockResolvedValue([swap, audio, restore]);
  api.composeKeycode.mockResolvedValue(key);
  api.keycodeView.mockResolvedValue(key);
  const choose = vi.fn();
  component = mount(Picker, {
    target: document.body,
    props: {
      view: {
        revision: 1,
        layerCount: 4,
        names: { layers: {}, macros: {}, combos: {} },
        macros: [],
      } as unknown as ViewModel,
      onchoose: choose,
      target: "Selected keys",
    },
  });
  await settle();
  [...document.querySelectorAll<HTMLButtonElement>("nav button")]
    .find((button) => button.textContent === "Quantum")!
    .click();
  await settle();
  return { choose, key };
}

describe("picker presentation", () => {
  it("groups category families and keeps full behavior and raw names in tooltips", async () => {
    const { choose, key } = await picker();
    expect([...document.querySelectorAll("h3")].map((heading) => heading.textContent)).toEqual([
      "Key remapping",
      "Audio and click sounds",
    ]);
    const group = document.querySelector('[aria-label="Key remapping"]')!;
    expect([...group.querySelectorAll("small")].map((caption) => caption.textContent)).toEqual([
      "Swap keys",
      "Restore keys",
    ]);
    const tile = group.querySelector<HTMLButtonElement>("button")!;
    expect(tile.querySelector("span")?.textContent).toBe("Ctrl / Caps");
    expect(tile.title).toBe(`${swap.description}\n${swap.name}`);
    expect(tile.getAttribute("aria-label")).toContain(swap.name);
    tile.click();
    await settle();
    expect(api.keycodeView).toHaveBeenCalledExactlyOnceWith(swap.name);
    expect(choose).toHaveBeenCalledExactlyOnceWith(key);
  });

  it("uses full behavior for builder previews while keeping their captions short", async () => {
    const { key } = await picker();
    [...document.querySelectorAll<HTMLButtonElement>("nav button")]
      .find((button) => button.textContent === "Layers")!
      .click();
    await settle();
    const preview = document.querySelector<HTMLButtonElement>(".preview")!;
    expect(preview.title).toBe(`${key.tooltip}\n${key.wire}`);
    expect(preview.textContent).not.toContain(key.tooltip);
    expect(preview.textContent).not.toContain(key.description);
  });

  it("keeps ranked search results in order without category group headings", async () => {
    await picker();
    api.searchKeycodes.mockResolvedValue([audio, restore, swap]);
    const input = document.querySelector<HTMLInputElement>('input[type="search"]')!;
    input.value = "anything";
    input.dispatchEvent(new Event("input", { bubbles: true }));
    await settle();
    expect(document.querySelectorAll("h3")).toHaveLength(0);
    expect(
      [...document.querySelectorAll<HTMLButtonElement>(".tiles button")].map(
        (tile) => tile.title.split("\n")[1],
      ),
    ).toEqual([audio.name, restore.name, swap.name]);
  });
});
