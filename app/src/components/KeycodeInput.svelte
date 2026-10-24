<script lang="ts">
  import { untrack } from "svelte";
  import { api, errorText } from "../lib/api";
  import type { SearchHitDto } from "../lib/types";
  import { hitLabel } from "../lib/keycode-hits";

  interface Props {
    initial: string;
    onsubmit: (wire: string) => void;
    oncancel: () => void;
  }

  const { initial, onsubmit, oncancel }: Props = $props();

  let text = $state(untrack(() => initial));
  let hits: SearchHitDto[] = $state([]);
  let active = $state(0);
  let error = $state("");
  let input: HTMLInputElement | undefined = $state();
  let request = 0;

  type Resolved = { ok: true; wire: string } | { ok: false; error: string };

  /** The part after the last "(", or the whole text, without a trailing ")". */
  function token(value: string): string {
    const open = value.lastIndexOf("(");
    const rest = open === -1 ? value : value.slice(open + 1);
    return rest.endsWith(")") ? rest.slice(0, -1) : rest;
  }

  /** The text up to and including the last "(", or "". */
  function prefix(value: string): string {
    const open = value.lastIndexOf("(");
    return open === -1 ? "" : value.slice(0, open + 1);
  }

  async function search(): Promise<void> {
    error = "";
    const id = ++request;
    const found = await api.searchKeycodes(token(text), 12);
    if (id !== request) return;
    hits = found;
    active = 0;
  }

  async function resolve(value: string): Promise<Resolved> {
    try {
      return { ok: true, wire: await api.resolveKeycode(value) };
    } catch (e) {
      return { ok: false, error: errorText(e) };
    }
  }

  async function submit(): Promise<void> {
    const first = await resolve(text);
    if (first.ok) {
      onsubmit(first.wire);
      return;
    }
    error = first.error;
  }

  function complete(hit: SearchHitDto): void {
    text = prefix(text) + hit.insert;
    input?.focus();
    void search();
  }

  async function keydown(e: KeyboardEvent): Promise<void> {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      active = Math.max(Math.min(active + 1, hits.length - 1), 0);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      active = Math.max(active - 1, 0);
    } else if (e.key === "Tab") {
      e.preventDefault();
      const hit = hits[active];
      if (hit) complete(hit);
    } else if (e.key === "Enter") {
      e.preventDefault();
      await submit();
    } else if (e.key === "Escape") {
      e.preventDefault();
      oncancel();
    }
  }

  $effect(() => {
    input?.focus();
    input?.select();
    untrack(() => void search());
  });
</script>

<div class="keycode-input">
  <!-- Always rendered at the reserved height, so the dialog does not resize while typing. -->
  <div class="hits">
    {#each hits as hit, i (i)}
      <button
        type="button"
        title={`${hit.description}\n${hit.name}`}
        class:active={i === active}
        onclick={() => complete(hit)}
      >
        <span class="name">{hit.name}</span>
        <span class="label">{hitLabel(hit.name, hit.label)}</span>
      </button>
    {/each}
  </div>
  <input
    bind:this={input}
    bind:value={text}
    type="text"
    spellcheck="false"
    autocomplete="off"
    aria-label="keycode"
    oninput={() => void search()}
    onkeydown={(e) => void keydown(e)}
  />
  {#if error !== ""}
    <p role="alert">{error}</p>
  {/if}
</div>

<style>
  .keycode-input {
    display: flex;
    flex-direction: column;
    gap: 4px;
    box-sizing: border-box;
    width: min(560px, calc(100vw - 32px));
  }
  /* Two shared columns: every row's code starts at one x, and every label at another. */
  .hits {
    display: grid;
    grid-template-columns: 16ch minmax(0, 1fr);
    align-content: start;
    height: min(40vh, 264px);
    overflow-y: auto;
  }
  .hits button {
    grid-column: 1 / -1;
    display: grid;
    grid-template-columns: subgrid;
    column-gap: 12px;
    align-items: baseline;
    width: 100%;
    min-width: 0;
    text-align: left;
    background: transparent;
    border: none;
    padding: 2px 6px;
    font: inherit;
    color: inherit;
    cursor: pointer;
  }
  .hits button.active {
    background: var(--selected);
  }
  .name,
  .label {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .name {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  }
  .label {
    color: var(--muted);
  }
  p[role="alert"] {
    color: var(--changed);
    margin: 0;
  }
</style>
