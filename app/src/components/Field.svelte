<script lang="ts">
  import { ui } from "../lib/state.svelte";
  let { path, label }: { path: string; label: string } = $props();
  const key = $derived(ui.view?.keycodes[path]);
</script>

<button
  class="field"
  class:active={ui.inspectorPaths.length === 1 && ui.inspectorPaths[0] === path}
  aria-pressed={ui.inspectorPaths.length === 1 && ui.inspectorPaths[0] === path}
  title={key ? `${key.tooltip}\n${key.wire}` : path}
  onclick={() => {
    ui.inspectorPaths = [path];
    ui.baseTarget = false;
  }}
>
  <span>{label}</span><strong>{key?.label || key?.description || "No key"}</strong><small
    >{key?.description ?? ""}</small
  ><code>{key?.wire ?? ""}</code>
</button>

<style>
  .field {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    width: 100%;
    text-align: left;
  }
  .active {
    background: var(--selected);
  }
  .field > span {
    color: var(--muted);
    font-size: 12px;
    min-width: 55px;
  }
  .field > strong {
    font-size: 15px;
    font-weight: 500;
  }
  .field > small,
  .field > code {
    color: var(--muted);
    font-size: 11px;
  }
</style>
