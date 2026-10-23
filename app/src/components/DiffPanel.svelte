<script lang="ts">
  import type { DiffBase, DiffLineDto } from "../lib/types";

  interface Props {
    diff: DiffLineDto[];
    base: DiffBase;
    onjump: (layer: number, keyId: string) => void;
  }

  const { diff, base, onjump }: Props = $props();

  const heading = $derived(
    base === "git"
      ? `Changes vs git HEAD (${diff.length})`
      : `Changes vs file on disk (${diff.length})`,
  );

  function jump(entry: DiffLineDto): void {
    if (entry.layer !== null && entry.keyId !== null) {
      onjump(entry.layer, entry.keyId);
    }
  }
</script>

<section class="panel">
  <h2>{heading}</h2>
  {#if diff.length === 0}
    <p>No changes</p>
  {:else}
    <ul>
      {#each diff as entry, i (i)}
        <li>
          <button type="button" class="line" onclick={() => jump(entry)}>{entry.line}</button>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .panel h2 {
    font-size: 14px;
    margin: 0 0 8px;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .line {
    display: block;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    padding: 2px 4px;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 12px;
    color: inherit;
    cursor: pointer;
  }
  .line:hover {
    background: var(--selected);
  }
</style>
