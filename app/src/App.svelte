<script lang="ts">
  import { onMount, tick } from "svelte";
  import { api, errorText } from "./lib/api";
  import { setView, ui } from "./lib/state.svelte";
  import Keyboard from "./components/Keyboard.svelte";
  import LayerBar from "./components/LayerBar.svelte";
  import StatusBar from "./components/StatusBar.svelte";

  function onkeyclick(id: string, shift: boolean): void {
    if (shift) {
      ui.selection = ui.selection.includes(id) ? ui.selection.filter((s) => s !== id) : [...ui.selection, id];
    } else {
      ui.cursor = id;
    }
  }

  onMount(async () => {
    try {
      setView(await api.getView());
    } catch (e) {
      ui.message = errorText(e);
    }
    await tick();
    const rendered = document.querySelectorAll("[data-key-id]").length;
    await api.smokeReport(rendered, ui.view?.layerCount ?? 0);
  });
</script>

<div class="app">
  {#if ui.view !== null && ui.view.board !== null}
    <LayerBar
      layerCount={ui.view.layerCount}
      current={ui.layer}
      diff={ui.view.diff}
      onselect={(layer: number) => (ui.layer = layer)}
    />
    <main class="board">
      <Keyboard
        keys={ui.view.board.keys}
        cells={ui.view.layers[ui.layer] ?? []}
        cursor={ui.cursor}
        selection={ui.selection}
        {onkeyclick}
      />
    </main>
  {:else}
    <main class="empty">
      <p>No file open. Start vile with a .vil path, or press ⌘O.</p>
    </main>
  {/if}
  <StatusBar
    mode={ui.mode}
    layer={ui.layer}
    path={ui.view?.path ?? null}
    dirty={ui.view?.dirty ?? false}
    selectionSize={ui.selection.length}
    message={ui.message}
  />
</div>

<style>
  .app {
    display: grid;
    grid-template-rows: auto 1fr auto;
    height: 100vh;
  }
  .board {
    padding: 12px;
    overflow: auto;
  }
  .empty {
    display: grid;
    place-items: center;
    color: var(--muted);
  }
</style>
