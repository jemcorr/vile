<script lang="ts">
  import type { TapDanceDto } from "../lib/types";
  import Field from "./Field.svelte";
  import FeatureSlotList from "./FeatureSlotList.svelte";
  import type { OpDto } from "../lib/types";
  import { ui } from "../lib/state.svelte";
  import { currentLabels, tapDanceSet, tapDanceText } from "../lib/slots";
  let { tapDances, onsave, onoperation }: { tapDances: TapDanceDto[]; onsave: (value: TapDanceDto) => void; onoperation: (op: OpDto) => Promise<boolean> } =
    $props();
  let selected = $state(0);
  $effect(() => {
    if (!tapDances.some((slot) => slot.index === selected)) selected = tapDances[0]?.index ?? 0;
  });
  function pickSlot(index: number): void {
    selected = index;
    ui.inspectorPaths = [];
  }
  $effect(() => {
    const reveal = ui.revealSlot;
    if (reveal?.panel === "tapDances") {
      pickSlot(reveal.index);
      ui.revealSlot = null;
    }
  });

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

<section class="slot-editor" data-slot-editor="tapDance"><!-- svelte-ignore a11y_no_noninteractive_element_interactions --><FeatureSlotList kind="tapDance" label="Tap dance" perColumn={16} selected={selected} onselect={pickSlot} onoperation={onoperation} items={tapDances.map((slot) => ({ index: slot.index, set: tapDanceSet(slot), text: tapDanceText(slot, currentLabels(ui.view)) }))} /><div class="slot-form">{#each tapDances as td (td.index)}{#if td.index === selected}<article class="slot-edit-panel" aria-label={`Editing tap dance TD${td.index}`}><div class="slot-edit-title"><span class="slot-kicker">Tap dance {td.index}</span></div><div class="keycap-row">{#each ["Tap", "Hold", "Double tap", "Tap and hold"] as name, i}<Field path={`tap_dance[${td.index}][${i}]`} label={name} />{/each}</div><label class="slot-number">Tapping term <input type="number" min="0" max="65535" value={td.tappingTerm} onchange={(e) => onsave({ ...td, tappingTerm: Number(e.currentTarget.value) })} /> ms</label></article>{/if}{/each}</div></section>


