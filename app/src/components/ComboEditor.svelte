<script lang="ts">
  import type { ComboDto } from "../lib/types";

  interface Props {
    combos: ComboDto[];
    onsave: (index: number, keys: string[], output: string) => void;
  }

  const { combos, onsave }: Props = $props();

  let editing: number | null = $state(null);
  let draft: string[] = $state(["", "", "", ""]);
  let output = $state("");

  function inputsText(combo: ComboDto): string {
    const inputs = combo.keys.filter((k) => k !== "KC_NO");
    return inputs.length > 0 ? inputs.join(" + ") : "(empty)";
  }

  function startEdit(combo: ComboDto): void {
    editing = combo.index;
    draft = [...combo.keys];
    output = combo.output;
  }

  function apply(): void {
    if (editing === null) return;
    onsave(editing, [...draft], output);
    editing = null;
  }
</script>

<section class="combo-editor">
  <h2>Combos</h2>
  <ul>
    {#each combos as combo (combo.index)}
      <li>
        {#if editing === combo.index}
          <div class="edit">
            <span>#{combo.index}</span>
            {#each draft as _, k (k)}
              <label>Key {k + 1} <input type="text" bind:value={draft[k]} /></label>
            {/each}
            <label>Output <input type="text" bind:value={output} /></label>
            <div class="buttons">
              <button type="button" onclick={apply}>Apply</button>
              <button type="button" onclick={() => (editing = null)}>Cancel</button>
            </div>
          </div>
        {:else}
          <div class="row">
            <span class="index">#{combo.index}</span>
            <span class="keys">{inputsText(combo)}</span>
            <span>=&gt;</span>
            <span class="output">{combo.output}</span>
            <button type="button" onclick={() => startEdit(combo)}>Edit</button>
          </div>
        {/if}
      </li>
    {/each}
  </ul>
</section>

<style>
  .combo-editor h2 {
    font-size: 14px;
    margin: 0 0 8px;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  li {
    padding: 6px 0;
    border-bottom: 1px solid var(--key-border);
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
    font-size: 13px;
  }
  .index {
    color: var(--muted);
  }
  .keys,
  .output {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  }
  .edit {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 13px;
  }
  .edit label {
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .edit input {
    flex: 1;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  }
  .buttons {
    display: flex;
    gap: 6px;
  }
</style>
