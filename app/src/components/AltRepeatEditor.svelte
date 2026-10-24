<script lang="ts">
  import type { AltRepeatDto } from "../lib/types";
  import Field from "./Field.svelte";
  import FeatureSlotList from "./FeatureSlotList.svelte";
  import type { OpDto } from "../lib/types";
  import { ui } from "../lib/state.svelte";
  import { MOD_BITS, ALT_REPEAT_OPTION_BITS, hasBit, toggleBit } from "../lib/bits";
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
  $effect(() => {
    if (!altRepeatKeys.some((slot) => slot.index === selected)) selected = altRepeatKeys[0]?.index ?? 0;
  });
  function pickSlot(index: number): void {
    selected = index;
    ui.inspectorPaths = [];
  }

  function slotSet(ar: AltRepeatDto): boolean {
    return ar.keycode !== "KC_NO" || ar.altKeycode !== "KC_NO" || ar.allowedMods !== 0 || ar.options !== 0;
  }
  function altOptionLabel(name: string): string {
    return ({ "default to this alt key": "Use as default alternate", bidirectional: "Works both ways", "ignore mod handedness": "Ignore left/right", enabled: "Enabled" } as Record<string, string>)[name] || name;
  }
  function altOptionTitle(name: string): string {
    return ({ "default to this alt key": "Used when no other alt repeat entry matches the last key", bidirectional: "Alt Repeat after the alternate key sends the original key", "ignore mod handedness": "Treat left and right modifiers as the same", enabled: "Turn this entry on or off" } as Record<string, string>)[name] || "";
  }
  function altModLabel(name: string): string {
    return ({ LCTL: "L Ctrl", LSFT: "L Shift", LALT: "L Alt", LGUI: "L Gui", RCTL: "R Ctrl", RSFT: "R Shift", RALT: "R Alt", RGUI: "R Gui" } as Record<string, string>)[name] || name;
  }
  function slotText(ar: AltRepeatDto): string {
    if (!slotSet(ar)) return "";
    const key = ar.keycode === "KC_NO" ? "" : ui.view?.keycodes["alt_repeat_key[" + ar.index + "].keycode"]?.label || ar.keycode;
    const alt = ar.altKeycode === "KC_NO" ? "" : ui.view?.keycodes["alt_repeat_key[" + ar.index + "].alt_keycode"]?.label || ar.altKeycode;
    return (key || "…") + " → " + (alt || "…");
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

<section class="slot-editor" data-slot-editor="altRepeat"><!-- svelte-ignore a11y_no_noninteractive_element_interactions --><FeatureSlotList kind="altRepeat" label="Alt repeat" perColumn={16} selected={selected} onselect={pickSlot} onoperation={onoperation} items={altRepeatKeys.map((slot) => ({ index: slot.index, set: slotSet(slot), text: slotText(slot) }))} /><div class="slot-form">{#each altRepeatKeys as ar (ar.index)}{#if ar.index === selected}<article class="slot-edit-panel" aria-label={`Editing alternate repeat AR${ar.index}`}><div class="slot-edit-title"><span class="slot-kicker">Alt repeat {ar.index}</span></div><div class="keycap-row"><Field path={`alt_repeat_key[${ar.index}].keycode`} label="Key" /><span class="keycap-arrow" aria-hidden="true">→</span><Field path={`alt_repeat_key[${ar.index}].alt_keycode`} label="Alternate" /></div><div class="override-body alt-body">{#each masks as mask, index}<fieldset class={index === 0 ? "modifier-toggles" : "override-options"}><legend>{mask.name}</legend>{#each mask.bits as bit}<label title={index === 0 ? undefined : altOptionTitle(bit.name)}><input type="checkbox" checked={hasBit(index === 0 ? ar.allowedMods : ar.options, bit.bit)} onchange={() => onsave({ ...ar, ...(index === 0 ? { allowedMods: toggleBit(ar.allowedMods, bit.bit) } : { options: toggleBit(ar.options, bit.bit) }) })} />{index === 0 ? altModLabel(bit.name) : altOptionLabel(bit.name)}</label>{/each}</fieldset>{/each}</div></article>{/if}{/each}</div></section>


