<script lang="ts">
  import {
    pointerDrag,
    pointerDrop,
    startPointerDrag,
    suppressDragClick,
  } from "../lib/pointer-drag.svelte";
  import { notify } from "../lib/state.svelte";
  import type { OpDto } from "../lib/types";

  type Kind = "combo" | "macro" | "tapDance" | "keyOverride" | "altRepeat";
  type Item = { index: number; set: boolean; text: string; title?: string };
  let {
    kind,
    items,
    selected,
    perColumn,
    label,
    dirty = false,
    onselect,
    onoperation,
  }: {
    kind: Kind;
    items: Item[];
    selected: number;
    perColumn: number;
    label: string;
    dirty?: boolean;
    onselect: (index: number) => void;
    onoperation: (op: OpDto) => Promise<boolean>;
  } = $props();
  const shortcutModifier = /Mac|iPhone|iPad/.test(navigator.platform) ? "⌘" : "Ctrl+";
  let menu = $state<{ index: number; x: number; y: number } | null>(null);

  function guarded(): boolean {
    if (kind === "macro" && dirty) {
      notify("Apply or revert the macro draft first");
      return false;
    }
    return true;
  }
  function slotAriaLabel(index: number): string {
    const label = {
      combo: `Combo slot C${index}`,
      macro: `Macro slot M${index}`,
      tapDance: `Tap dance slot TD${index}`,
      keyOverride: `Key override slot KO${index}`,
      altRepeat: `Alternate repeat slot AR${index}`,
    }[kind];
    return label;
  }
  function dragLabel(index: number, text: string): string {
    return `${slotAriaLabel(index).replace(" slot", "")} · ${text || "Empty"}`;
  }
  async function copy(index: number): Promise<void> {
    if (!guarded()) return;
    await onoperation({ type: "copyFeatureSlot", kind, index });
    menu = null;
  }
  async function paste(index: number): Promise<void> {
    if (!guarded()) return;
    await onoperation({ type: "pasteFeatureSlot", kind, index });
    menu = null;
  }
  function drop(event: import("../lib/pointer-drag.svelte").PointerDrop, target: number): boolean {
    const payload = event.payload;
    if (payload.kind !== "featureSlot" || payload.feature !== kind) return false;
    if (payload.index === target || !guarded()) return true;
    if (event.altKey) void copyTo(payload.index, target);
    else void onoperation({ type: "swapFeatureSlots", kind, a: payload.index, b: target });
    return true;
  }
  async function copyTo(from: number, to: number): Promise<void> {
    if (await onoperation({ type: "copyFeatureSlot", kind, index: from }))
      await onoperation({ type: "pasteFeatureSlot", kind, index: to });
  }
  function keydown(e: KeyboardEvent, index: number): void {
    if (!(e.metaKey || e.ctrlKey) || e.altKey || e.shiftKey) return;
    if (
      e.target instanceof Element &&
      e.target.closest("input,textarea,select,[contenteditable=true]")
    )
      return;
    const key = e.key.toLowerCase();
    if (key !== "c" && key !== "v") return;
    e.preventDefault();
    if (key === "c") void copy(index);
    else void paste(index);
  }
  function menuAt(e: MouseEvent, index: number): void {
    e.preventDefault();
    onselect(index);
    menu = {
      index,
      x: Math.max(4, Math.min(e.clientX, window.innerWidth - 170)),
      y: Math.max(4, Math.min(e.clientY, window.innerHeight - 76)),
    };
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="slot-list feature-slot-list"
  role="group"
  style="--per-column:{perColumn}"
  aria-label={label}
  onkeydown={(e) => keydown(e, selected)}
>
  {#each items as item (item.index)}
    <button
      class="slot-cell"
      data-slot-index={item.index}
      data-feature-slot-kind={kind}
      data-feature-slot-index={item.index}
      data-feature-slot-ready={kind !== "macro" || !dirty}
      data-state={item.set ? "set" : "empty"}
      class:active={selected === item.index}
      class:drag-source={pointerDrag.current?.payload.kind === "featureSlot" && pointerDrag.current.payload.feature === kind && pointerDrag.current.payload.index === item.index}
      class:drop-target={pointerDrag.current?.featureTarget?.feature === kind && pointerDrag.current.featureTarget.index === item.index}
      class:copy-target={pointerDrag.current?.featureTarget?.feature === kind && pointerDrag.current.featureTarget.index === item.index && pointerDrag.current.altKey}
      aria-pressed={selected === item.index}
      aria-label={slotAriaLabel(item.index)}
      title={item.title ?? (item.text || "Empty")}
      onclick={() => { if (!suppressDragClick()) onselect(item.index); }}
      oncontextmenu={(e) => menuAt(e, item.index)}
      onpointerdown={(e) =>
        startPointerDrag(
          e,
          { kind: "featureSlot", feature: kind, index: item.index },
          dragLabel(item.index, item.text),
        )}
      use:pointerDrop={(event) => drop(event, item.index)}
    >
      <span class="slot-id">{item.index}</span><span class="slot-desc">{item.text}</span>
    </button>
  {/each}
</div>
{#if menu}
  <div
    class="feature-slot-menu"
    role="menu"
    tabindex="-1"
    style:left="{menu.x}px"
    style:top="{menu.y}px"
    onkeydown={(e) => {
      if (e.key === "Escape") menu = null;
    }}
  >
    <button role="menuitem" onclick={() => void copy(menu!.index)}
      >Copy <kbd>{shortcutModifier}C</kbd></button
    >
    <button role="menuitem" onclick={() => void paste(menu!.index)}
      >Paste <kbd>{shortcutModifier}V</kbd></button
    >
  </div>
  <button class="feature-slot-dismiss" aria-label="Close slot menu" onclick={() => (menu = null)}
  ></button>
{/if}

<style>
  .feature-slot-list .slot-cell {
    touch-action: none;
  }
  .feature-slot-menu {
    position: fixed;
    z-index: 500;
    display: grid;
    min-width: 150px;
    padding: 4px;
    border: 1px solid var(--edge);
    border-radius: 9px;
    background: var(--panel);
    box-shadow: 0 8px 24px #0006;
  }
  .feature-slot-menu button {
    display: flex;
    justify-content: space-between;
    gap: 20px;
    padding: 7px 9px;
    border: 0;
    border-radius: 5px;
    background: transparent;
    color: inherit;
    text-align: left;
  }
  .feature-slot-menu button:hover,
  .feature-slot-menu button:focus-visible {
    background: var(--hover);
  }
  .feature-slot-menu kbd {
    color: var(--muted);
  }
  .feature-slot-dismiss {
    position: fixed;
    inset: 0;
    z-index: 499;
    border: 0;
    background: transparent;
  }
  .feature-slot-menu {
    z-index: 500;
  }
</style>
