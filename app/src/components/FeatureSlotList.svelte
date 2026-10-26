<script lang="ts">
  import { untrack } from "svelte";
  import {
    pointerDrag,
    pointerDrop,
    startPointerDrag,
    suppressDragClick,
  } from "../lib/pointer-drag.svelte";
  import { focusScope } from "../lib/focus";
  import {
    duplicateTargets,
    NOUNS,
    pasteTargets,
    slotMenu,
    type SlotCell,
    type SlotMenuId,
    type SlotMenuItem,
  } from "../lib/slot-menu";
  import { firstUseJump, usesOf, usesPhrase } from "../lib/slots";
  import { notify, ui } from "../lib/state.svelte";
  import type { OpDto, SlotKind, SlotUseDto } from "../lib/types";

  type Kind = SlotKind;
  type Item = {
    index: number;
    set: boolean;
    text: string;
    title?: string;
    changed?: boolean;
    /** Key overrides and alt repeat keys: the Enabled switch. */
    enabled?: boolean;
    /** Macros and tap dances: how many keycodes play the slot. */
    uses?: number;
  };
  let {
    kind,
    items,
    selected,
    perColumn,
    label,
    dirty = false,
    onselect,
    onoperation,
    onrename,
  }: {
    kind: Kind;
    items: Item[];
    selected: number;
    perColumn: number;
    label: string;
    dirty?: boolean;
    onselect: (index: number) => void;
    /** Runs an operation; `done` is the toast text when it succeeds. */
    onoperation: (op: OpDto, done?: string) => Promise<boolean>;
    /** Opens the rename dialog for a combo or macro. */
    onrename?: (index: number) => void;
  } = $props();
  const shortcutModifier = /Mac|iPhone|iPad/.test(navigator.platform) ? "⌘" : "Ctrl+";
  const KEY_ACTIONS: Record<string, SlotMenuId> = { x: "cut", c: "copy", v: "paste", d: "duplicate" };
  /** The menu's position, or null when it is closed. */
  let menu = $state<{ x: number; y: number } | null>(null);
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
  /** The slots the menu and the keys act on: the picked set, or the active slot alone. */
  function targets(): number[] {
    return picked.length > 0 ? [...picked].sort((a, b) => a - b) : [selected];
  }
  function cells(): SlotCell[] {
    return items.map((item) => ({ index: item.index, set: item.set, enabled: item.enabled }));
  }
  function isSet(index: number): boolean {
    return items.some((item) => item.index === index && item.set);
  }
  function menuItems(): SlotMenuItem[] {
    return slotMenu({
      kind,
      targets: targets(),
      cells: cells(),
      clipboard: ui.view?.clipboard ?? null,
      draft: kind === "macro" && dirty,
      modifier: shortcutModifier,
    });
  }
  const menuList: SlotMenuItem[] = $derived(menu ? menuItems() : []);

  /** Runs one menu item, or the key that names it. A disabled item says why instead. */
  async function run(id: SlotMenuId): Promise<void> {
    const entry = menuItems().find((candidate) => candidate.id === id);
    if (!entry) return;
    if (entry.reason) {
      notify(entry.reason);
      return;
    }
    menu = null;
    const indices = targets();
    const word = NOUNS[kind][0];
    const plural = NOUNS[kind][1];
    const described = indices.length === 1 ? `${word} ${indices[0]}` : `${indices.length} ${plural}`;
    const uses = indices.flatMap((index) => usesOf(ui.view, kind, index));
    switch (id) {
      case "clear": {
        const done = `Cleared ${described}`;
        if (await onoperation({ type: "clearFeatureSlots", kind, indices }, done)) followUses(done, uses);
        return;
      }
      case "cut": {
        const done = `Cut ${described}`;
        if (await onoperation({ type: "cutFeatureSlots", kind, indices }, done)) followUses(done, uses);
        return;
      }
      case "copy":
        await onoperation({ type: "copyFeatureSlots", kind, indices }, `Copied ${described}`);
        return;
      case "paste": {
        const clip = ui.view?.clipboard;
        const landing = clip ? pasteTargets(clip.count, indices, items.length) : [];
        const done =
          clip?.count === 1 && Array.isArray(landing)
            ? `Pasted into ${word} ${landing[0]}`
            : `Pasted ${clip?.count ?? 0} ${plural}`;
        await onoperation({ type: "pasteFeatureSlots", kind, indices }, done);
        return;
      }
      case "duplicate": {
        const sources = indices.filter(isSet);
        const destinations = duplicateTargets(cells(), indices) ?? [];
        const done =
          sources.length === 1
            ? `Duplicated ${word} ${sources[0]} to ${word} ${destinations[0]}`
            : `Duplicated ${sources.length} ${plural}`;
        if (await onoperation({ type: "copyFeatureSlotsTo", kind, from: sources, to: destinations }, done) && destinations[0] !== undefined)
          onselect(destinations[0]);
        return;
      }
      case "toggle": {
        const sources = indices.filter(isSet);
        const enable = !sources.every((index) => items.find((item) => item.index === index)?.enabled === true);
        const sourceText = sources.length === 1 ? `${word} ${sources[0]}` : `${sources.length} ${plural}`;
        await onoperation(
          { type: "setFeatureSlotsEnabled", kind, indices: sources, enabled: enable },
          `${enable ? "Enabled" : "Disabled"} ${sourceText}`,
        );
        return;
      }
      case "rename":
        onrename?.(indices[0]!);
        return;
    }
  }
  /** Clearing or cutting a slot that keys still use says so, with a way to the first use. */
  function followUses(done: string, uses: SlotUseDto[]): void {
    if (uses.length === 0) return;
    notify(`${done} · still used by ${usesPhrase(uses)}`, false, {
      label: "Show",
      jump: firstUseJump(uses),
    });
  }
  function drop(event: import("../lib/pointer-drag.svelte").PointerDrop, target: number): boolean {
    const payload = event.payload;
    if (payload.kind !== "featureSlot" || payload.feature !== kind) return false;
    if (payload.index === target || !guarded()) return true;
    if (event.altKey) void copyTo(payload.index, target);
    else void swapSlots(payload.index, target);
    return true;
  }
  async function swapSlots(a: number, b: number): Promise<void> {
    const word = NOUNS[kind][0];
    const moved = !isSet(b);
    const references = usesOf(ui.view, kind, a).length + usesOf(ui.view, kind, b).length;
    const verb = moved ? `Moved ${word} ${a} to ${word} ${b}` : `Swapped ${word} ${a} and ${word} ${b}`;
    const suffix = references > 0 ? ` · ${references} ${references === 1 ? "reference" : "references"} updated` : "";
    await onoperation({ type: "swapFeatureSlots", kind, a, b }, verb + suffix);
  }
  async function copyTo(from: number, to: number): Promise<void> {
    const word = NOUNS[kind][0];
    await onoperation(
      { type: "copyFeatureSlotsTo", kind, from: [from], to: [to] },
      `Copied ${word} ${from} to ${word} ${to}`,
    );
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
    keydown(e);
  }
  function keydown(e: KeyboardEvent): void {
    if (
      e.target instanceof Element &&
      e.target.closest("input,textarea,select,[contenteditable=true]")
    )
      return;
    if ((e.key === "Delete" || e.key === "Backspace") && !(e.metaKey || e.ctrlKey || e.altKey || e.shiftKey)) {
      e.preventDefault();
      void run("clear");
      return;
    }
    if (!(e.metaKey || e.ctrlKey) || e.altKey || e.shiftKey) return;
    const id = KEY_ACTIONS[e.key.toLowerCase()];
    if (!id) return;
    e.preventDefault();
    void run(id);
  }
  function menuAt(e: MouseEvent, index: number): void {
    e.preventDefault();
    // A right-click on a slot outside the picked set selects that slot alone first.
    if (!targets().includes(index)) {
      picked = [];
      anchor = index;
      onselect(index);
    }
    const height = menuItems().length * 30 + 24;
    menu = {
      x: Math.max(4, Math.min(e.clientX, window.innerWidth - 254)),
      y: Math.max(4, Math.min(e.clientY, window.innerHeight - height)),
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
      title={`${item.title ?? (item.text || "Empty")}${item.uses ? `\nUsed by ${item.uses}` : ""}${item.changed ? "\nChanged" : ""}\nDrag to swap · Alt-drag to copy`}
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
    class="menu feature-slot-menu"
    role="menu"
    tabindex="-1"
    style:left="{menu.x}px"
    style:top="{menu.y}px"
    use:focusScope={() => (menu = null)}
  >
    {#each menuList as entry (entry.id)}
      <button
        role="menuitem"
        disabled={!!entry.reason}
        title={entry.reason || entry.title}
        data-menu-divider={entry.divider}
        onclick={() => void run(entry.id)}
      >
        <span class="menu-label">{entry.label}</span>
        {#if entry.reason}<small class="menu-reason">{entry.reason}</small>{/if}
        <kbd>{entry.shortcut}</kbd>
      </button>
    {/each}
  </div>
{/if}

<style>
  .feature-slot-list .slot-cell {
    touch-action: none;
  }
  /* Out of the layout flow: a menu in the grid would stretch to the row's height. */
  .feature-slot-menu {
    position: fixed;
    z-index: 500;
    display: flex;
    flex-direction: column;
  }
</style>
