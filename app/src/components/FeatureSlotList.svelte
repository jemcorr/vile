<script lang="ts">
  import { untrack } from "svelte";
  import {
    pointerDrag,
    pointerDrop,
    startPointerDrag,
    suppressDragClick,
  } from "../lib/pointer-drag.svelte";
  import { focusScope } from "../lib/focus";
  import { notify } from "../lib/state.svelte";
  import type { OpDto } from "../lib/types";

  type Kind = "combo" | "macro" | "tapDance" | "keyOverride" | "altRepeat";
  type Item = { index: number; set: boolean; text: string; title?: string; changed?: boolean };
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
  /** Slots picked with Shift or Command, besides the active one. Empty means only the active slot. */
  let picked = $state<number[]>([]);
  /** Where Shift+arrow started, when it is not the active slot. */
  let anchor = $state<number | null>(null);

  // The active slot moved without a click here (a reveal, a clamp): forget the picked set.
  $effect(() => {
    if (picked.length > 0 && !picked.includes(selected)) {
      untrack(() => {
        picked = [];
        anchor = null;
      });
    }
  });

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
    await onoperation({ type: "copyFeatureSlots", kind, indices: [index] });
    menu = null;
  }
  async function paste(index: number): Promise<void> {
    if (!guarded()) return;
    await onoperation({ type: "pasteFeatureSlots", kind, indices: [index] });
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
    await onoperation({ type: "copyFeatureSlotsTo", kind, from: [from], to: [to] });
  }
  function clicked(e: MouseEvent, index: number): void {
    if (suppressDragClick()) return;
    if (!(e.shiftKey || e.metaKey || e.ctrlKey)) {
      picked = [];
      anchor = index;
      onselect(index);
      return;
    }
    if (!guarded()) return;
    toggle(index);
  }
  function toggle(index: number): void {
    const seeded = picked.length > 0 ? picked : [selected];
    if (seeded.includes(index)) {
      const remaining = seeded.filter((i) => i !== index);
      picked = remaining;
      const last = remaining.at(-1);
      if (index === selected && last !== undefined) onselect(last);
      return;
    }
    picked = [...seeded, index];
    anchor = index;
    onselect(index);
  }
  /** The slot an arrow key reaches from the active one: two columns, a column per `perColumn`. */
  function neighbour(key: string): number | null {
    const row = selected % perColumn;
    const column = Math.floor(selected / perColumn);
    const next =
      key === "ArrowUp" && row > 0
        ? selected - 1
        : key === "ArrowDown" && row < perColumn - 1
          ? selected + 1
          : key === "ArrowLeft" && column > 0
            ? selected - perColumn
            : key === "ArrowRight" && column < 1
              ? selected + perColumn
              : -1;
    return next >= 0 && next < items.length ? next : null;
  }
  /** Moves with an arrow key; Shift extends the picked set from the anchor. Returns false when the key is not ours. */
  function arrow(e: KeyboardEvent): boolean {
    if (e.metaKey || e.ctrlKey || e.altKey) return false;
    if (!["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].includes(e.key)) return false;
    if (e.target instanceof Element && e.target.closest("input,textarea,select,[contenteditable=true]"))
      return false;
    const next = neighbour(e.key);
    if (next === null) return false;
    e.preventDefault();
    if (e.shiftKey) {
      if (!guarded()) return true;
      anchor = anchor ?? selected;
      const from = Math.min(anchor, next);
      const to = Math.max(anchor, next);
      picked = Array.from({ length: to - from + 1 }, (_, i) => from + i);
    } else {
      picked = [];
      anchor = next;
    }
    onselect(next);
    (e.currentTarget as HTMLElement)
      .querySelector<HTMLElement>(`[data-slot-index="${next}"]`)
      ?.focus();
    return true;
  }
  function onkey(e: KeyboardEvent): void {
    if (arrow(e)) return;
    keydown(e, selected);
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

<!-- A click outside the menu closes it, as the keyboard's context menu does. The board has no backdrop. -->
<svelte:window
  onclick={(e) => {
    if (menu && e.target instanceof Element && !e.target.closest(".feature-slot-menu")) menu = null;
  }}
/>
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="slot-list feature-slot-list"
  role="group"
  style="--per-column:{perColumn}"
  aria-label={label}
  onkeydown={onkey}
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
      class:picked={picked.includes(item.index)}
      class:drag-source={pointerDrag.current?.payload.kind === "featureSlot" && pointerDrag.current.payload.feature === kind && pointerDrag.current.payload.index === item.index}
      class:drop-target={pointerDrag.current?.featureTarget?.feature === kind && pointerDrag.current.featureTarget.index === item.index}
      class:copy-target={pointerDrag.current?.featureTarget?.feature === kind && pointerDrag.current.featureTarget.index === item.index && pointerDrag.current.altKey}
      aria-pressed={selected === item.index}
      aria-label={slotAriaLabel(item.index)}
      title={`${item.title ?? (item.text || "Empty")}${item.changed ? "\nChanged" : ""}\nDrag to swap · Alt-drag to copy`}
      onclick={(e) => clicked(e, item.index)}
      oncontextmenu={(e) => menuAt(e, item.index)}
      onpointerdown={(e) =>
        startPointerDrag(
          e,
          { kind: "featureSlot", feature: kind, index: item.index },
          dragLabel(item.index, item.text),
        )}
      use:pointerDrop={(event) => drop(event, item.index)}
    >
      <span class="slot-id">{item.index}</span><span class="slot-desc">{item.text}</span>{#if item.changed}<i class="change-dot slot-change" aria-hidden="true"></i>{/if}
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
    use:focusScope={() => (menu = null)}
  >
    <button role="menuitem" onclick={() => void copy(menu!.index)}
      >Copy <kbd>{shortcutModifier}C</kbd></button
    >
    <button role="menuitem" onclick={() => void paste(menu!.index)}
      >Paste <kbd>{shortcutModifier}V</kbd></button
    >
  </div>
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
</style>
