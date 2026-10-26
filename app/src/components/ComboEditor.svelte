<script lang="ts">
  import type { ComboDto } from "../lib/types";
  import { ui } from "../lib/state.svelte";
  import { comboSet, comboText, currentLabels } from "../lib/slots";
  import { changedSlots } from "../lib/changes";
  import Field from "./Field.svelte";
  import FeatureSlotList from "./FeatureSlotList.svelte";
  import type { OpDto } from "../lib/types";
  let { combos, onrename, onoperation }: { combos: ComboDto[]; onrename: (index: number) => void; onoperation: (op: OpDto) => Promise<boolean> } = $props();
  import MiniBoard from "./MiniBoard.svelte";
  let selected = $state(0);
  const changed = $derived(changedSlots(ui.view).combo);
  $effect(() => {
    if (!combos.some((slot) => slot.index === selected)) selected = combos[0]?.index ?? 0;
  });
  function pickSlot(index: number): void {
    selected = index;
    ui.inspectorPaths = [];
    ui.comboEditing = null;
  }
  $effect(() => {
    const reveal = ui.revealSlot;
    if (reveal?.panel === "combos") {
      pickSlot(reveal.index);
      ui.revealSlot = null;
    }
  });

</script>

<section class="slot-editor" data-slot-editor="combo"><!-- svelte-ignore a11y_no_noninteractive_element_interactions --><FeatureSlotList kind="combo" label="Combo" perColumn={16} selected={selected} onselect={pickSlot} onoperation={onoperation} items={combos.map((slot) => ({ index: slot.index, set: comboSet(slot), text: comboText(slot, currentLabels(ui.view), ui.view?.names.combos[slot.index]), changed: changed.has(slot.index) }))} /><div class="slot-form">{#each combos as combo (combo.index)}{#if combo.index === selected}<article class="slot-edit-panel" aria-label={`Editing combo C${combo.index}`}><div class="slot-edit-title"><div><span class="slot-kicker">Combo {combo.index}</span>{#if ui.view?.names.combos[combo.index]}<h3>{ui.view.names.combos[combo.index]}</h3>{/if}</div><button class="quiet-button" onclick={() => onrename(combo.index)}>Rename</button></div><p class="slot-edit-summary">{combo.keys.map((key, i) => key === "KC_NO" ? "" : ui.view?.keycodes[`combo[${combo.index}][${i}]`]?.label || key).filter(Boolean).join(" + ") || "No triggers"} <span aria-hidden="true">→</span> {combo.outputLabel || "No output"}</p><div class="slot-actions"><button class:active={ui.comboEditing === combo.index} onclick={() => { ui.comboEditing = ui.comboEditing === combo.index ? null : combo.index; if (ui.comboEditing !== null) window.dispatchEvent(new Event("canvas-board-triggers")); }}>{ui.comboEditing === combo.index ? "Finish board triggers" : "Choose triggers on board"}</button></div><div class="keycap-row">{#each combo.keys as _, i}<Field path={`combo[${combo.index}][${i}]`} label={`Trigger ${i + 1}`} />{/each}<span class="keycap-arrow" aria-hidden="true">→</span><Field path={`combo[${combo.index}][4]`} label="Output" /></div><MiniBoard index={combo.index} /></article>{/if}{/each}</div></section>


