<script lang="ts">
  import type { TapDanceDto } from "../lib/types";
  import Field from "./Field.svelte";
  import { ui } from "../lib/state.svelte";
  let { tapDances, onsave }: { tapDances: TapDanceDto[]; onsave: (value: TapDanceDto) => void } =
    $props();
  let selected = $state(0);
  $effect(() => {
    if (!tapDances.some((slot) => slot.index === selected)) selected = tapDances[0]?.index ?? 0;
  });
  function pickSlot(index: number): void {
    selected = index;
    ui.inspectorPaths = [];
  }

  function slotSet(td: TapDanceDto): boolean {
    return [td.onTap, td.onHold, td.onDoubleTap, td.onTapHold].some((key) => key !== "KC_NO");
  }
  function slotText(td: TapDanceDto): string {
    const keys = [td.onTap, td.onHold, td.onDoubleTap, td.onTapHold];
    const names = ["Tap", "Hold", "Double tap", "Tap and hold"];
    return keys
      .map((key, i) => (key === "KC_NO" ? "" : names[i] + " " + (ui.view?.keycodes["tap_dance[" + td.index + "][" + i + "]"]?.label || key)))
      .filter(Boolean)
      .join(" · ");
  }
  function slotKeys(e: KeyboardEvent, perColumn: number, pick: (index: number) => void): void {
    const row = selected % perColumn, column = Math.floor(selected / perColumn);
    const next = e.key === "ArrowUp" && row > 0 ? selected - 1
      : e.key === "ArrowDown" && row < perColumn - 1 ? selected + 1
      : e.key === "ArrowLeft" && column > 0 ? selected - perColumn
      : e.key === "ArrowRight" && column < 1 ? selected + perColumn
      : -1;
    if (next < 0) return;
    e.preventDefault();
    pick(next);
    if (selected === next) (e.currentTarget as HTMLElement).querySelector<HTMLElement>('[data-slot-index="' + next + '"]')?.focus();
  }
</script>

<section class="slot-editor" data-slot-editor="tapDance"><!-- svelte-ignore a11y_no_noninteractive_element_interactions --><div class="slot-list" role="group" style="--per-column:16" aria-label="tapDance slots" onkeydown={(e) => slotKeys(e, 16, pickSlot)}>{#each tapDances as td (td.index)}<button class="slot-cell" data-slot-index={td.index} data-state={slotSet(td) ? "set" : "empty"} class:active={selected === td.index} aria-pressed={selected === td.index} aria-label={"Tap dance slot TD" + td.index} title={slotText(td) || "Empty"} onclick={() => pickSlot(td.index)}><span class="slot-id">{td.index}</span><span class="slot-desc">{slotText(td)}</span></button>{/each}</div><div class="slot-form">{#each tapDances as td (td.index)}{#if td.index === selected}<article class="slot-edit-panel" aria-label={`Editing tap dance TD${td.index}`}><div class="slot-edit-title"><span class="slot-kicker">Tap dance {td.index}</span></div><div class="keycap-row">{#each ["Tap", "Hold", "Double tap", "Tap and hold"] as name, i}<Field path={`tap_dance[${td.index}][${i}]`} label={name} />{/each}</div><label class="slot-number">Tapping term <input type="number" min="0" max="65535" value={td.tappingTerm} onchange={(e) => onsave({ ...td, tappingTerm: Number(e.currentTarget.value) })} /> ms</label></article>{/if}{/each}</div></section>


