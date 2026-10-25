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
          {@const disabledReason = commandReason(command.id, ui.view, ui.selection.length > 0)}
          <li>
            <button
              type="button"
              class:active={i === active}
              disabled={!!disabledReason}
              title={disabledReason}
              aria-describedby={disabledReason ? `palette-reason-${command.id}` : undefined}
              onclick={() => choose(command)}
            >
              <span class="command-details">
                <span class="command-title">{command.title}</span>
                {#if disabledReason}<small id={`palette-reason-${command.id}`} class="command-reason">{disabledReason}</small>{/if}
              </span>
              {#if command.shortcut}<kbd>{command.shortcut}</kbd>{/if}
            </button>
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
  li {
    min-width: 0;
  }
  ul button {
    display: flex;
    align-items: center;
    justify-content: flex-start;
    gap: 8px;
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    padding: 4px 8px;
    font: inherit;
    color: inherit;
    cursor: pointer;
  }
  ul button:disabled {
    color: var(--muted);
    cursor: default;
    opacity: 1;
  }
  .command-details {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1 1 auto;
    min-width: 0;
  }
  .command-title {
    flex: 0 0 auto;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .command-reason {
    flex: 0 1 50%;
    max-width: 50%;
    min-width: 0;
    overflow: hidden;
    text-align: left;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--muted);
    font-size: 0.82em;
    font-weight: 400;
  }
  ul button kbd {
    flex: none;
    margin-left: auto;
    white-space: nowrap;
  }
  ul button.active {
    background: var(--selected);
  }
  ul button:disabled.active {
    color: color-mix(in srgb, var(--muted) 50%, var(--fg));
  }
  ul button:disabled.active .command-reason {
    color: inherit;
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
