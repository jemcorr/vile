<script lang="ts">
  import { onMount, tick } from "svelte";
  import { api, errorText } from "./lib/api";
  import { setView, ui } from "./lib/state.svelte";
  import { neighbor } from "./lib/grid";
  import { actionFor, type Action } from "./lib/keymap";
  import type { KeyRefDto, MacroActionDto, OpDto, ViewModel } from "./lib/types";
  import Keyboard from "./components/Keyboard.svelte";
  import LayerBar from "./components/LayerBar.svelte";
  import StatusBar from "./components/StatusBar.svelte";
  import KeycodeInput from "./components/KeycodeInput.svelte";
  import DiffPanel from "./components/DiffPanel.svelte";
  import Palette from "./components/Palette.svelte";
  import ComboEditor from "./components/ComboEditor.svelte";
  import MacroEditor from "./components/MacroEditor.svelte";
  import { comboOverlays } from "./lib/combos";
  import { PANELS } from "./lib/modes";

  function onkeyclick(id: string, shift: boolean): void {
    if (shift) {
      ui.selection = ui.selection.includes(id) ? ui.selection.filter((s) => s !== id) : [...ui.selection, id];
    } else {
      ui.cursor = id;
    }
  }

  /** Key ids sorted in board order. */
  function boardOrder(ids: string[]): string[] {
    const order = ui.view?.board?.keys.map((k) => k.id) ?? [];
    return [...ids].sort((a, b) => order.indexOf(a) - order.indexOf(b));
  }

  /** The keys an action applies to: the selection, or the cursor key. */
  function targets(): string[] {
    return ui.selection.length > 0 ? boardOrder(ui.selection) : [ui.cursor];
  }

  function keyRefs(ids: string[]): KeyRefDto[] {
    return ids.map((key) => ({ layer: ui.layer, key }));
  }

  async function apply(op: OpDto, done: string): Promise<void> {
    try {
      setView(await api.applyOp(op));
      const warnings = ui.view?.warnings ?? [];
      ui.message = warnings.length > 0 ? `${done} (warning: ${warnings.join("; ")})` : done;
    } catch (e) {
      ui.message = "error: " + errorText(e);
    }
  }

  async function runView(call: () => Promise<ViewModel | null>, done: (view: ViewModel) => string): Promise<void> {
    try {
      const view = await call();
      if (view) {
        setView(view);
        ui.message = done(view);
      }
    } catch (e) {
      ui.message = "error: " + errorText(e);
    }
  }

  async function run(action: Action): Promise<void> {
    if (action.kind === "palette") {
      ui.mode = "palette";
      return;
    }
    if (ui.view === null && !["open", "save", "escape"].includes(action.kind)) {
      ui.message = "open a file first (⌘O)";
      return;
    }
    const layer = ui.layer;
    switch (action.kind) {
      case "move": {
        ui.cursor = neighbor(ui.view?.board?.keys ?? [], ui.cursor, action.dir);
        if (ui.mode === "visual" && !ui.selection.includes(ui.cursor)) {
          ui.selection = [...ui.selection, ui.cursor];
        }
        return;
      }
      case "layer":
        if (action.layer < (ui.view?.layerCount ?? 0)) {
          ui.layer = action.layer;
          ui.message = `layer ${action.layer}`;
        }
        return;
      case "toggleSelect":
        ui.selection = ui.selection.includes(ui.cursor)
          ? ui.selection.filter((s) => s !== ui.cursor)
          : [...ui.selection, ui.cursor];
        ui.message = `${ui.selection.length} selected`;
        return;
      case "visual":
        if (ui.mode === "visual") {
          ui.mode = "normal";
          ui.message = "normal";
        } else {
          ui.mode = "visual";
          if (!ui.selection.includes(ui.cursor)) ui.selection = [...ui.selection, ui.cursor];
          ui.message = "visual";
        }
        return;
      case "escape":
        ui.selection = [];
        ui.swapMark = null;
        ui.mode = "normal";
        ui.message = "";
        return;
      case "insert":
        ui.mode = "insert";
        return;
      case "clear": {
        const keys = targets();
        await apply({ type: "clearKeys", layer, keys }, `cleared ${keys.length} keys`);
        return;
      }
      case "transparent": {
        const keys = targets();
        await apply({ type: "transparentKeys", layer, keys }, `made ${keys.length} keys transparent`);
        return;
      }
      case "copy": {
        const keys = targets();
        await apply({ type: "copyKeys", layer, keys }, `copied ${keys.length} keys`);
        return;
      }
      case "paste":
        await apply({ type: "pasteKeys", layer, keys: boardOrder(ui.selection) }, "pasted");
        return;
      case "swap": {
        if (ui.swapMark === null) {
          ui.swapMark = keyRefs(targets());
          ui.message = "swap: select the other keys and press s";
        } else {
          const a = ui.swapMark;
          const b = keyRefs(targets());
          await apply({ type: "swapKeys", a, b }, `swapped ${b.length} keys`);
          ui.swapMark = null;
        }
        return;
      }
      case "undo":
        await runView(() => api.undo(), () => "undo");
        return;
      case "redo":
        await runView(() => api.redo(), () => "redo");
        return;
      case "copyLayer":
        await apply({ type: "copyLayer", layer }, `copied layer ${layer}`);
        return;
      case "pasteLayer":
        await apply({ type: "pasteLayer", layer }, `pasted onto layer ${layer}`);
        return;
      case "clearLayer":
        await apply({ type: "clearLayer", layer, transparent: false }, `cleared layer ${layer}`);
        return;
      case "transparentLayer":
        await apply({ type: "clearLayer", layer, transparent: true }, `made layer ${layer} transparent`);
        return;
      case "save":
        await runView(
          () => (ui.view?.path ? api.save() : api.saveAsDialog()),
          (view) => `saved ${view.path ?? ""}`,
        );
        return;
      case "open":
        try {
          const opened = await api.openDialog();
          if (opened) {
            setView(opened);
            ui.layer = 0;
            ui.message = `opened ${opened.path ?? ""}`;
          }
        } catch (e) {
          ui.message = "error: " + errorText(e);
        }
        return;
      case "toggleDiffBase": {
        const next = ui.view?.diffBase === "disk" ? "git" : "disk";
        await runView(
          () => api.setDiffBase(next),
          () => (next === "git" ? "diff against git HEAD" : "diff against the file on disk"),
        );
        return;
      }
      case "nextPanel": {
        const index = PANELS.indexOf(ui.panel);
        ui.panel = PANELS[(index + 1) % PANELS.length] ?? "diff";
        ui.message = `panel: ${ui.panel}`;
        return;
      }
      case "toggleCombos":
        ui.showCombos = !ui.showCombos;
        ui.message = ui.showCombos ? "combo overlay on" : "combo overlay off";
        return;
    }
  }

  function onkeydown(e: KeyboardEvent): void {
    if (ui.mode !== "normal" && ui.mode !== "visual") return;
    const target = e.target;
    if (
      target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement ||
      target instanceof HTMLSelectElement
    ) {
      return;
    }
    const action = actionFor(ui.mode, e);
    if (action === null) return;
    e.preventDefault();
    void run(action);
  }

  /** The wire name of the cursor key when exactly one key is targeted, else "". */
  function insertInitial(): string {
    const keys = targets();
    if (keys.length !== 1) return "";
    const index = ui.view?.board?.keys.findIndex((k) => k.id === keys[0]) ?? -1;
    const cells = ui.view?.layers[ui.layer] ?? [];
    return cells[index]?.wire ?? "";
  }

  /** Runs a command chosen in the palette. */
  async function onPaletteRun(id: string, arg: number | null): Promise<void> {
    ui.mode = "normal";
    const pureUi = [
      "panelDiff",
      "panelCombos",
      "panelMacros",
      "panelTapDances",
      "panelKeyOverrides",
      "panelAltRepeat",
      "toggleComboOverlay",
    ];
    if (ui.view === null && !["open", "saveAs", ...pureUi].includes(id)) {
      ui.message = "open a file first (⌘O)";
      return;
    }
    const layer = ui.layer;
    switch (id) {
      case "open":
        await run({ kind: "open" });
        return;
      case "save":
        await run({ kind: "save" });
        return;
      case "undo":
        await run({ kind: "undo" });
        return;
      case "redo":
        await run({ kind: "redo" });
        return;
      case "clearLayer":
        await run({ kind: "clearLayer" });
        return;
      case "transparentLayer":
        await run({ kind: "transparentLayer" });
        return;
      case "copyLayer":
        await run({ kind: "copyLayer" });
        return;
      case "pasteLayer":
        await run({ kind: "pasteLayer" });
        return;
      case "saveAs":
        await runView(() => api.saveAsDialog(), (view) => `saved ${view.path ?? ""}`);
        return;
      case "swapLayer":
        if (arg !== null) {
          await apply({ type: "swapLayers", a: layer, b: arg }, `swapped layers ${layer} and ${arg}`);
          ui.layer = arg;
        }
        return;
      case "moveLayer":
        if (arg !== null) {
          await apply({ type: "moveLayer", from: layer, to: arg }, `moved layer ${layer} to ${arg}`);
          ui.layer = arg;
        }
        return;
      case "mirrorLeftToRight":
        await apply({ type: "mirrorLayer", layer, mode: "leftToRight" }, `mirrored layer ${layer}`);
        return;
      case "mirrorRightToLeft":
        await apply({ type: "mirrorLayer", layer, mode: "rightToLeft" }, `mirrored layer ${layer}`);
        return;
      case "mirrorSwap":
        await apply({ type: "mirrorLayer", layer, mode: "swap" }, `mirrored layer ${layer}`);
        return;
      case "diffDisk":
        await runView(() => api.setDiffBase("disk"), () => "diff against the file on disk");
        return;
      case "diffGit":
        await runView(() => api.setDiffBase("git"), () => "diff against git HEAD");
        return;
      case "panelDiff":
        ui.panel = "diff";
        return;
      case "panelCombos":
        ui.panel = "combos";
        return;
      case "panelMacros":
        ui.panel = "macros";
        return;
      case "panelTapDances":
        ui.panel = "tapDances";
        return;
      case "panelKeyOverrides":
        ui.panel = "keyOverrides";
        return;
      case "panelAltRepeat":
        ui.panel = "altRepeat";
        return;
      case "toggleComboOverlay":
        ui.showCombos = !ui.showCombos;
        return;
    }
  }

  function onMacroSave(index: number, actions: MacroActionDto[]): void {
    void apply({ type: "setMacro", index, actions }, `macro M${index} saved`);
  }

  function onComboSave(index: number, keys: string[], output: string): void {
    void apply({ type: "setCombo", index, keys, output }, `combo ${index} saved`);
  }

  function onjump(layer: number, keyId: string): void {
    ui.layer = layer;
    ui.cursor = keyId;
  }

  function onInsertSubmit(wire: string): void {
    const keys = targets();
    ui.mode = "normal";
    void apply({ type: "setKeys", layer: ui.layer, keys, keycode: wire }, `set ${keys.length} keys to ${wire}`);
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

<svelte:window {onkeydown} />

<div class="app">
  {#if ui.view !== null && ui.view.board !== null}
    <LayerBar
      layerCount={ui.view.layerCount}
      current={ui.layer}
      diff={ui.view.diff}
      onselect={(layer: number) => (ui.layer = layer)}
    />
    <div class="workspace">
      <main class="board">
        <Keyboard
          keys={ui.view.board.keys}
          cells={ui.view.layers[ui.layer] ?? []}
          cursor={ui.cursor}
          selection={ui.selection}
          {onkeyclick}
          overlays={ui.showCombos ? comboOverlays(ui.view.combos, ui.view.layers[ui.layer] ?? [], ui.view.board.keys) : []}
        />
      </main>
      <aside class="side">
        {#if ui.panel === "combos"}
          <ComboEditor combos={ui.view.combos} onsave={onComboSave} />
        {:else if ui.panel === "macros"}
          <MacroEditor macros={ui.view.macros} onsave={onMacroSave} />
        {:else}
          <DiffPanel diff={ui.view.diff} base={ui.view.diffBase} {onjump} />
        {/if}
      </aside>
    </div>
  {:else}
    <main class="empty">
      <p>No file open. Start vile with a .vil path, or press ⌘O.</p>
    </main>
  {/if}
  {#if ui.mode === "insert"}
    <footer class="insert">
      <KeycodeInput initial={insertInitial()} onsubmit={onInsertSubmit} oncancel={() => (ui.mode = "normal")} />
    </footer>
  {/if}
  <StatusBar
    mode={ui.mode}
    layer={ui.layer}
    path={ui.view?.path ?? null}
    dirty={ui.view?.dirty ?? false}
    selectionSize={ui.selection.length}
    message={ui.message}
  />
  {#if ui.mode === "palette"}
    <Palette
      layerCount={ui.view?.layerCount ?? 0}
      onrun={(id, arg) => void onPaletteRun(id, arg)}
      onclose={() => (ui.mode = "normal")}
    />
  {/if}
</div>

<style>
  .app {
    display: grid;
    grid-template-rows: auto 1fr auto;
    height: 100vh;
  }
  .workspace {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 380px;
    min-height: 0;
  }
  .board {
    padding: 12px;
    overflow: auto;
  }
  .side {
    padding: 12px;
    overflow: auto;
    border-left: 1px solid var(--key-border);
  }
  .empty {
    display: grid;
    place-items: center;
    color: var(--muted);
  }
  .insert {
    padding: 8px 12px;
    border-top: 1px solid var(--key-border);
    background: var(--bg);
  }
</style>
