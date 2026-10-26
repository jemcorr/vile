<script lang="ts">
  import type { AltRepeatDto } from "../lib/types";
  import Field from "./Field.svelte";
  import FeatureSlotList from "./FeatureSlotList.svelte";
  import type { OpDto } from "../lib/types";
  import { ui } from "../lib/state.svelte";
  import { MOD_BITS, ALT_REPEAT_OPTION_BITS, ALT_REPEAT_OPTION_LABELS, ALT_REPEAT_OPTION_TITLES, MOD_LABELS, hasBit, toggleBit } from "../lib/bits";
  import { altRepeatSet, altRepeatText, currentLabels } from "../lib/slots";
  import { changedSlots } from "../lib/changes";
  const masks = [
    { name: "Allowed modifiers", bits: MOD_BITS },
    { name: "Options", bits: ALT_REPEAT_OPTION_BITS },
  ];
  let {
    altRepeatKeys,
    onsave,
    onoperation,
  }: { altRepeatKeys: AltRepeatDto[]; onsave: (value: AltRepeatDto) => void; onoperation: (op: OpDto) => Promise<boolean> } = $props();
  let selected = $state(0);
  const changed = $derived(changedSlots(ui.view).altRepeatKey);
  $effect(() => {
    if (!altRepeatKeys.some((slot) => slot.index === selected)) selected = altRepeatKeys[0]?.index ?? 0;
  });
  function pickSlot(index: number): void {
    selected = index;
    ui.inspectorPaths = [];
  }
  $effect(() => {
    const reveal = ui.revealSlot;
    if (reveal?.panel === "altRepeat") {
      pickSlot(reveal.index);
      ui.revealSlot = null;
    }
  });

  function altOptionLabel(name: string): string {
    return ALT_REPEAT_OPTION_LABELS[name] ?? name;
  }
  function altOptionTitle(name: string): string {
    return ALT_REPEAT_OPTION_TITLES[name] ?? "";
  }
  function altModLabel(name: string): string {
    return MOD_LABELS[name] ?? name;
  }
</script>

<section class="slot-editor" data-slot-editor="altRepeat"><!-- svelte-ignore a11y_no_noninteractive_element_interactions --><FeatureSlotList kind="altRepeat" label="Alt repeat" perColumn={16} selected={selected} onselect={pickSlot} onoperation={onoperation} items={altRepeatKeys.map((slot) => ({ index: slot.index, set: altRepeatSet(slot), text: altRepeatText(slot, currentLabels(ui.view)), changed: changed.has(slot.index) }))} /><div class="slot-form">{#each altRepeatKeys as ar (ar.index)}{#if ar.index === selected}<article class="slot-edit-panel" aria-label={`Editing alternate repeat AR${ar.index}`}><div class="slot-edit-title"><span class="slot-kicker">Alt repeat {ar.index}</span></div><div class="keycap-row"><Field path={`alt_repeat_key[${ar.index}].keycode`} label="Key" /><span class="keycap-arrow" aria-hidden="true">→</span><Field path={`alt_repeat_key[${ar.index}].alt_keycode`} label="Alternate" /></div><div class="override-body alt-body">{#each masks as mask, index}<fieldset class={index === 0 ? "modifier-toggles" : "override-options"}><legend>{mask.name}</legend>{#each mask.bits as bit}<label title={index === 0 ? undefined : altOptionTitle(bit.name)}><input type="checkbox" checked={hasBit(index === 0 ? ar.allowedMods : ar.options, bit.bit)} onchange={() => onsave({ ...ar, ...(index === 0 ? { allowedMods: toggleBit(ar.allowedMods, bit.bit) } : { options: toggleBit(ar.options, bit.bit) }) })} />{index === 0 ? altModLabel(bit.name) : altOptionLabel(bit.name)}</label>{/each}</fieldset>{/each}</div></article>{/if}{/each}</div></section>


