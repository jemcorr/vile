<script lang="ts">
  import type { DiffLineDto, Names } from "../lib/types";
  let {
    layerCount,
    current,
    diff,
    onselect,
    onmenu,
    names,
  }: {
    layerCount: number;
    current: number;
    diff: DiffLineDto[];
    onselect: (layer: number) => void;
    onmenu: (layer: number, x: number, y: number) => void;
    names: Names;
  } = $props();
  const changedLayers = $derived(new Set(diff.map((d) => d.layer).filter((l) => l !== null)));
</script>

<nav class="layers" aria-label="Layers">
  {#each Array.from({ length: layerCount }, (_, i) => i) as layer}<button
      class:current={layer === current}
      aria-current={layer === current ? "true" : undefined}
      title={`Layer ${layer}${layer < 10 ? ` · ${layer}` : ""}${changedLayers.has(layer) ? " · changed" : ""}`}
      onclick={() => onselect(layer)}
      oncontextmenu={(e) => {
        e.preventDefault();
        onmenu(layer, e.clientX, e.clientY);
      }}
      onkeydown={(e) => {
        if ((e.key === "F10" && e.shiftKey) || e.key === "ContextMenu") {
          e.preventDefault();
          const rect = e.currentTarget.getBoundingClientRect();
          onmenu(layer, rect.left, rect.bottom);
        }
      }}
      >{names.layers[layer] || `L${layer}`}{#if changedLayers.has(layer)}<i class="change-dot tab-change" aria-hidden="true"></i>{/if}<small
        >{layer < 10 ? layer : ""}</small
      ></button
    >{/each}
</nav>

<style>
  .layers {
    display: flex;
    gap: 4px;
    padding: 6px 16px;
    overflow-x: auto;
    border-bottom: 1px solid var(--key-border);
  }
  button.current {
    background: var(--selected);
  }
  small {
    margin-left: 8px;
    color: var(--muted);
    font-size: 10px;
  }
</style>
