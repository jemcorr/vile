<script lang="ts">
  import type { MacroActionDto, MacroActionKind, MacroDto } from "../lib/types";

  interface Props {
    macros: MacroDto[];
    onsave: (index: number, actions: MacroActionDto[]) => void;
  }

  const { macros, onsave }: Props = $props();

  interface Row {
    kind: MacroActionKind;
    keys: string;
    text: string;
    delay: number;
  }

  let selected: number | null = $state(null);
  let rows: Row[] = $state([]);

  function summary(macro: MacroDto): string {
    if (macro.actions.length === 0) return "(empty)";
    return macro.actions
      .map((a) => {
        if (a.kind === "text") return `text "${a.text}"`;
        if (a.kind === "delay") return `delay ${a.delay}`;
        return [a.kind, ...a.keys].join(" ");
      })
      .join("; ");
  }

  function toRows(macro: MacroDto): Row[] {
    return macro.actions.map((a) => ({
      kind: a.kind,
      keys: a.keys.join(" "),
      text: a.text,
      delay: a.delay,
    }));
  }

  function select(macro: MacroDto): void {
    selected = macro.index;
    rows = toRows(macro);
  }

  function revert(): void {
    const macro = macros.find((m) => m.index === selected);
    if (macro) rows = toRows(macro);
  }

  function addRow(): void {
    rows = [...rows, { kind: "tap", keys: "", text: "", delay: 0 }];
  }

  function removeRow(index: number): void {
    rows = rows.filter((_, i) => i !== index);
  }

  function apply(): void {
    if (selected === null) return;
    const actions: MacroActionDto[] = rows.map((row) => {
      const usesKeys = row.kind === "tap" || row.kind === "down" || row.kind === "up";
      return {
        kind: row.kind,
        keys: usesKeys ? row.keys.split(/\s+/).filter((k) => k !== "") : [],
        text: row.kind === "text" ? row.text : "",
        delay: row.kind === "delay" ? row.delay : 0,
      };
    });
    onsave(selected, actions);
  }
</script>

<section class="macro-editor">
  <h2>Macros</h2>
  <ul class="list">
    {#each macros as macro (macro.index)}
      <li>
        <button type="button" class:active={selected === macro.index} onclick={() => select(macro)}>
          <span class="name">M{macro.index}</span>
          <span class="summary">{summary(macro)}</span>
        </button>
      </li>
    {/each}
  </ul>

  {#if selected !== null}
    <div class="editor">
      {#each rows as row, i (i)}
        <div class="row">
          <label>
            Action
            <select bind:value={row.kind}>
              <option value="tap">tap</option>
              <option value="down">down</option>
              <option value="up">up</option>
              <option value="text">text</option>
              <option value="delay">delay</option>
            </select>
          </label>
          {#if row.kind === "tap" || row.kind === "down" || row.kind === "up"}
            <label>Keycodes <input type="text" bind:value={row.keys} /></label>
          {:else if row.kind === "text"}
            <label>Text <input type="text" bind:value={row.text} /></label>
          {:else}
            <label>Milliseconds <input type="number" min="0" max="65024" bind:value={row.delay} /></label>
          {/if}
          <button type="button" onclick={() => removeRow(i)}>Remove</button>
        </div>
      {/each}
      <div class="buttons">
        <button type="button" onclick={addRow}>Add action</button>
        <button type="button" onclick={apply}>Apply</button>
        <button type="button" onclick={revert}>Revert</button>
      </div>
    </div>
  {/if}
</section>

<style>
  .macro-editor h2 {
    font-size: 14px;
    margin: 0 0 8px;
  }
  .list {
    list-style: none;
    margin: 0 0 12px;
    padding: 0;
    max-height: 30vh;
    overflow-y: auto;
  }
  .list button {
    display: flex;
    gap: 8px;
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    padding: 3px 4px;
    font: inherit;
    font-size: 13px;
    color: inherit;
    cursor: pointer;
  }
  .list button.active {
    background: var(--selected);
  }
  .name {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    min-width: 3em;
  }
  .summary {
    color: var(--muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .editor {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
    font-size: 13px;
  }
  .row label {
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .row input[type="text"] {
    width: 14em;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  }
  .buttons {
    display: flex;
    gap: 6px;
  }
</style>
