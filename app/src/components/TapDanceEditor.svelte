<script lang="ts">
  import { usesOf } from "../lib/slots";
  import type { TapDanceDto } from "../lib/types";
  import Field from "./Field.svelte";
  import FeatureSlotList from "./FeatureSlotList.svelte";
  import type { OpDto } from "../lib/types";
  import { ui } from "../lib/state.svelte";
  import { currentLabels, tapDanceSet, tapDanceText } from "../lib/slots";
  import SlotUses from "./SlotUses.svelte";
  import { changedSlots } from "../lib/changes";
  let { tapDances, onsave, onoperation }: { tapDances: TapDanceDto[]; onsave: (value: TapDanceDto) => void; onoperation: (op: OpDto) => Promise<boolean> } =
    $props();
  let selected = $state(0);
  const changed = $derived(changedSlots(ui.view).tapDance);
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

</script>

<section class="slot-editor" data-slot-editor="tapDance"><!-- svelte-ignore a11y_no_noninteractive_element_interactions --><FeatureSlotList kind="tapDance" label="Tap dance" perColumn={16} selected={selected} onselect={pickSlot} onoperation={onoperation} items={tapDances.map((slot) => ({ index: slot.index, set: tapDanceSet(slot), text: tapDanceText(slot, currentLabels(ui.view)), changed: changed.has(slot.index), uses: usesOf(ui.view, "tapDance", slot.index).length }))} /><div class="slot-form">{#each tapDances as td (td.index)}{#if td.index === selected}<article class="slot-edit-panel" aria-label={`Editing tap dance TD${td.index}`}><div class="slot-edit-title"><span class="slot-kicker">Tap dance {td.index}</span></div><SlotUses feature="tapDance" index={td.index} /><div class="keycap-row">{#each ["Tap", "Hold", "Double tap", "Tap and hold"] as name, i}<Field path={`tap_dance[${td.index}][${i}]`} label={name} />{/each}</div><label class="slot-number">Tapping term <input type="number" min="0" max="65535" value={td.tappingTerm} onchange={(e) => onsave({ ...td, tappingTerm: Number(e.currentTarget.value) })} /> ms</label></article>{/if}{/each}</div></section>


