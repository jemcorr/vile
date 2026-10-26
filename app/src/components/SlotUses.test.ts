// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { mount, unmount, tick } from "svelte";
import SlotUses from "./SlotUses.svelte";

const state = vi.hoisted(() => ({ ui: { view: null as unknown, jump: null as unknown } }));
vi.mock("../lib/state.svelte", () => ({ ui: state.ui }));

let mounted: ReturnType<typeof mount> | undefined;
afterEach(async () => {
  if (mounted) await unmount(mounted);
  mounted = undefined;
  document.body.innerHTML = "";
  state.ui.view = null;
  state.ui.jump = null;
});

const keyUse = (layer: number, keyId: string, path: string) => ({
  feature: "macro",
  index: 2,
  path,
  place: { kind: "key", layer, keyId },
});
const comboUse = (index: number, field: string) => ({
  feature: "macro",
  index: 2,
  path: `combo[${index}][4]`,
  place: { kind: "slot", feature: "combo", index, field },
});

function view(slotUses: unknown[]) {
  return {
    slotUses,
    board: { keys: [{ id: "r4c0", row: 4, col: 0, name: null }] },
    layers: [
      [
        { id: "r4c0", label: "S", kind: "basic" },
        { id: "r6c1", label: "'", kind: "basic" },
      ],
    ],
    names: { layers: { "7": "Symbols" } },
  };
}

function render() {
  mounted = mount(SlotUses, { target: document.body, props: { feature: "macro", index: 2 } });
}
const chips = () => [...document.querySelectorAll<HTMLButtonElement>(".use-chip")];

describe("slot uses row", () => {
  it("names each place: the key on its layer, a combo field", async () => {
    state.ui.view = view([
      keyUse(7, "r4c0", "layout[7][4][0]"),
      keyUse(0, "r6c1", "layout[0][6][1]"),
      comboUse(3, "output"),
    ]);
    render();
    await tick();
    expect(chips().map((chip) => chip.textContent)).toEqual([
      "Layer 7 · S",
      "Layer 0 · r6c1",
      "Combo 3 · Output",
    ]);
    expect(document.querySelector(".slot-uses-label")!.textContent).toBe("Used by");
  });

  it("shows eight chips and a +N chip for the rest", async () => {
    state.ui.view = view(Array.from({ length: 10 }, (_, i) => keyUse(0, "r6c1", `layout[0][${i}][0]`)));
    render();
    await tick();
    expect(chips()).toHaveLength(9);
    expect(chips().at(-1)!.textContent).toBe("+2");
  });

  it("says when nothing plays the slot", async () => {
    state.ui.view = view([]);
    render();
    await tick();
    expect(document.querySelector(".slot-uses-none")!.textContent).toBe("Unused");
    expect(chips()).toHaveLength(0);
  });

  it("jumps to the key when its chip is clicked", async () => {
    state.ui.view = view([keyUse(7, "r4c0", "layout[7][4][0]")]);
    render();
    await tick();
    chips()[0]!.click();
    expect(state.ui.jump).toEqual({ kind: "keys", layer: 7, keyIds: ["r4c0"] });
  });
});
