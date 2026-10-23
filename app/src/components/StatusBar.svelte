<script lang="ts">
  import type { Mode } from "../lib/modes";

  interface Props {
    mode: Mode;
    layer: number;
    path: string | null;
    dirty: boolean;
    selectionSize: number;
    message: string;
  }

  let { mode, layer, path, dirty, selectionSize, message }: Props = $props();
</script>

<footer class="status">
  <span class="mode">{mode.toUpperCase()}</span>
  <span>L{layer}</span>
  <span>{path ?? "no file"}{dirty ? " ●" : ""}</span>
  {#if selectionSize > 0}<span>{selectionSize} selected</span>{/if}
  <span class="message">{message}</span>
</footer>

<style>
  .status {
    display: flex;
    gap: 16px;
    padding: 4px 10px;
    font-size: 12px;
    border-top: 1px solid var(--key-border);
  }
  .mode {
    font-weight: 700;
  }
  .message {
    color: var(--muted);
  }
</style>
