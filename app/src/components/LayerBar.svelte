<script lang="ts">
  import type { DiffLineDto } from "../lib/types";

  interface Props {
    layerCount: number;
    current: number;
    diff: DiffLineDto[];
    onselect: (layer: number) => void;
  }

  let { layerCount, current, diff, onselect }: Props = $props();

  const changedLayers = $derived(new Set(diff.map((d) => d.layer).filter((l) => l !== null)));
</script>

<nav class="layers" aria-label="layers">
  {#each Array.from({ length: layerCount }, (_, i) => i) as layer (layer)}
    <button class:current={layer === current} onclick={() => onselect(layer)}>
      L{layer}{changedLayers.has(layer) ? " •" : ""}
    </button>
  {/each}
</nav>

<style>
  .layers {
    display: flex;
    gap: 4px;
    padding: 6px 8px;
  }
  button {
    font: inherit;
    padding: 4px 10px;
    border: 1px solid var(--key-border);
    border-radius: 6px;
    background: var(--key-bg);
    color: var(--key-fg);
  }
  button.current {
    background: var(--selected);
    font-weight: 600;
  }
</style>
