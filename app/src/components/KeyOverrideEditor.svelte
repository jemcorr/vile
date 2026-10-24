<script lang="ts">
  import type { KeyOverrideDto } from "../lib/types";
  import Field from "./Field.svelte";
  import FeatureSlotList from "./FeatureSlotList.svelte";
  import type { OpDto } from "../lib/types";
  import { ui } from "../lib/state.svelte";
  import { MOD_BITS, KEY_OVERRIDE_OPTION_BITS, hasBit, toggleBit } from "../lib/bits";
  const canvasBitTitle = (name: string): string => ({ 'activation: trigger down': "Turns on when the trigger key is pressed while the modifiers are held", 'activation: required mod down': "Turns on when a required modifier is pressed while the trigger is held", 'activation: negative mod up': "Turns on when an excluded modifier is released", 'one mod': "One of the trigger modifiers is enough, instead of all of them", 'no reregister trigger': "When the override ends, don't press the trigger key again", 'no unregister on other key down': "Pressing another key doesn't end the override", enabled: "Turn this override on or off" } as Record<string, string>)[name] || '';
  const canvasBitLabel = (name: string): string => ({ LCTL: 'L Ctrl', LSFT: 'L Shift', LALT: 'L Alt', LGUI: 'L Gui', RCTL: 'R Ctrl', RSFT: 'R Shift', RALT: 'R Alt', RGUI: 'R Gui', 'activation: trigger down': 'Activate on key press', 'activation: required mod down': 'Activate on modifier press', 'activation: negative mod up': 'Activate on excluded release', 'one mod': 'Any one modifier', 'no reregister trigger': "Don't resend the key", 'no unregister on other key down': 'Stay on for other keys', enabled: 'Enabled' } as Record<string, string>)[name] || name;
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
  $effect(() => {
    if (!keyOverrides.some((slot) => slot.index === selected)) selected = keyOverrides[0]?.index ?? 0;
  });
  function pickSlot(index: number): void {
    selected = index;
    ui.inspectorPaths = [];
  }

  function slotSet(ko: KeyOverrideDto): boolean {
    return ko.trigger !== "KC_NO" || ko.replacement !== "KC_NO";
  }
  function slotText(ko: KeyOverrideDto): string {
    if (!slotSet(ko)) return "";
    const mods = MOD_BITS.filter((bit) => hasBit(ko.triggerMods, bit.bit)).map((bit) => bit.name).join("+");
    const trigger = ko.trigger === "KC_NO" ? "" : ui.view?.keycodes["key_override[" + ko.index + "].trigger"]?.label || ko.trigger;
    const replacement = ko.replacement === "KC_NO" ? "" : ui.view?.keycodes["key_override[" + ko.index + "].replacement"]?.label || ko.replacement;
    const left = [mods, trigger].filter(Boolean).join("+");
    return (left || "…") + " → " + (replacement || "…");
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

<section class="slot-editor" data-slot-editor="keyOverride"><!-- svelte-ignore a11y_no_noninteractive_element_interactions --><FeatureSlotList kind="keyOverride" label="Key override" perColumn={16} selected={selected} onselect={pickSlot} onoperation={onoperation} items={keyOverrides.map((slot) => ({ index: slot.index, set: slotSet(slot), text: slotText(slot) }))} /><div class="slot-form">{#each keyOverrides as ko (ko.index)}{#if ko.index === selected}<article class="slot-edit-panel override-card" aria-label={`Editing key override KO${ko.index}`}><div class="slot-edit-title"><span class="slot-kicker">Key override {ko.index}</span></div><div class="override-body"><div class="keycap-row"><Field path={`key_override[${ko.index}].trigger`} label="Trigger" /><span class="keycap-arrow" aria-hidden="true">→</span><Field path={`key_override[${ko.index}].replacement`} label="Replace" /></div><fieldset class="layer-toggles"><legend>Layers</legend>{#each Array.from({ length: Math.min(layerCount, 16) }, (_, i) => i) as i}<label><input type="checkbox" checked={hasBit(ko.layers, i)} onchange={() => onsave({ ...ko, layers: toggleBit(ko.layers, i) })} />L{i}</label>{/each}</fieldset>{#each modFields as [name, field]}<fieldset class="modifier-toggles"><legend>{name}</legend>{#each MOD_BITS as bit}<label><input type="checkbox" checked={hasBit(ko[field as "triggerMods"], bit.bit)} onchange={() => onsave({ ...ko, [field]: toggleBit(ko[field as "triggerMods"], bit.bit) })} />{canvasBitLabel(bit.name)}</label>{/each}</fieldset>{/each}<fieldset class="override-options"><legend>Options</legend>{#each KEY_OVERRIDE_OPTION_BITS as bit}<label title={canvasBitTitle(bit.name)}><input type="checkbox" checked={hasBit(ko.options, bit.bit)} onchange={() => onsave({ ...ko, options: toggleBit(ko.options, bit.bit) })} />{canvasBitLabel(bit.name)}</label>{/each}</fieldset></div></article>{/if}{/each}</div></section>


