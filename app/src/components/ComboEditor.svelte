<script lang="ts">
  import type { ComboDto } from "../lib/types";
  import { ui } from "../lib/state.svelte";
  import Field from "./Field.svelte";
  let { combos, onrename }: { combos: ComboDto[]; onrename: (index: number) => void } = $props();
</script>

<section>
  <h2>Combos</h2>
  <p>Triggers are keycodes. Matching positions share one trigger.</p>
  {#each combos as combo (combo.index)}<details
      ontoggle={(e) => {
        if (!e.currentTarget.open) {
          if (ui.inspectorPaths[0]?.startsWith(`combo[${combo.index}]`)) ui.inspectorPaths = [];
          if (ui.comboEditing === combo.index) ui.comboEditing = null;
        }
      }}
    >
      <summary
        >{ui.view?.names.combos[combo.index] || `Combo ${combo.index}`} · {combo.keys
          .map((key, i) =>
            key === "KC_NO" ? "" : ui.view?.keycodes[`combo[${combo.index}][${i}]`]?.label || key,
          )
          .filter(Boolean)
          .join(" + ") || "Empty"}</summary
      >
      <button onclick={() => onrename(combo.index)}>Name combo</button>
      <button
        class:active={ui.comboEditing === combo.index}
        onclick={() => {
          ui.comboEditing = ui.comboEditing === combo.index ? null : combo.index;
        }}
        >{ui.comboEditing === combo.index
          ? "Finish board triggers"
          : "Choose triggers on board"}</button
      >
      {#each combo.keys as _, i}<Field
          path={`combo[${combo.index}][${i}]`}
          label={`Trigger ${i + 1}`}
        />{/each}<Field path={`combo[${combo.index}][4]`} label="Output" />
    </details>{/each}
</section>

<style>
  h2 {
    font-size: 15px;
  }
  p {
    font-size: 12px;
    color: var(--muted);
  }
  details {
    padding: 8px 0;
    border-bottom: 1px solid var(--key-border);
  }
  summary {
    cursor: pointer;
    padding-bottom: 8px;
  }
  details :global(.field) {
    margin-top: 4px;
  }
  .active {
    background: var(--selected);
  }
</style>
