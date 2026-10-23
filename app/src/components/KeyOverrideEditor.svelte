<script lang="ts">
  import type { KeyOverrideDto } from "../lib/types";
  import { KEY_OVERRIDE_OPTION_BITS, MOD_BITS, hasBit, modNames, toggleBit, type BitName } from "../lib/bits";

  interface Props {
    keyOverrides: KeyOverrideDto[];
    layerCount: number;
    onsave: (ko: KeyOverrideDto) => void;
  }

  const { keyOverrides, layerCount, onsave }: Props = $props();

  let editing: number | null = $state(null);
  let draft: KeyOverrideDto = $state({
    index: 0,
    trigger: "",
    replacement: "",
    layers: 0,
    triggerMods: 0,
    negativeModMask: 0,
    suppressedMods: 0,
    options: 0,
  });

  const layerNumbers = $derived(Array.from({ length: layerCount }, (_, n) => n));

  function startEdit(ko: KeyOverrideDto): void {
    editing = ko.index;
    draft = { ...ko };
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
  <h2>Key overrides</h2>
  <ul>
    {#each keyOverrides as ko (ko.index)}
      <li>
        {#if editing === ko.index}
          <div class="edit">
            <span>#{ko.index}</span>
            <label>Trigger <input type="text" bind:value={draft.trigger} /></label>
            <label>Replacement <input type="text" bind:value={draft.replacement} /></label>
            <fieldset>
              <legend>Layers</legend>
              {#each layerNumbers as n (n)}
                <label>
                  <input
                    type="checkbox"
                    checked={hasBit(draft.layers, n)}
                    onchange={() => (draft.layers = toggleBit(draft.layers, n))}
                  />
                  L{n}
                </label>
              {/each}
            </fieldset>
            {@render bitBoxes("Trigger mods", draft.triggerMods, MOD_BITS, (next) => (draft.triggerMods = next))}
            {@render bitBoxes("Negative mod mask", draft.negativeModMask, MOD_BITS, (next) => (draft.negativeModMask = next))}
            {@render bitBoxes("Suppressed mods", draft.suppressedMods, MOD_BITS, (next) => (draft.suppressedMods = next))}
            {@render bitBoxes("Options", draft.options, KEY_OVERRIDE_OPTION_BITS, (next) => (draft.options = next))}
            <div class="buttons">
              <button type="button" onclick={apply}>Apply</button>
              <button type="button" onclick={() => (editing = null)}>Cancel</button>
            </div>
          </div>
        {:else}
          <div class="row">
            <span class="index">#{ko.index}</span>
            <span class="keys">{ko.trigger} -&gt; {ko.replacement}</span>
            <span>{modNames(ko.triggerMods)}</span>
            <span>{hasBit(ko.options, 7) ? "on" : "off"}</span>
            <button type="button" onclick={() => startEdit(ko)}>Edit</button>
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
