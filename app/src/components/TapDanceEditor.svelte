<script lang="ts">
  import type { TapDanceDto } from "../lib/types";

  interface Props {
    tapDances: TapDanceDto[];
    onsave: (td: TapDanceDto) => void;
  }

  const { tapDances, onsave }: Props = $props();

  let editing: number | null = $state(null);
  let draft: TapDanceDto = $state({
    index: 0,
    onTap: "",
    onHold: "",
    onDoubleTap: "",
    onTapHold: "",
    tappingTerm: 0,
  });

  function startEdit(td: TapDanceDto): void {
    editing = td.index;
    draft = { ...td };
  }

  function apply(): void {
    if (editing === null) return;
    onsave({ ...draft, index: editing });
    editing = null;
  }
</script>

<section>
  <h2>Tap dances</h2>
  <ul>
    {#each tapDances as td (td.index)}
      <li>
        {#if editing === td.index}
          <div class="edit">
            <span>TD({td.index})</span>
            <label>On tap <input type="text" bind:value={draft.onTap} /></label>
            <label>On hold <input type="text" bind:value={draft.onHold} /></label>
            <label>On double tap <input type="text" bind:value={draft.onDoubleTap} /></label>
            <label>On tap and hold <input type="text" bind:value={draft.onTapHold} /></label>
            <label>Tapping term (ms) <input type="number" min="0" max="65535" bind:value={draft.tappingTerm} /></label>
            <div class="buttons">
              <button type="button" onclick={apply}>Apply</button>
              <button type="button" onclick={() => (editing = null)}>Cancel</button>
            </div>
          </div>
        {:else}
          <div class="row">
            <span class="index">TD({td.index})</span>
            <span class="keys">{td.onTap} / {td.onHold} / {td.onDoubleTap} / {td.onTapHold}</span>
            <span>{td.tappingTerm} ms</span>
            <button type="button" onclick={() => startEdit(td)}>Edit</button>
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
  .edit input {
    flex: 1;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  }
  .buttons {
    display: flex;
    gap: 6px;
  }
</style>
