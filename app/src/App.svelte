<script lang="ts">
  import { onMount, tick } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { api, errorText } from "./lib/api";
  import { ui, setView } from "./lib/state.svelte";
  import { neighbor } from "./lib/grid";
  import { actionFor, type Action } from "./lib/keymap";
  import { COMMANDS, commandReason } from "./lib/palette";
  import { PANELS, type Panel } from "./lib/modes";
  import { comboOverlays } from "./lib/combos";
  import type {
    OpDto,
    KeycodeView,
    KeyPartsPatch,
    MacroActionDto,
    TapDanceDto,
    KeyOverrideDto,
    AltRepeatDto,
  } from "./lib/types";
  import Keyboard from "./components/Keyboard.svelte";
  import ResizeHandle from "./components/ResizeHandle.svelte";
  import LayerBar from "./components/LayerBar.svelte";
  import Picker from "./components/Picker.svelte";
  import Inspector from "./components/Inspector.svelte";
  import KeycodeInput from "./components/KeycodeInput.svelte";
  import DiffPanel from "./components/DiffPanel.svelte";
  import Palette from "./components/Palette.svelte";
  import ComboEditor from "./components/ComboEditor.svelte";
  import MacroEditor from "./components/MacroEditor.svelte";
  import TapDanceEditor from "./components/TapDanceEditor.svelte";
  import KeyOverrideEditor from "./components/KeyOverrideEditor.svelte";
  import AltRepeatEditor from "./components/AltRepeatEditor.svelte";
  import Field from "./components/Field.svelte";
  import { focusScope } from "./lib/focus";
  let viewportWidth = $state(1280);
  let viewportHeight = $state(820);
  function savedPanelSizes(): { side: number; picker: number } {
    try {
      const saved = JSON.parse(localStorage.getItem("vile.panelSizes") ?? "null");
      return {
        side:
          typeof saved?.side === "number" && Number.isFinite(saved.side)
            ? Math.max(240, saved.side)
            : 390,
        picker:
          typeof saved?.picker === "number" && Number.isFinite(saved.picker)
            ? Math.max(120, saved.picker)
            : 240,
      };
    } catch {
      return { side: 390, picker: 240 };
    }
  }
  let panelSizes = $state(savedPanelSizes());
  const sideMax = $derived(Math.max(240, viewportWidth - 300));
  const pickerMax = $derived(Math.max(120, viewportHeight - 240));
  const sideWidth = $derived(Math.min(panelSizes.side, sideMax));
  const pickerHeight = $derived(Math.min(panelSizes.picker, pickerMax));
  $effect(() => {
    const saved = JSON.stringify(panelSizes);
    try {
      localStorage.setItem("vile.panelSizes", saved);
    } catch {
      /* Editing still works without preference storage. */
    }
  });
  let busy = $state(false);
  let definitionStatus = $state("");
  async function refreshGeometry(): Promise<void> {
    try {
      setView(await api.refreshDefinition());
      definitionStatus = "Definition refreshed";
    } catch (e) {
      definitionStatus = errorText(e);
    }
  }
  let menu: { kind: "keys" | "layer"; x: number; y: number } | null = $state(null);
  let layerAction: string | null = $state(null);
  let review: { revision: number; saveAs: boolean; error: string } | null = $state(null);
  let unsaved: "open" | "close" | null = $state(null);
  let afterSave: "open" | "close" | null = $state(null);
  let rename: { kind: "layer" | "macro" | "combo"; index: number; name: string } | null =
    $state(null);
  let remapsText = $state("");
  let settings = $state(false);
  let boardFit = $state(false);
  let blankPress: { pointer: number; x: number; y: number; moved: boolean } | null = null;
  function blankTarget(e: PointerEvent): boolean {
    if (
      review ||
      unsaved ||
      rename ||
      settings ||
      layerAction ||
      ui.mode === "insert" ||
      ui.mode === "palette" ||
      !(e.target instanceof Element) ||
      !e.target.matches(
        ".app,.toolbar,.spacer,.layers,.workspace,.board,.board-heading,aside,.panels,.picker-dock,.picker,.picker-head,.tiles,footer",
      )
    )
      return false;
    const target = e.target;
    if (target instanceof HTMLElement) {
      const rect = target.getBoundingClientRect();
      if (
        (target.offsetWidth > target.clientWidth && e.clientX >= rect.left + target.clientWidth) ||
        (target.offsetHeight > target.clientHeight && e.clientY >= rect.top + target.clientHeight)
      )
        return false;
    }
    return true;
  }
  const compact = $derived(viewportWidth < 900);
  $effect(() => {
    if (compact) {
      ui.sideVisible = false;
      ui.inspectorPaths = [];
      ui.baseTarget = false;
      ui.comboEditing = null;
    }
  });
  const drafts = $derived(Object.values(ui.drafts).some(Boolean));
  const cells = $derived(ui.view?.layers[ui.layer] ?? []);
  const activeFeaturePath = $derived(
    ui.sideVisible &&
      ui.inspectorPaths.length === 1 &&
      (ui.inspectorPaths[0]?.startsWith(
        `${ui.panel === "combos" ? "combo" : ui.panel === "macros" ? "macro" : ui.panel === "tapDances" ? "tap_dance" : ui.panel === "keyOverrides" ? "key_override" : ui.panel === "altRepeat" ? "alt_repeat" : "__none__"}[`,
      ) ||
        ui.inspectorPaths[0]?.startsWith(`encoder_layout[${ui.layer}]`))
      ? ui.inspectorPaths
      : [],
  );
  const selectedPaths = $derived(
    activeFeaturePath.length
      ? activeFeaturePath
      : cells.filter((cell) => ui.selection.includes(cell.id)).map((cell) => cell.path),
  );
  const targetLabel = $derived.by(() => {
    if (ui.comboEditing !== null && activeFeaturePath[0] !== `combo[${ui.comboEditing}][4]`)
      return `Select combo ${ui.comboEditing} output field for picker assignment`;
    if (activeFeaturePath.length) {
      const path = activeFeaturePath[0]!;
      if (path.startsWith("encoder_layout")) return `Encoder on layer ${ui.layer}: ${path}`;
      const combo = /^combo\[(\d+)\]\[(\d+)\]$/.exec(path);
      if (combo)
        return `Combo ${combo[1]} ${combo[2] === "4" ? "output" : `trigger ${Number(combo[2]) + 1}`}`;
      const macro = /^macro\[(\d+)\]\[(\d+)\]\[(\d+)\]$/.exec(path);
      if (macro) return `Macro ${macro[1]}, action ${Number(macro[2]) + 1}, key ${macro[3]}`;
      const dance = /^tap_dance\[(\d+)\]\[(\d+)\]$/.exec(path);
      if (dance)
        return `Tap dance ${dance[1]} ${["tap", "hold", "double tap", "tap and hold"][Number(dance[2])] ?? path}`;
      const override = /^key_override\[(\d+)\]\.(trigger|replacement)$/.exec(path);
      if (override) return `Override ${override[1]} ${override[2]}`;
      const repeat = /^alt_repeat_key\[(\d+)\]\.(keycode|alt_keycode)$/.exec(path);
      if (repeat)
        return `Alt repeat ${repeat[1]} ${repeat[2] === "keycode" ? "key" : "alternate key"}`;
      return `Feature field: ${path}`;
    }
    const selected = cells.filter((cell) => ui.selection.includes(cell.id));
    if (!selected.length) return "No picker target selected";
    return selected.length === 1
      ? `Board key on layer ${ui.layer}: ${selected[0]!.id}`
      : `${selected.length} board keys on layer ${ui.layer}`;
  });
  function switchPanel(panel: Panel): void {
    ui.panel = panel;
    ui.inspectorPaths = [];
    ui.baseTarget = false;
    ui.comboEditing = null;
  }
  const inspected = $derived(
    selectedPaths.map((path) => ui.view?.keycodes[path]).filter((key): key is KeycodeView => !!key),
  );
  const activeCombo = $derived(ui.view?.combos.find((combo) => combo.index === ui.comboEditing));
  const comboSelection = $derived(
    activeCombo
      ? cells
          .filter(
            (cell) =>
              activeCombo.keys.some(
                (wire) =>
                  ui.view?.keycodes[
                    `combo[${activeCombo.index}][${activeCombo.keys.indexOf(wire)}]`
                  ]?.code === cell.code,
              ) && cell.code !== 0,
          )
          .map((cell) => cell.id)
      : ui.selection,
  );
  const menuCommands = $derived(
    COMMANDS.filter((command) =>
      menu?.kind === "keys"
        ? [
            "clearKeys",
            "transparentKeys",
            "copyKeys",
            "cutKeys",
            "pasteKeys",
            "swapKeys",
            "editKey",
          ].includes(command.id)
        : [
            "copyLayer",
            "pasteLayer",
            "clearLayer",
            "transparentLayer",
            "swapLayer",
            "moveLayer",
            "mirrorLeftToRight",
            "mirrorRightToLeft",
            "mirrorSwap",
          ].includes(command.id),
    ),
  );
  function resetDocument(): void {
    ui.generation += 1;
    ui.layer = 0;
    ui.selection = [];
    ui.inspectorPaths = [];
    ui.baseTarget = false;
    ui.comboEditing = null;
    ui.drafts = {};
    ui.draftMacro = null;
    ui.revealMacro = null;
    ui.swapMark = null;
  }
  async function refresh(): Promise<boolean> {
    try {
      setView(await api.getView());
      return true;
    } catch (e) {
      ui.message = errorText(e);
      return false;
    }
  }
  async function apply(op: OpDto, done = "Updated"): Promise<boolean> {
    if (busy) return false;
    busy = true;
    try {
      setView(await api.applyOp(op));
      ui.message = ui.view?.warnings.length ? ui.view.warnings.join("; ") : done;
      return true;
    } catch (e) {
      ui.message = errorText(e);
      return false;
    } finally {
      busy = false;
    }
  }
  async function change(
    call: () => Promise<import("./lib/types").ViewModel | null>,
  ): Promise<void> {
    if (busy) return;
    busy = true;
    try {
      const view = await call();
      if (view) setView(view);
    } catch (e) {
      ui.message = errorText(e);
    } finally {
      busy = false;
    }
  }
  function reason(id: string): string {
    return busy
      ? "An operation is in progress"
      : commandReason(id, ui.view, ui.selection.length > 0);
  }
  function selectLayer(layer: number): void {
    ui.layer = layer;
    ui.selection = [];
    ui.inspectorPaths = [];
    ui.baseTarget = false;
    ui.comboEditing = null;
  }
  function onkeyclick(id: string, shift: boolean): void {
    const cell = cells.find((cell) => cell.id === id);
    if (!cell) return;
    if (ui.comboEditing !== null && activeCombo) {
      const codes = activeCombo.keys.map(
        (_, i) => ui.view?.keycodes[`combo[${activeCombo.index}][${i}]`]?.code ?? 0,
      );
      const existing = codes.indexOf(cell.code);
      let keys = activeCombo.keys.filter(
        (_, i) => codes[i] !== 0 && codes[i] !== cell.code && codes.indexOf(codes[i]!) === i,
      );
      if (existing < 0) {
        if (keys.length >= 4) {
          ui.message = "A combo supports at most four distinct trigger keycodes";
          return;
        }
        if (cell.code === 0) {
          ui.message = "No key cannot be a combo trigger";
          return;
        }
        keys.push(cell.wire);
      }
      while (keys.length < 4) keys.push("KC_NO");
      void apply({ type: "setCombo", index: activeCombo.index, keys, output: activeCombo.output });
      return;
    }
    ui.cursor = id;
    ui.inspectorPaths = [];
    ui.baseTarget = false;
    if (ui.swapMark) {
      const a = ui.swapMark;
      ui.swapMark = null;
      void apply({ type: "swapKeys", a, b: [{ layer: ui.layer, key: id }] });
      ui.selection = [id];
      return;
    }
    ui.selection = shift ? Array.from(new Set([...ui.selection, id])) : [id];
  }
  function editKey(id: string): void {
    onkeyclick(id, false);
    ui.mode = "insert";
  }
  function menuKey(id: string, x: number, y: number): void {
    if (!ui.selection.includes(id)) onkeyclick(id, false);
    menu = { kind: "keys", x, y };
  }
  function assign(key: KeycodeView): void {
    if (drafts) {
      ui.message = "Apply or revert the macro draft before assigning keys";
      return;
    }
    if (ui.comboEditing !== null && activeFeaturePath[0] !== `combo[${ui.comboEditing}][4]`) {
      ui.message = "Select this combo's output field before assigning from the picker";
      return;
    }
    if (!selectedPaths.length) {
      ui.message = "Select a key or keycode field first";
      return;
    }
    if (ui.baseTarget) {
      void apply(
        { type: "patchFields", paths: selectedPaths, patch: { base: key.code } },
        "Base changed",
      );
      ui.baseTarget = false;
    } else void apply({ type: "replaceFields", paths: selectedPaths, keycode: key.wire });
  }
  function patch(patch: KeyPartsPatch): void {
    if (drafts) {
      ui.message = "Apply or revert the macro draft first";
      return;
    }
    void apply({ type: "patchFields", paths: selectedPaths, patch });
  }
  async function open(discard = false): Promise<void> {
    busy = true;
    try {
      const view = await api.openDialog(discard);
      if (view) {
        resetDocument();
        setView(view);
        ui.message = `Opened ${view.path}`;
        void refreshGeometry();
      }
    } catch (e) {
      ui.message = errorText(e);
    } finally {
      busy = false;
    }
  }
  function beginSave(saveAs = false): void {
    if (drafts) {
      ui.message = "Apply or revert the macro draft before saving";
      return;
    }
    if (!ui.view) return;
    review = { revision: ui.view.revision, saveAs, error: "" };
  }
  async function confirmSave(): Promise<void> {
    if (busy || !review || !ui.view) return;
    if (review.revision < 0) {
      if ((await refresh()) && review && ui.view)
        review = {
          ...review,
          revision: ui.view.revision,
          error: "Review the current diff, then save again.",
        };
      return;
    }
    if (review.revision !== ui.view.revision) {
      ui.message = "The document changed. Review its current diff before saving";
      review = {
        ...review,
        revision: ui.view.revision,
        error: "Review the current diff, then save again.",
      };
      return;
    }
    busy = true;
    let completed = false;
    try {
      const view = await (review.saveAs || !ui.view.path ? api.saveAsDialog() : api.save());
      if (view) {
        setView(view);
        review = null;
        completed = true;
        ui.message = `Saved ${view.path}`;
      }
    } catch (e) {
      const saveError = errorText(e);
      await refresh();
      ui.message = saveError;
      if (review) review = { ...review, revision: -1, error: saveError };
    } finally {
      busy = false;
    }
    if (completed && afterSave) {
      const action = afterSave;
      afterSave = null;
      if (action === "open") await open();
      else await api.closeWindow();
    }
  }
  async function requestClose(): Promise<void> {
    if (busy) return;
    if (ui.view?.dirty || drafts) unsaved = "close";
    else await api.closeWindow();
  }
  async function command(id: string, arg: number | null = null): Promise<void> {
    menu = null;
    if (reason(id)) return;
    if (id === "open") {
      if (drafts || ui.view?.dirty) unsaved = "open";
      else await open();
      return;
    }
    if (id === "save" || id === "saveAs") {
      beginSave(id === "saveAs");
      return;
    }
    if (id === "undo" || id === "redo") {
      if (drafts) {
        ui.message = "Apply or revert the macro draft first";
        return;
      }
      await change(() => (id === "undo" ? api.undo() : api.redo()));
      ui.inspectorPaths = [];
      return;
    }
    if (id === "editKey") {
      ui.mode = "insert";
      return;
    }
    if (id === "swapKeys") {
      if (ui.selection.length !== 1) {
        ui.message = "Select one key, then choose its swap target";
        return;
      }
      ui.swapMark = ui.selection.map((key) => ({ layer: ui.layer, key }));
      ui.message = "Click the key to swap with. Escape cancels.";
      return;
    }
    const keyOps = {
      clearKeys: "clearKeys",
      transparentKeys: "transparentKeys",
      copyKeys: "copyKeys",
      pasteKeys: "pasteKeys",
    } as const;
    if (id in keyOps) {
      await apply({ type: keyOps[id as keyof typeof keyOps], layer: ui.layer, keys: ui.selection });
      return;
    }
    if (id === "cutKeys") {
      if (await apply({ type: "copyKeys", layer: ui.layer, keys: ui.selection }))
        await apply({ type: "clearKeys", layer: ui.layer, keys: ui.selection });
      return;
    }
    if (["copyLayer", "pasteLayer"].includes(id)) {
      await apply({ type: id as "copyLayer" | "pasteLayer", layer: ui.layer });
      return;
    }
    if (["clearLayer", "transparentLayer"].includes(id)) {
      await apply({ type: "clearLayer", layer: ui.layer, transparent: id === "transparentLayer" });
      return;
    }
    if (["swapLayer", "moveLayer"].includes(id)) {
      if (arg === null) {
        layerAction = id;
        return;
      }
      if (
        await apply(
          id === "swapLayer"
            ? { type: "swapLayers", a: ui.layer, b: arg }
            : { type: "moveLayer", from: ui.layer, to: arg },
        )
      )
        selectLayer(arg);
      layerAction = null;
      return;
    }
    if (id.startsWith("mirror")) {
      await apply({
        type: "mirrorLayer",
        layer: ui.layer,
        mode:
          id === "mirrorLeftToRight"
            ? "leftToRight"
            : id === "mirrorRightToLeft"
              ? "rightToLeft"
              : "swap",
      });
      return;
    }
    if (id === "diffDisk" || id === "diffGit") {
      await change(() => api.setDiffBase(id === "diffGit" ? "git" : "disk"));
      return;
    }
    const panels: Record<string, Panel> = {
      panelDiff: "diff",
      panelCombos: "combos",
      panelMacros: "macros",
      panelTapDances: "tapDances",
      panelKeyOverrides: "keyOverrides",
      panelAltRepeat: "altRepeat",
    };
    if (panels[id]) {
      switchPanel(panels[id]);
      ui.sideVisible = true;
      return;
    }
    if (id === "toggleComboOverlay") ui.showCombos = !ui.showCombos;
  }
  function onkeydown(e: KeyboardEvent): void {
    if (e.key === "Escape" && dismissInteraction(false)) {
      e.preventDefault();
      return;
    }
    if (review && e.metaKey && e.key.toLowerCase() === "s") {
      e.preventDefault();
      void confirmSave();
      return;
    }
    if (menu || review || unsaved || rename || settings) return;
    const target = e.target;
    if (target instanceof Element) {
      if (target.closest("[role=dialog],input,textarea,select,[contenteditable]")) return;
      const control = target.closest("button,summary,a[href],[role=button],[role=menuitem]");
      if (
        control &&
        !target.closest("[data-key-id]") &&
        (!e.metaKey || !["s", "o", "k", "z"].includes(e.key.toLowerCase()))
      )
        return;
    }
    if (ui.mode !== "normal" && ui.mode !== "visual") return;
    const action = actionFor(ui.mode, e, ui.vimKeys, ui.vimRemaps);
    if (!action) return;
    e.preventDefault();
    if (action.kind === "move") {
      const previous = ui.cursor;
      ui.cursor = neighbor(ui.view?.board?.keys ?? [], previous, action.dir);
      ui.selection = action.extend
        ? Array.from(new Set([...ui.selection, previous, ui.cursor])).filter(Boolean)
        : [ui.cursor].filter(Boolean);
      ui.inspectorPaths = [];
      return;
    }
    if (action.kind === "layer") {
      if (action.layer < (ui.view?.layerCount ?? 0)) selectLayer(action.layer);
      return;
    }
    if (action.kind === "palette") {
      ui.mode = "palette";
      return;
    }
    if (action.kind === "escape") {
      dismissInteraction(true);
      return;
    }
    if (action.kind === "toggleSelect") {
      onkeyclick(ui.cursor, true);
      return;
    }
    if (action.kind === "visual") {
      ui.mode = ui.mode === "visual" ? "normal" : "visual";
      return;
    }
    const map: Partial<Record<Action["kind"], string>> = {
      open: "open",
      save: "save",
      undo: "undo",
      redo: "redo",
      insert: "editKey",
      copy: "copyKeys",
      paste: "pasteKeys",
      cut: "cutKeys",
      clear: "clearKeys",
      transparent: "transparentKeys",
      swap: "swapKeys",
      copyLayer: "copyLayer",
      pasteLayer: "pasteLayer",
      clearLayer: "clearLayer",
      transparentLayer: "transparentLayer",
      toggleCombos: "toggleComboOverlay",
    };
    if (action.kind === "nextPanel") {
      switchPanel(PANELS[(PANELS.indexOf(ui.panel) + 1) % PANELS.length] ?? "diff");
      return;
    }
    if (action.kind === "toggleDiffBase") {
      void command(ui.view?.diffBase === "git" ? "diffDisk" : "diffGit");
      return;
    }
    const id = map[action.kind];
    if (id) void command(id);
  }
  function dismissInteraction(clearSelection: boolean): boolean {
    const hadTransient = !!(
      menu ||
      layerAction ||
      review ||
      unsaved ||
      rename ||
      settings ||
      ui.mode === "insert" ||
      ui.mode === "palette"
    );
    menu = null;
    layerAction = null;
    if (review) {
      review = null;
      afterSave = null;
    }
    unsaved = null;
    rename = null;
    settings = false;
    if (ui.mode === "insert" || ui.mode === "palette") ui.mode = "normal";
    if (!hadTransient && clearSelection) {
      ui.selection = [];
      ui.inspectorPaths = [];
      ui.swapMark = null;
      ui.baseTarget = false;
      ui.comboEditing = null;
    }
    return hadTransient;
  }
  function saveTapDance(td: TapDanceDto): void {
    void apply({ type: "setTapDance", ...td });
  }
  function saveOverride(ko: KeyOverrideDto): void {
    void apply({ type: "setKeyOverride", ...ko });
  }
  function saveAltRepeat(ar: AltRepeatDto): void {
    void apply({ type: "setAltRepeat", ...ar });
  }
  function saveMacro(index: number, actions: MacroActionDto[]): Promise<boolean> {
    return apply({ type: "setMacro", index, actions });
  }
  onMount(() => {
    let unlisten: (() => void) | undefined;
    try {
      ui.vimKeys = localStorage.getItem("vile.vimKeys") === "true";
      ui.vimRemaps = JSON.parse(localStorage.getItem("vile.vimRemaps") ?? "{}");
    } catch {
      ui.vimRemaps = {};
    }
    void (async () => {
      await refresh();
      await tick();
      await api.smokeReport(
        document.querySelectorAll("[data-key-id]").length,
        ui.view?.layerCount ?? 0,
        ui.view?.board?.name ?? "none",
      );
      unlisten = await listen("request-close", () => void requestClose());
      if (ui.view) void refreshGeometry();
    })();
    return () => unlisten?.();
  });
</script>

<svelte:window
  {onkeydown}
  onpointermove={(e) => {
    if (blankPress?.pointer === e.pointerId)
      blankPress.moved ||= Math.hypot(e.clientX - blankPress.x, e.clientY - blankPress.y) > 6;
  }}
  onpointerup={(e) => {
    if (blankPress?.pointer === e.pointerId) blankPress = null;
  }}
  onpointercancel={(e) => {
    if (blankPress?.pointer === e.pointerId) blankPress = null;
  }}
  onclick={(e) => {
    if (menu && e.target instanceof Element && !e.target.closest(".menu")) menu = null;
  }}
  bind:innerWidth={viewportWidth}
  bind:innerHeight={viewportHeight}
/>
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="app"
  aria-busy={busy}
  inert={busy}
  onpointerdown={(e) => {
    blankPress =
      e.button === 0 && blankTarget(e)
        ? { pointer: e.pointerId, x: e.clientX, y: e.clientY, moved: false }
        : null;
  }}
  onpointerup={(e) => {
    const press = blankPress;
    if (press?.pointer !== e.pointerId) return;
    blankPress = null;
    if (!press.moved && Math.hypot(e.clientX - press.x, e.clientY - press.y) <= 6 && blankTarget(e))
      dismissInteraction(true);
  }}
>
  <header class="toolbar">
    <strong class="brand">vile</strong
    >{#each COMMANDS.filter((c) => ["open", "save", "undo", "redo"].includes(c.id)) as c}<button
        disabled={!!reason(c.id)}
        title={reason(c.id) || c.title}
        onclick={() => void command(c.id)}>{c.title} <kbd>{c.shortcut}</kbd></button
      >{#if ui.view && reason(c.id)}<small class="disabled-reason">{reason(c.id)}</small>{/if}
    {/each}
    <button
      disabled={!ui.view}
      onclick={() => {
        switchPanel("diff");
        ui.sideVisible = true;
      }}>{ui.view?.pendingDiff.length ?? 0} pending changes</button
    >{#if drafts}<button
        class="draft-indicator"
        onclick={() => {
          switchPanel("macros");
          ui.sideVisible = true;
          ui.revealMacro = ui.draftMacro;
        }}>Draft in macro M{ui.draftMacro} · open</button
      >{/if}
    <span class="spacer"></span><button onclick={() => (ui.mode = "palette")}
      >Commands <kbd>⌘K</kbd></button
    ><button
      onclick={() => {
        settings = true;
        remapsText = JSON.stringify(ui.vimRemaps, null, 2);
      }}>Settings</button
    >
  </header>
  {#if ui.view && ui.view.board}
    <LayerBar
      layerCount={ui.view.layerCount}
      current={ui.layer}
      diff={ui.view.pendingDiff}
      names={ui.view.names}
      onselect={selectLayer}
      onmenu={(layer, x, y) => {
        selectLayer(layer);
        menu = { kind: "layer", x, y };
      }}
    />
    <div
      class="workspace"
      style:grid-template-columns={compact
        ? "minmax(0, 1fr)"
        : ui.sideVisible
          ? `minmax(0, 1fr) 8px ${sideWidth}px`
          : "minmax(0, 1fr)"}
    >
      <main class="board">
        <div class="board-heading">
          <span>{ui.view.board.name}</span><span role="status" class="geometry-status"
            >{ui.view.geometrySource === "device"
              ? "Shape: connected keyboard"
              : ui.view.geometrySource === "cache"
                ? "Shape: saved definition"
                : "Shape: matrix fallback. No matching keyboard definition is available."}</span
          ><button
            title={definitionStatus || "Read the connected keyboard definition"}
            onclick={() => void refreshGeometry()}>Refresh shape</button
          >{#if definitionStatus}<span role="status">{definitionStatus}</span>{/if}<button
            onclick={() => {
              ui.sideVisible = !ui.sideVisible;
              if (!ui.sideVisible) {
                ui.inspectorPaths = [];
                ui.baseTarget = false;
                ui.comboEditing = null;
              }
            }}>{ui.sideVisible ? "Hide panel" : "Show panel"}</button
          ><button
            onclick={() =>
              (rename = {
                kind: "layer",
                index: ui.layer,
                name: ui.view?.names.layers[ui.layer] ?? "",
              })}>Name layer</button
          ><button onclick={() => (boardFit = !boardFit)}
            >{boardFit ? "Full-size keys" : "Fit board"}</button
          >
        </div>
        {#if ui.comboEditing !== null}<p class="notice">
            Editing combo {ui.comboEditing} triggers. Click board keys to toggle their keycodes.
            <button onclick={() => (ui.comboEditing = null)}>Done</button>
          </p>{/if}
        <Keyboard
          keys={ui.view.board.keys}
          {cells}
          cursor={ui.cursor}
          selection={comboSelection}
          fit={boardFit}
          {onkeyclick}
          locked={ui.comboEditing !== null || busy}
          onmarquee={(ids, additive) => {
            ui.selection = additive ? Array.from(new Set([...ui.selection, ...ids])) : ids;
            ui.inspectorPaths = [];
          }}
          onblankclick={() => dismissInteraction(true)}
          ondragkey={(from, to, copy) => {
            if (copy) {
              const source = cells.find((cell) => cell.id === from);
              if (source)
                void apply({ type: "setKeys", layer: ui.layer, keys: [to], keycode: source.wire });
            } else
              void apply({
                type: "swapKeys",
                a: [{ layer: ui.layer, key: from }],
                b: [{ layer: ui.layer, key: to }],
              });
          }}
          onassign={(id, wire) =>
            void apply({ type: "setKeys", layer: ui.layer, keys: [id], keycode: wire })}
          onedit={editKey}
          onmenu={menuKey}
          overlays={ui.showCombos ? comboOverlays(ui.view.combos, cells, ui.view.board.keys) : []}
        />
        {#if Object.keys(ui.view.keycodes).some( (p) => p.startsWith(`encoder_layout[${ui.layer}]`) )}<details
            ontoggle={(e) => {
              if (
                !e.currentTarget.open &&
                ui.inspectorPaths[0]?.startsWith(`encoder_layout[${ui.layer}]`)
              )
                ui.inspectorPaths = [];
            }}
          >
            <summary>Encoders</summary
            >{#each Object.keys(ui.view.keycodes).filter( (p) => p.startsWith(`encoder_layout[${ui.layer}]`) ) as path}<Field
                {path}
                label={path}
              />{/each}
          </details>{/if}
      </main>
      {#if ui.sideVisible && !compact}<ResizeHandle
          orientation="vertical"
          label="Resize editor panel"
          value={sideWidth}
          min={240}
          max={sideMax}
          initial={390}
          onresize={(value) => (panelSizes.side = value)}
        />{/if}
      <aside class:hidden={!ui.sideVisible} class:drawer={compact}>
        <div class="target-status" role="status" aria-live="polite">
          Picker target: {targetLabel}
        </div>
        {#if inspected.length}<Inspector
            keys={inspected}
            layerCount={ui.view.layerCount}
            onpatch={patch}
            onbase={() => (ui.baseTarget = !ui.baseTarget)}
            choosing={ui.baseTarget}
          />{:else}<p class="hint">
            Select a board key or a keycode field to inspect its parts.
          </p>{/if}
        <nav class="panels" aria-label="Editor panels">
          {#each PANELS as panel}<button
              class:active={ui.panel === panel}
              aria-current={ui.panel === panel ? "true" : undefined}
              onclick={() => switchPanel(panel)}
              >{{
                diff: "Diff",
                combos: "Combos",
                macros: "Macros",
                tapDances: "Tap dances",
                keyOverrides: "Overrides",
                altRepeat: "Alt repeat",
              }[panel]}</button
            >{/each}
        </nav>
        <div class:hidden={ui.panel !== "diff"}>
          <div class="row">
            <button onclick={() => void command("diffDisk")}>Since save</button><button
              onclick={() => void command("diffGit")}>Git HEAD</button
            >
          </div>
          <DiffPanel
            diff={ui.view.diff}
            base={ui.view.diffBase}
            onjump={(layer, key) => {
              selectLayer(layer);
              onkeyclick(key, false);
            }}
          />
        </div>
        <div class:hidden={ui.panel !== "combos"}>
          <ComboEditor
            combos={ui.view.combos}
            onrename={(index) =>
              (rename = { kind: "combo", index, name: ui.view?.names.combos[index] ?? "" })}
          />
        </div>
        <div class:hidden={ui.panel !== "macros"}>
          {#key ui.generation}<MacroEditor macros={ui.view.macros} onsave={saveMacro} />{/key}
          <details>
            <summary>Name a macro</summary>{#each ui.view.macros as macro}<button
                onclick={() =>
                  (rename = {
                    kind: "macro",
                    index: macro.index,
                    name: ui.view?.names.macros[macro.index] ?? "",
                  })}>M{macro.index} {ui.view.names.macros[macro.index] ?? ""}</button
              >{/each}
          </details>
        </div>
        <div class:hidden={ui.panel !== "tapDances"}>
          <TapDanceEditor tapDances={ui.view.tapDances} onsave={saveTapDance} />
        </div>
        <div class:hidden={ui.panel !== "keyOverrides"}>
          <KeyOverrideEditor
            keyOverrides={ui.view.keyOverrides}
            layerCount={ui.view.layerCount}
            onsave={saveOverride}
          />
        </div>
        <div class:hidden={ui.panel !== "altRepeat"}>
          <AltRepeatEditor altRepeatKeys={ui.view.altRepeatKeys} onsave={saveAltRepeat} />
        </div>
      </aside>
    </div>
    <ResizeHandle
      orientation="horizontal"
      label="Resize keycode picker"
      value={pickerHeight}
      min={120}
      max={pickerMax}
      initial={240}
      onresize={(value) => (panelSizes.picker = value)}
    />
    <div class="picker-dock" style:height={`${pickerHeight}px`}>
      <Picker view={ui.view} onchoose={assign} choosingBase={ui.baseTarget} target={targetLabel} />
    </div>
  {:else}<main class="empty">
      <h1>Edit your keyboard visually.</h1>
      <button onclick={() => void command("open")}>Open a Vial layout <kbd>⌘O</kbd></button>
    </main>{/if}
  {#if ui.mode === "insert"}<div class="overlay">
      <div
        class="dialog"
        role="dialog"
        aria-modal="true"
        aria-label="Type keycode"
        tabindex="-1"
        use:focusScope={() => (ui.mode = "normal")}
      >
        <h2>Type a keycode</h2>
        <KeycodeInput
          initial={inspected.length === 1 ? inspected[0]!.wire : ""}
          onsubmit={(wire) => {
            ui.mode = "normal";
            if (selectedPaths.length)
              void apply({ type: "replaceFields", paths: selectedPaths, keycode: wire });
          }}
          oncancel={() => (ui.mode = "normal")}
        /><button onclick={() => (ui.mode = "normal")}>Cancel <kbd>Esc</kbd></button>
      </div>
    </div>{/if}
  <footer>
    <span>{ui.view?.path ?? "No layout open"}{ui.view?.dirty ? " • unsaved" : ""}</span><span
      role="status">{busy ? "Working…" : ui.message}</span
    >
  </footer>
  {#if ui.view?.warnings.length}<div class="warnings" role="status">
      {ui.view.warnings.join(" · ")}
    </div>{/if}
</div>
{#if menu}<div
    class="menu"
    use:focusScope={() => (menu = null)}
    tabindex="-1"
    style:left={`${Math.max(8, Math.min(menu.x, viewportWidth - 270))}px`}
    style:top={`${Math.max(8, Math.min(menu.y, viewportHeight - 350))}px`}
    role="menu"
  >
    {#each menuCommands as c}<button
        role="menuitem"
        disabled={!!reason(c.id)}
        title={reason(c.id)}
        onclick={() => void command(c.id)}
        >{c.title}<kbd>{c.shortcut}</kbd>{#if reason(c.id)}<small>{reason(c.id)}</small
          >{/if}</button
      >{/each}<button role="menuitem" onclick={() => (menu = null)}>Cancel <kbd>Esc</kbd></button>
  </div>{/if}
{#if layerAction}<div class="overlay">
    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      aria-label="Choose layer"
      tabindex="-1"
      use:focusScope={() => (layerAction = null)}
    >
      <h2>{layerAction === "swapLayer" ? "Swap with layer" : "Move to layer"}</h2>
      <div class="row">
        {#each Array.from({ length: ui.view?.layerCount ?? 0 }, (_, i) => i) as i}<button
            onclick={() => void command(layerAction!, i)}
            >{ui.view?.names.layers[i] || `L${i}`}</button
          >{/each}
      </div>
      <button onclick={() => (layerAction = null)}>Cancel</button>
    </div>
  </div>{/if}
{#if review && ui.view}<div class="overlay">
    <div
      class="dialog review"
      role="dialog"
      aria-modal="true"
      aria-label="Review changes before save"
      tabindex="-1"
      use:focusScope={() => {
        review = null;
        afterSave = null;
      }}
    >
      <h2>Review changes before saving</h2>
      {#if review.error}<p role="alert">{review.error}</p>{/if}
      <DiffPanel diff={ui.view.pendingDiff} base="disk" onjump={() => {}} />
      <div class="row">
        <button disabled={busy} onclick={() => void confirmSave()}
          >{review.revision < 0
            ? "Refresh review"
            : review.revision !== ui.view.revision
              ? "Review current changes"
              : review.saveAs
                ? "Save reviewed changes as…"
                : "Save reviewed changes"} <kbd>⌘S</kbd></button
        >{#if review.error && !review.saveAs}<button
            onclick={() => {
              if (review) review = { ...review, saveAs: true };
            }}>Save to another file…</button
          >{/if}<button
          onclick={() => {
            review = null;
            afterSave = null;
          }}>Cancel</button
        >
      </div>
    </div>
  </div>{/if}
{#if unsaved}<div class="overlay">
    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      aria-label="Unsaved changes"
      tabindex="-1"
      use:focusScope={() => (unsaved = null)}
    >
      <h2>Keep the pending changes?</h2>
      <p>
        {drafts
          ? "A macro draft also needs to be applied or reverted before saving."
          : "The current layout has unsaved changes."}
      </p>
      <div class="row">
        <button
          disabled={drafts}
          onclick={() => {
            afterSave = unsaved;
            unsaved = null;
            beginSave();
          }}>Review and save</button
        ><button
          onclick={() => {
            const action = unsaved;
            unsaved = null;
            if (action === "open") void open(true);
            else void api.closeWindow();
          }}>Discard and {unsaved === "open" ? "open" : "close"}</button
        ><button onclick={() => (unsaved = null)}>Cancel</button>
      </div>
    </div>
  </div>{/if}
{#if rename}<div class="overlay">
    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      use:focusScope={() => (rename = null)}
      aria-label="Name item"
    >
      <form
        class="rename-form"
        onsubmit={(e) => {
          e.preventDefault();
          if (rename)
            void apply({ type: "rename", ...rename }).then((ok) => {
              if (ok) rename = null;
            });
        }}
      >
        <h2>Name {rename.kind} {rename.index}</h2>
        <input aria-label="Name" maxlength="100" bind:value={rename.name} />
        <div class="row">
          <button type="submit">Apply</button><button type="button" onclick={() => (rename = null)}
            >Cancel</button
          >
        </div>
      </form>
    </div>
  </div>{/if}
{#if settings}<div class="overlay">
    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      aria-label="Settings"
      tabindex="-1"
      use:focusScope={() => (settings = false)}
    >
      <h2>Keyboard shortcuts</h2>
      <label
        ><input
          type="checkbox"
          bind:checked={ui.vimKeys}
          onchange={() => localStorage.setItem("vile.vimKeys", String(ui.vimKeys))}
        /> Enable vim-style letter bindings</label
      ><label
        >Remap letter bindings, JSON object <textarea
          bind:value={remapsText}
          rows="5"
          placeholder={JSON.stringify({ a: "h" })}></textarea></label
      >
      <p>Arrows, digits, Delete and conventional Cmd shortcuts work by default.</p>
      <div class="row">
        <button
          onclick={() => {
            try {
              const value = JSON.parse(remapsText);
              if (
                !value ||
                Array.isArray(value) ||
                typeof value !== "object" ||
                Object.values(value).some((v) => typeof v !== "string")
              )
                throw new Error("Use a JSON object with string values");
              ui.vimRemaps = value;
              localStorage.setItem("vile.vimRemaps", JSON.stringify(value));
              settings = false;
            } catch (e) {
              ui.message = errorText(e);
            }
          }}>Apply</button
        ><button onclick={() => (settings = false)}>Cancel</button>
      </div>
    </div>
  </div>{/if}
{#if ui.mode === "palette"}<Palette
    layerCount={ui.view?.layerCount ?? 0}
    onrun={(id, arg) => {
      ui.mode = "normal";
      void command(id, arg);
    }}
    onclose={() => (ui.mode = "normal")}
  />{/if}

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }
  .toolbar {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
    padding: 10px 16px;
    border-bottom: 1px solid var(--key-border);
  }
  .brand {
    font-size: 22px;
    margin-right: 16px;
    letter-spacing: -1px;
  }
  .spacer {
    flex: 1;
  }
  .workspace {
    display: grid;
    position: relative;
    flex: 1;
    min-height: 0;
  }
  .picker-dock {
    flex: 0 0 auto;
    min-height: 0;
    overflow: hidden;
  }
  .board {
    padding: 16px;
    overflow: auto;
  }
  .board-heading {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    margin-bottom: 16px;
    color: var(--muted);
    font-size: 12px;
  }
  aside {
    padding: 16px;
    overflow: auto;
  }
  aside.drawer {
    position: absolute;
    inset: 0 0 0 auto;
    z-index: 5;
    width: min(420px, 95%);
    background: var(--bg);
    border-left: 1px solid var(--key-border);
    box-shadow: -8px 0 24px #0003;
  }
  .target-status {
    padding: 8px;
    margin-bottom: 8px;
    border: 1px solid var(--key-border);
    border-radius: 6px;
    font-size: 12px;
  }
  .draft-indicator {
    background: var(--key-mod);
  }
  .geometry-status {
    color: var(--fg);
  }
  .disabled-reason {
    color: var(--muted);
    font-size: 11px;
  }
  .panels,
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
    margin-bottom: 8px;
  }
  .panels {
    padding-bottom: 10px;
    border-bottom: 1px solid var(--key-border);
  }
  .active {
    background: var(--selected);
  }
  .hint {
    color: var(--muted);
    font-size: 13px;
  }
  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
  }
  .empty h1 {
    font-size: 24px;
    font-weight: 500;
  }
  footer {
    display: flex;
    gap: 16px;
    justify-content: space-between;
    padding: 7px 16px;
    border-top: 1px solid var(--key-border);
    font-size: 11px;
    color: var(--muted);
  }
  footer > span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .warnings {
    padding: 6px 16px;
    color: var(--changed);
    font-size: 12px;
  }
  .overlay {
    position: fixed;
    inset: 0;
    background: #0005;
    display: grid;
    place-items: center;
    z-index: 20;
  }
  .dialog {
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-width: min(320px, calc(100vw - 24px));
    max-width: calc(100vw - 24px);
    max-height: 80vh;
    overflow: auto;
    background: var(--bg);
    padding: 24px;
    border: 1px solid var(--key-border);
    border-radius: 12px;
    box-shadow: 0 16px 60px #0004;
  }
  .dialog h2 {
    font-size: 18px;
    margin: 0;
  }
  .review {
    width: min(750px, calc(100vw - 24px));
  }
  .menu {
    position: fixed;
    z-index: 10;
    display: flex;
    flex-direction: column;
    width: min(260px, calc(100vw - 16px));
    max-height: min(350px, calc(100vh - 16px));
    overflow-y: auto;
    background: var(--bg);
    padding: 6px;
    border: 1px solid var(--key-border);
    border-radius: 8px;
    box-shadow: 0 8px 24px #0003;
  }
  .menu button {
    display: flex;
    justify-content: space-between;
    text-align: left;
    border: 0;
  }
  .hidden {
    display: none;
  }
  .notice {
    font-size: 13px;
    color: var(--cursor);
  }
  .dialog label {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .rename-form {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  @media (max-width: 900px) {
    .toolbar {
      padding: 6px 8px;
    }
    .brand {
      margin-right: 4px;
    }
    .board {
      padding: 8px;
    }
    .board-heading {
      margin-bottom: 8px;
    }
  }
</style>
