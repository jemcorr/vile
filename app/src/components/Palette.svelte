<script lang="ts">
  import {
    filterCommands,
    parseLayerArg,
    commandReason,
    type PaletteCommand,
  } from "../lib/palette";

  import { ui } from "../lib/state.svelte";
  import { focusScope } from "../lib/focus";

  interface Props {
    layerCount: number;
    onrun: (id: string, arg: number | null) => void;
    onclose: () => void;
  }

  const { layerCount, onrun, onclose }: Props = $props();

  let query = $state("");
  let stage: "command" | "layer" = $state("command");
  let chosen: PaletteCommand | null = $state(null);
  let active = $state(0);
  let error = $state("");
  let input: HTMLInputElement | undefined = $state();

  const list = $derived(filterCommands(query));

  $effect(() => {
    void stage;
    input?.focus();
  });

  function choose(command: PaletteCommand): void {
    if (command.arg === "none") {
      onrun(command.id, null);
      return;
    }
    chosen = command;
    stage = "layer";
    query = "";
    error = "";
    active = 0;
  }

  function submitLayer(): void {
    if (chosen === null) return;
    const n = parseLayerArg(query, layerCount);
    if (n === null) {
      error = `enter a layer number from 0 to ${layerCount - 1}`;
      return;
    }
    onrun(chosen.id, n);
  }

  function keydown(e: KeyboardEvent): void {
    if (e.key === "Escape") {
      e.preventDefault();
      onclose();
      return;
    }
    if (stage === "command") {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        active = Math.max(Math.min(active + 1, list.length - 1), 0);
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        active = Math.max(active - 1, 0);
      } else if (e.key === "Enter") {
        e.preventDefault();
        const command = list[active];
        if (command) choose(command);
      }
    } else if (e.key === "Enter") {
      e.preventDefault();
      submitLayer();
    }
  }
</script>

<div class="overlay">
  <div
    class="palette"
    role="dialog"
    aria-modal="true"
    aria-label="command palette"
    tabindex="-1"
    use:focusScope={onclose}
  >
    {#if stage === "command"}
      <input
        bind:this={input}
        bind:value={query}
        oninput={() => (active = 0)}
        onkeydown={keydown}
        type="text"
        spellcheck="false"
        autocomplete="off"
        aria-label="command"
        placeholder="Type a command"
      />
      <ul>
        {#each list as command, i (command.id)}
          <li>
            <button
              type="button"
              class:active={i === active}
              disabled={!!commandReason(command.id, ui.view, ui.selection.length > 0)}
              title={commandReason(command.id, ui.view, ui.selection.length > 0)}
              onclick={() => choose(command)}
            >
              {command.title}
              {command.shortcut}
            </button>
            {#if commandReason(command.id, ui.view, ui.selection.length > 0)}<small
                >{commandReason(command.id, ui.view, ui.selection.length > 0)}</small
              >{/if}
          </li>
        {/each}
      </ul>
    {:else}
      <p class="chosen">{chosen?.title ?? ""}</p>
      <input
        bind:this={input}
        bind:value={query}
        onkeydown={keydown}
        type="text"
        spellcheck="false"
        autocomplete="off"
        aria-label="layer"
        placeholder={`Layer number (0 to ${layerCount - 1})`}
      />
      {#if error !== ""}
        <p role="alert">{error}</p>
      {/if}
    {/if}
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 10vh;
    background: rgba(0, 0, 0, 0.12);
    z-index: 10;
  }
  .palette {
    width: min(560px, 90vw);
    max-height: 70vh;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    background: var(--bg);
    border: 1px solid var(--key-border);
    border-radius: 8px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
  }
  .palette input {
    font: inherit;
    padding: 6px 8px;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    overflow-y: auto;
  }
  ul button {
    display: block;
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    padding: 4px 8px;
    font: inherit;
    color: inherit;
    cursor: pointer;
  }
  ul button.active {
    background: var(--selected);
  }
  .chosen {
    margin: 0;
    font-weight: 600;
  }
  p[role="alert"] {
    color: var(--changed);
    margin: 0;
  }
</style>
