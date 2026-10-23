<script lang="ts">
  import type { AltRepeatDto } from "../lib/types";
  import { ALT_REPEAT_OPTION_BITS, MOD_BITS, hasBit, toggleBit, type BitName } from "../lib/bits";

  interface Props {
    altRepeatKeys: AltRepeatDto[];
    onsave: (ar: AltRepeatDto) => void;
  }

  const { altRepeatKeys, onsave }: Props = $props();

  let editing: number | null = $state(null);
  let draft: AltRepeatDto = $state({
    index: 0,
    keycode: "",
    altKeycode: "",
    allowedMods: 0,
    options: 0,
  });

  function startEdit(ar: AltRepeatDto): void {
    editing = ar.index;
    draft = { ...ar };
  }

  function apply(): void {
    if (editing === null) return;
    onsave({ ...draft, index: editing });
    editing = null;
  }
</script>

{#snippet bitBoxes(title: string, value: number, names: BitName[], set: (next: number) => void)}
  <fieldset>
    <legend>{title}</legend>
    {#each names as b (b.bit)}
      <label>
        <input type="checkbox" checked={hasBit(value, b.bit)} onchange={() => set(toggleBit(value, b.bit))} />
        {b.name}
      </label>
    {/each}
  </fieldset>
{/snippet}

<section>
  <h2>Alt repeat keys</h2>
  <ul>
    {#each altRepeatKeys as ar (ar.index)}
      <li>
        {#if editing === ar.index}
          <div class="edit">
            <span>#{ar.index}</span>
            <label>Keycode <input type="text" bind:value={draft.keycode} /></label>
            <label>Alt keycode <input type="text" bind:value={draft.altKeycode} /></label>
            {@render bitBoxes("Allowed mods", draft.allowedMods, MOD_BITS, (next) => (draft.allowedMods = next))}
            {@render bitBoxes("Options", draft.options, ALT_REPEAT_OPTION_BITS, (next) => (draft.options = next))}
            <div class="buttons">
              <button type="button" onclick={apply}>Apply</button>
              <button type="button" onclick={() => (editing = null)}>Cancel</button>
            </div>
          </div>
        {:else}
          <div class="row">
            <span class="index">#{ar.index}</span>
            <span class="keys">{ar.keycode} -&gt; {ar.altKeycode}</span>
            <button type="button" onclick={() => startEdit(ar)}>Edit</button>
          </div>
        {/if}
      </li>
    {/each}
  </ul>
</section>

<style>
  section h2 {
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
  .keys {
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
  .edit fieldset {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 10px;
    border: 1px solid var(--key-border);
    border-radius: 4px;
    padding: 4px 8px;
  }
  .edit input[type="text"] {
    flex: 1;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  }
  .buttons {
    display: flex;
    gap: 6px;
  }
</style>
