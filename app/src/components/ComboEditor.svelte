<script lang="ts">
  import type { ComboDto } from "../lib/types";
  import { ui } from "../lib/state.svelte";
  import Field from "./Field.svelte";
  let { combos, onrename }: { combos: ComboDto[]; onrename: (index: number) => void } = $props();
  import MiniBoard from "./MiniBoard.svelte";
  let selected = $state(0);
  $effect(() => {
    if (!combos.some((slot) => slot.index === selected)) selected = combos[0]?.index ?? 0;
  });
  function pickSlot(index: number): void {
    selected = index;
    ui.inspectorPaths = [];
    ui.comboEditing = null;
  }

  function slotSet(combo: ComboDto): boolean {
    return combo.keys.some((key) => key !== "KC_NO") || combo.output !== "KC_NO";
  }
  function slotText(combo: ComboDto): string {
    if (!slotSet(combo)) return "";
    const name = ui.view?.names.combos[combo.index];
    if (name) return name;
    const triggers = combo.keys
      .map((key, i) => (key === "KC_NO" ? "" : ui.view?.keycodes["combo[" + combo.index + "][" + i + "]"]?.label || key))
      .filter(Boolean)
      .join(" + ");
    const output = combo.output === "KC_NO" ? "" : combo.outputLabel || combo.output;
    return (triggers || "…") + " → " + (output || "…");
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

<section class="slot-editor" data-slot-editor="combo"><!-- svelte-ignore a11y_no_noninteractive_element_interactions --><div class="slot-list" role="group" style="--per-column:16" aria-label="combo slots" onkeydown={(e) => slotKeys(e, 16, pickSlot)}>{#each combos as combo (combo.index)}<button class="slot-cell" data-slot-index={combo.index} data-state={slotSet(combo) ? "set" : "empty"} class:active={selected === combo.index} aria-pressed={selected === combo.index} aria-label={"Combo slot C" + combo.index} title={slotText(combo) || "Empty"} onclick={() => pickSlot(combo.index)}><span class="slot-id">{combo.index}</span><span class="slot-desc">{slotText(combo)}</span></button>{/each}</div><div class="slot-form">{#each combos as combo (combo.index)}{#if combo.index === selected}<article class="slot-edit-panel" aria-label={`Editing combo C${combo.index}`}><div class="slot-edit-title"><div><span class="slot-kicker">Combo {combo.index}</span>{#if ui.view?.names.combos[combo.index]}<h3>{ui.view.names.combos[combo.index]}</h3>{/if}</div><button class="quiet-button" onclick={() => onrename(combo.index)}>Rename</button></div><p class="slot-edit-summary">{combo.keys.map((key, i) => key === "KC_NO" ? "" : ui.view?.keycodes[`combo[${combo.index}][${i}]`]?.label || key).filter(Boolean).join(" + ") || "No triggers"} <span aria-hidden="true">→</span> {combo.outputLabel || "No output"}</p><div class="slot-actions"><button class:active={ui.comboEditing === combo.index} onclick={() => { ui.comboEditing = ui.comboEditing === combo.index ? null : combo.index; if (ui.comboEditing !== null) window.dispatchEvent(new Event("canvas-board-triggers")); }}>{ui.comboEditing === combo.index ? "Finish board triggers" : "Choose triggers on board"}</button></div><div class="keycap-row">{#each combo.keys as _, i}<Field path={`combo[${combo.index}][${i}]`} label={`Trigger ${i + 1}`} />{/each}<span class="keycap-arrow" aria-hidden="true">→</span><Field path={`combo[${combo.index}][4]`} label="Output" /></div><MiniBoard index={combo.index} /></article>{/if}{/each}</div></section>


