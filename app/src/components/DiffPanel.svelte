<script lang="ts">
  import type { DiffBase, DiffLineDto } from "../lib/types";
  let {
    diff,
    base,
    onjump,
  }: { diff: DiffLineDto[]; base: DiffBase; onjump: (layer: number, keyId: string) => void } =
    $props();
</script>

<section>
  <h2>{base === "git" ? "Changes vs Git HEAD" : "Pending changes"} ({diff.length})</h2>
  {#if !diff.length}<p>No changes</p>{/if}
  <ul>
    {#each diff as entry, i (i)}<li>
        <button
          class="line"
          onclick={() => {
            if (entry.layer !== null && entry.keyId !== null) onjump(entry.layer, entry.keyId);
          }}
          ><span>{entry.line.split(":")[0]}</span><strong
            >{entry.oldDescription} → {entry.newDescription}</strong
          ><code>{entry.old} → {entry.new}</code></button
        >
      </li>{/each}
  </ul>
</section>

<style>
  h2 {
    font-size: 14px;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .line {
    display: flex;
    flex-direction: column;
    gap: 4px;
    width: 100%;
    text-align: left;
    border: 0;
    border-bottom: 1px solid var(--key-border);
    border-radius: 0;
    background: none;
    padding: 10px 2px;
  }
  .line span,
  .line code {
    color: var(--muted);
    font-size: 11px;
  }
  .line strong {
    font-weight: 400;
    font-size: 13px;
  }
</style>
