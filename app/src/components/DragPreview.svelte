<script lang="ts">
  import { pointerDrag } from "../lib/pointer-drag.svelte";

  const featureDrag = $derived(pointerDrag.current?.payload.kind === "featureSlot");
  const action = $derived(pointerDrag.current?.altKey ? "Copy" : "Swap");
</script>

{#if pointerDrag.current}
  <div
    class="drag-preview"
    class:feature-drag={featureDrag}
    style:left={`${Math.max(8, Math.min(pointerDrag.current.clientX + 14, window.innerWidth - 348))}px`}
    style:top={`${Math.max(8, Math.min(pointerDrag.current.clientY + 14, window.innerHeight - 56))}px`}
    title={pointerDrag.current.label.replaceAll("\n", " ")}
    aria-hidden="true"
  >
    <span class="drag-label">{pointerDrag.current.label.replaceAll("\n", " ")}</span>
    {#if featureDrag}<span class="drag-action" class:copy={pointerDrag.current.altKey}>{action}</span>{/if}
  </div>
{/if}

<style>
  .drag-preview {
    position: fixed;
    z-index: 1000;
    display: flex;
    align-items: center;
    gap: 10px;
    width: max-content;
    max-width: min(340px, calc(100vw - 16px));
    min-height: 34px;
    padding: 6px 9px;
    border: 1px solid var(--cursor);
    border-radius: 8px;
    background: var(--bg);
    color: var(--fg);
    box-shadow: 0 5px 18px #0005;
    pointer-events: none;
    font-size: 12px;
  }
  .drag-label {
    overflow: hidden;
    min-width: 0;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .drag-action {
    flex: none;
    padding: 2px 6px;
    border-radius: 5px;
    background: color-mix(in srgb, var(--cursor) 24%, var(--bg));
    color: var(--fg);
    font-size: 10px;
    font-weight: 650;
  }
  .drag-action.copy {
    background: color-mix(in srgb, var(--changed) 26%, var(--bg));
  }
</style>
