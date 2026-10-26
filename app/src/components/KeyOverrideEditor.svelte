<script lang="ts">
  import { enabledOf } from "../lib/slots";
  import type { KeyOverrideDto } from "../lib/types";
  import Field from "./Field.svelte";
  import FeatureSlotList from "./FeatureSlotList.svelte";
  import type { OpDto } from "../lib/types";
  import { ui } from "../lib/state.svelte";
  import { MOD_BITS, KEY_OVERRIDE_OPTION_BITS, MOD_LABELS, KEY_OVERRIDE_OPTION_LABELS, KEY_OVERRIDE_OPTION_TITLES, hasBit, toggleBit } from "../lib/bits";
  import { currentLabels, keyOverrideSet, keyOverrideText } from "../lib/slots";
  import { changedSlots } from "../lib/changes";
  const canvasBitTitle = (name: string): string => KEY_OVERRIDE_OPTION_TITLES[name] ?? "";
  const canvasBitLabel = (name: string): string => MOD_LABELS[name] ?? KEY_OVERRIDE_OPTION_LABELS[name] ?? name;
  const modFields: [string, "triggerMods" | "negativeModMask" | "suppressedMods"][] = [
    ["Trigger mods", "triggerMods"],
    ["Negative mods", "negativeModMask"],
    ["Suppress mods", "suppressedMods"],
  ];
  let {
    keyOverrides,
    layerCount,
    onsave,
    onoperation,
  }: {
    keyOverrides: KeyOverrideDto[];
    layerCount: number;
    onsave: (value: KeyOverrideDto) => void;
    onoperation: (op: OpDto) => Promise<boolean>;
  } = $props();
  let selected = $state(0);
  const changed = $derived(changedSlots(ui.view).keyOverride);
  $effect(() => {
    if (!keyOverrides.some((slot) => slot.index === selected)) selected = keyOverrides[0]?.index ?? 0;
  });
  function pickSlot(index: number): void {
    selected = index;
    ui.inspectorPaths = [];
  }
  $effect(() => {
    const reveal = ui.revealSlot;
    if (reveal?.panel === "keyOverrides") {
      pickSlot(reveal.index);
      ui.revealSlot = null;
    }
  });

</script>

<section class="slot-editor" data-slot-editor="keyOverride"><!-- svelte-ignore a11y_no_noninteractive_element_interactions --><FeatureSlotList kind="keyOverride" label="Key override" perColumn={16} selected={selected} onselect={pickSlot} onoperation={onoperation} items={keyOverrides.map((slot) => ({ index: slot.index, set: keyOverrideSet(slot), text: keyOverrideText(slot, currentLabels(ui.view)), changed: changed.has(slot.index), enabled: enabledOf("keyOverride", slot) }))} /><div class="slot-form">{#each keyOverrides as ko (ko.index)}{#if ko.index === selected}<article class="slot-edit-panel override-card" aria-label={`Editing key override KO${ko.index}`}><div class="slot-edit-title"><span class="slot-kicker">Key override {ko.index}</span></div><div class="override-body"><div class="keycap-row"><Field path={`key_override[${ko.index}].trigger`} label="Trigger" /><span class="keycap-arrow" aria-hidden="true">→</span><Field path={`key_override[${ko.index}].replacement`} label="Replace" /></div><fieldset class="layer-toggles"><legend>Layers</legend>{#each Array.from({ length: Math.min(layerCount, 16) }, (_, i) => i) as i}<label><input type="checkbox" checked={hasBit(ko.layers, i)} onchange={() => onsave({ ...ko, layers: toggleBit(ko.layers, i) })} />L{i}</label>{/each}</fieldset>{#each modFields as [name, field]}<fieldset class="modifier-toggles"><legend>{name}</legend>{#each MOD_BITS as bit}<label><input type="checkbox" checked={hasBit(ko[field as "triggerMods"], bit.bit)} onchange={() => onsave({ ...ko, [field]: toggleBit(ko[field as "triggerMods"], bit.bit) })} />{canvasBitLabel(bit.name)}</label>{/each}</fieldset>{/each}<fieldset class="override-options"><legend>Options</legend>{#each KEY_OVERRIDE_OPTION_BITS as bit}<label title={canvasBitTitle(bit.name)}><input type="checkbox" checked={hasBit(ko.options, bit.bit)} onchange={() => onsave({ ...ko, options: toggleBit(ko.options, bit.bit) })} />{canvasBitLabel(bit.name)}</label>{/each}</fieldset></div></article>{/if}{/each}</div></section>


