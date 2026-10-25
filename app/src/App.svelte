<script lang="ts">
  import { onMount, tick, untrack } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { api, errorText } from "./lib/api";
  import { notify, ui, setView } from "./lib/state.svelte";
  import { PENDING_DELAY_MS } from "./lib/pending";
  import { effectiveTheme, nextThemeMode, readThemeMode, writeThemeMode, type ThemeMode } from "./lib/theme";
  import { neighbor } from "./lib/grid";
  import { emptyMatrixCells, matrixGeometry, readBoardView, writeBoardView, type BoardView } from "./lib/matrix";
  import { actionFor, isMacPlatform, primaryModifierDown, primaryModifierLabel, type Action } from "./lib/keymap";
  import { nextStep, readZoom, writeZoom, type ZoomSetting } from "./lib/zoom";
  import { keyboardRange, matrixRange } from "./lib/range";
  import { COMMANDS, commandReason } from "./lib/palette";
  import { PANELS, type Panel } from "./lib/modes";
  import { comboOverlays } from "./lib/combos";
  import { changeCount, changeGroups, compareErrorText, type ChangeJump } from "./lib/changes";
  import { renderLayerSheet } from "./lib/layer-export";
  import type {
    OpDto,
    DiffBase,
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
  import DragPreview from "./components/DragPreview.svelte";
  import Inspector from "./components/Inspector.svelte";
  import KeycodeInput from "./components/KeycodeInput.svelte";
  import ChangesSheet from "./components/ChangesSheet.svelte";
  import Palette from "./components/Palette.svelte";
  import ComboEditor from "./components/ComboEditor.svelte";
  import MacroEditor from "./components/MacroEditor.svelte";
  import TapDanceEditor from "./components/TapDanceEditor.svelte";
  import KeyOverrideEditor from "./components/KeyOverrideEditor.svelte";
  import AltRepeatEditor from "./components/AltRepeatEditor.svelte";
  import Field from "./components/Field.svelte";
  import { focusScope } from "./lib/focus";
  import Icon from "./components/Icon.svelte";
  let canvasPanel = $state(false);
  let boardZoom: ZoomSetting = $state(readZoom());
  let boardScale = $state(1);
  function setZoom(z: ZoomSetting): void {
    boardZoom = z;
    writeZoom(z);
  }
  const canvasPanelNames: Record<string, string> = { combos:'Combos', macros:'Macros', tapDances:'Tap dances', keyOverrides:'Key overrides', altRepeat:'Alt repeat' };
  const canvasLedes: Record<string, string> = { combos: 'Press several keys together to send a different key.', macros: 'Send a sequence of keys or text from one key.', tapDances: 'Give one key different actions for tap, hold and double tap.', keyOverrides: "Replace a key when it's pressed with certain modifiers.", altRepeat: 'Choose what the Alt Repeat key sends after each key.' };
  onMount(() => { const useBoard = () => { canvasPanel = false; ui.sideVisible = true; }; const renameMacro = (event: CustomEvent<number>) => { const index = event.detail; rename = { kind: 'macro', index, name: ui.view?.names.macros[index] ?? '' }; }; window.addEventListener('canvas-board-triggers', useBoard); window.addEventListener('canvas-rename-macro', renameMacro as EventListener); return () => { window.removeEventListener('canvas-board-triggers', useBoard); window.removeEventListener('canvas-rename-macro', renameMacro as EventListener); }; });
  let canvasBoardMenu = $state(false);
  const canvasMenuLabels: Record<string, string> = { clearKeys: 'Clear keys', transparentKeys: 'Make transparent', copyKeys: 'Copy', cutKeys: 'Cut', pasteKeys: 'Paste', swapKeys: 'Swap with…', editKey: 'Type keycode…', copyLayer: 'Copy layer', pasteLayer: 'Paste layer', clearLayer: 'Clear layer', transparentLayer: 'Make layer transparent', swapLayer: 'Swap layer with…', moveLayer: 'Move layer to…', mirrorLeftToRight: 'Mirror left to right', mirrorRightToLeft: 'Mirror right to left', mirrorSwap: 'Swap halves' };
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
  // The Keymap inspector is a floating card: 240 to 420 px, never wider than the window less 2 x 16 px.
  const sideMax = $derived(Math.max(240, Math.min(420, viewportWidth - 32)));
  const pickerMax = $derived(Math.max(120, viewportHeight - 240));
  const sideWidth = $derived(Math.min(panelSizes.side, sideMax));
  function readFlag(key: string): boolean {
    try {
      return localStorage.getItem(key) === "true";
    } catch {
      return false;
    }
  }
  let inspectorCollapsed = $state(readFlag("vile.inspectorCollapsed"));
  function setInspectorCollapsed(value: boolean): void {
    inspectorCollapsed = value;
    try {
      localStorage.setItem("vile.inspectorCollapsed", String(value));
    } catch {
      /* the card still works for this session */
    }
  }
  // Fit leaves room on the right for the expanded card when the window is wide enough for both.
  const inspectorReserve = $derived(
    !canvasPanel && !inspectorCollapsed && viewportWidth >= sideWidth + 560 ? sideWidth + 32 : 0,
  );
  // A collapsed pill sits on the layer ribbon's row. Where the ribbon reaches the pill, the pill moves
  // to the row below the ribbon, and fit keeps the keys out from under it (see collapsedPlacement).
  let workspaceEl: HTMLElement | undefined = $state();
  let asideEl: HTMLElement | undefined = $state();
  let viewportEl: HTMLElement | undefined = $state();
  let pillPlacement: { top: number; reserve: number } | null = $state(null);
  // Client rectangles of the things drawn over the board, for revealing the cursor key.
  function boardOccluders(): { l: number; t: number; r: number; b: number }[] {
    const out: { l: number; t: number; r: number; b: number }[] = [];
    const add = (el: Element | null | undefined) => {
      if (!el) return;
      const q = el.getBoundingClientRect();
      if (q.width > 0 && q.height > 0) out.push({ l: q.left, t: q.top, r: q.right, b: q.bottom });
    };
    if (!canvasPanel) add(asideEl);
    add(workspaceEl?.querySelector(".floating-category-bar"));
    add(workspaceEl?.querySelector(".floating-popup"));
    add(workspaceEl?.querySelector(".zoom-controls"));
    return out;
  }
  function collapsedPlacement(): { top: number; reserve: number } | null {
    if (!workspaceEl || !asideEl || !viewportEl || canvasPanel || !inspectorCollapsed) return null;
    const ribbonEl = workspaceEl.querySelector<HTMLElement>(".canvas-layer-switcher .layers");
    if (!ribbonEl) return null;
    const ws = workspaceEl.getBoundingClientRect();
    const pill = asideEl.getBoundingClientRect();
    const ribbon = ribbonEl.getBoundingClientRect();
    // Test the pill on its own row (top inset --space-4), so the result does not depend on where it is now.
    const rowTop = ws.top + 16;
    const rowBottom = rowTop + pill.height;
    const pillLeft = ws.right - 16 - pill.width;
    const onRibbon =
      ribbon.right > pillLeft - 8 && ribbon.top < rowBottom && ribbon.bottom > rowTop;
    if (!onRibbon) return null;
    const top = ribbon.bottom - ws.top + 8;
    const viewportTop = viewportEl.getBoundingClientRect().top - ws.top;
    return { top, reserve: Math.max(0, top + pill.height + 8 - viewportTop) };
  }
  $effect(() => {
    void viewportWidth;
    void inspectorCollapsed;
    void canvasPanel;
    void ui.selection.length;
    void ui.layer;
    void ui.view;
    void boardView;
    const placement = untrack(collapsedPlacement);
    untrack(() => {
      pillPlacement = placement;
    });
  });
  const pickerHeight = $derived(Math.min(panelSizes.picker, pickerMax));
  $effect(() => {
    const saved = JSON.stringify(panelSizes);
    try {
      localStorage.setItem("vile.panelSizes", saved);
    } catch {
      /* Editing still works without preference storage. */
    }
  });
  // Theme: the stored mode (system, light or dark) resolves to the effective theme on
  // <html data-theme>, which the stylesheets and the PNG export read.
  let themeMode: ThemeMode = $state(readThemeMode());
  // Board view: the physical layout, or the electrical matrix. Per viewer, never saved to the layout.
  let boardView: BoardView = $state(readBoardView());
  const matrixKeys = $derived(ui.view?.board ? matrixGeometry(ui.view.board.keys) : []);
  const matrixEmpties = $derived(ui.view?.board ? emptyMatrixCells(ui.view.board.keys) : []);
  function setBoardView(view: BoardView): void {
    boardView = view;
    writeBoardView(view);
  }
  const systemDark = typeof matchMedia === "function" ? matchMedia("(prefers-color-scheme: dark)") : null;
  let prefersDark = $state(systemDark?.matches ?? true);
  $effect(() => {
    if (!systemDark) return;
    const update = () => (prefersDark = systemDark.matches);
    systemDark.addEventListener("change", update);
    return () => systemDark.removeEventListener("change", update);
  });
  $effect(() => {
    document.documentElement.dataset.theme = effectiveTheme(themeMode, prefersDark);
  });
  const themeIcon = $derived(themeMode === "system" ? "half" : themeMode === "light" ? "sun" : "moon");
  const themeLabel = $derived(
    themeMode === "system"
      ? "Theme: system (click for light)"
      : themeMode === "light"
        ? "Theme: light (click for dark)"
        : "Theme: dark (click for system)",
  );
  function cycleTheme(): void {
    themeMode = nextThemeMode(themeMode);
    writeThemeMode(themeMode);
  }
  let busy = $state(false);
  // Shows the busy toast only when a call is still running after PENDING_DELAY_MS.
  let busyShown = $state(false);
  $effect(() => {
    if (!busy) {
      busyShown = false;
      return;
    }
    const timer = setTimeout(() => (busyShown = true), PENDING_DELAY_MS);
    return () => clearTimeout(timer);
  });
  let toastVisible = $state(false);
  $effect(() => {
    void ui.messageSeq;
    if (!ui.message) {
      toastVisible = false;
      return;
    }
    toastVisible = true;
    if (ui.messageError) return;
    const timer = setTimeout(() => (toastVisible = false), 3000);
    return () => clearTimeout(timer);
  });
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
  let review: { revision: number; saveAs: boolean; error: string; focusSave: boolean } | null = $state(null);
  let unsaved: "open" | "close" | null = $state(null);
  let afterSave: "open" | "close" | null = $state(null);
  let rename: { kind: "layer" | "macro" | "combo"; index: number; name: string } | null =
    $state(null);
  let remapsText = $state("");
  let settings = $state(false);
  let blankPress: { pointer: number; x: number; y: number; moved: boolean } | null = null;
  function blankTarget(e: PointerEvent): boolean {
    if (document.querySelector("[data-picker-open=true]") || (e.target instanceof Element && e.target.closest(".floating-picker"))) return false;
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
        ".app,.toolbar,.spacer,.layers,.workspace,.board,.board-heading,.keyboard-viewport,aside,.panels,.picker-dock,.picker,.picker-head,.tiles",
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
  const drafts = $derived(Object.values(ui.drafts).some(Boolean));
  const fileName = $derived(ui.view?.path?.split(/[\\/]/).pop() || "this layout");
  const groups = $derived(ui.view ? changeGroups(ui.view) : []);
  const total = $derived(changeCount(groups));
  const git = $derived(ui.view?.diffBase === "git");
  const countText = $derived(
    git
      ? total
        ? `${total} since commit`
        : "Matches commit"
      : total
        ? `${total} ${total === 1 ? "change" : "changes"}`
        : "No changes",
  );
  const countTitle = $derived(
    git
      ? total
        ? `Changes since the last Git commit. ${ui.view?.dirty ? "Some are not saved yet." : "All of them are saved."} Click to review.`
        : `Matches the last Git commit.${ui.view?.dirty ? " Not saved yet." : ""}`
      : total
        ? "Changes since the last save. Click to review."
        : "Nothing changed since the last save.",
  );
  const legendTitle = $derived(`A dot marks what differs from ${fileName} ${git ? "in the last Git commit" : "as last saved"}.`);
  const cells = $derived(ui.view?.layers[ui.layer] ?? []);
  const encoderPaths = $derived.by(() =>
    Object.keys(ui.view?.keycodes ?? {}).filter((path) =>
      path.startsWith(`encoder_layout[${ui.layer}]`),
    ),
  );
  const visibleEncoderPaths = $derived.by(() => {
    const visible = ui.view?.board?.encoderIndices;
    if (!Array.isArray(visible)) return encoderPaths;
    return encoderPaths.filter((path) => {
      const match = /^encoder_layout\[\d+\]\[(\d+)\]/.exec(path);
      return match !== null && visible.includes(Number(match[1]));
    });
  });
  function encoderLabel(path: string): string {
    const match = /^encoder_layout\[\d+\]\[(\d+)\]\[(\d+)\]$/.exec(path);
    if (!match) return path;
    return `Knob ${Number(match[1]) + 1} ${match[2] === "0" ? "clockwise" : "counterclockwise"}`;
  }
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
    canvasPanel = true;
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
    ui.anchor = null;
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
      notify(errorText(e), true);
      return false;
    }
  }
  async function apply(op: OpDto, done = "Updated"): Promise<boolean> {
    if (busy) return false;
    busy = true;
    try {
      setView(await api.applyOp(op));
      notify(ui.view?.warnings.length ? ui.view.warnings.join("; ") : done);
      return true;
    } catch (e) {
      notify(errorText(e), true);
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
      notify(errorText(e), true);
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
    ui.anchor = null;
    ui.selection = [];
    ui.inspectorPaths = [];
    ui.baseTarget = false;
    ui.comboEditing = null;
  }
  function onkeyclick(id: string, toggle: boolean): void {
    if (ui.comboEditing === null) canvasPanel = false;
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
          notify("A combo supports at most four distinct trigger keycodes");
          return;
        }
        if (cell.code === 0) {
          notify("No key cannot be a combo trigger");
          return;
        }
        keys.push(cell.wire);
      }
      while (keys.length < 4) keys.push("KC_NO");
      void apply({ type: "setCombo", index: activeCombo.index, keys, output: activeCombo.output });
      return;
    }
    ui.cursor = id;
    ui.anchor = id;
    ui.inspectorPaths = [];
    ui.baseTarget = false;
    if (ui.swapMark) {
      const a = ui.swapMark;
      ui.swapMark = null;
      void apply({ type: "swapKeys", a, b: [{ layer: ui.layer, key: id }] });
      ui.selection = [id];
      return;
    }
    if (!toggle) {
      ui.selection = [id];
      return;
    }
    ui.selection = ui.selection.includes(id)
      ? ui.selection.filter((key) => key !== id)
      : [...ui.selection, id];
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
      notify("Apply or revert the macro draft before assigning keys");
      return;
    }
    if (ui.comboEditing !== null && activeFeaturePath[0] !== `combo[${ui.comboEditing}][4]`) {
      notify("Select this combo's output field before assigning from the picker");
      return;
    }
    if (!selectedPaths.length) {
      notify("Select a key or keycode field first");
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
      notify("Apply or revert the macro draft first");
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
        notify(`Opened ${view.path}`);
        void refreshGeometry();
      }
    } catch (e) {
      notify(errorText(e), true);
    } finally {
      busy = false;
    }
  }
  function beginSave(saveAs = false): void {
    if (drafts) {
      notify("Apply or revert the macro draft before saving");
      return;
    }
    if (!ui.view) return;
    review = { revision: ui.view.revision, saveAs, error: "", focusSave: true };
  }
  function openChanges(): void {
    if (!ui.view) return;
    review = { revision: ui.view.revision, saveAs: false, error: "", focusSave: false };
  }
  function jump(target: ChangeJump): void {
    review = null;
    afterSave = null;
    if (target.kind === "slot") {
      switchPanel(target.panel);
      ui.sideVisible = true;
      if (target.panel === "macros") ui.revealMacro = target.index;
      else ui.revealSlot = { panel: target.panel, index: target.index };
      return;
    }
    canvasPanel = false;
    selectLayer(target.layer);
    if (target.kind === "keys" && target.keyIds.length) {
      ui.selection = [...target.keyIds];
      ui.cursor = target.keyIds[0]!;
      ui.anchor = ui.cursor;
      ui.revealTick += 1;
      // After the sheet's focus scope returns focus to the toolbar, move it to the key.
      setTimeout(() => document.querySelector<SVGElement>(`.board [data-key-id="${ui.cursor}"]`)?.focus({ preventScroll: true }), 0);
    }
  }
  async function compareWith(base: DiffBase): Promise<void> {
    if (busy || !ui.view || ui.view.diffBase === base) return;
    busy = true;
    try {
      setView(await api.setDiffBase(base));
    } catch (e) {
      notify(base === "git" ? compareErrorText(errorText(e), fileName) : errorText(e), true);
    } finally {
      busy = false;
    }
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
      notify("The document changed. Review its current diff before saving");
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
        notify(`Saved ${view.path}`);
      }
    } catch (e) {
      const saveError = errorText(e);
      await refresh();
      notify(saveError, true);
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
    if (id === "exportPng" && ui.view) {
      const view = ui.view;
      busy = true;
      notify("Rendering layer PNG…");
      try {
        const { png, layerCount } = await renderLayerSheet(view);
        const path = await api.exportLayers(png, view.revision);
        notify(
          path ? `Exported ${layerCount} ${layerCount === 1 ? "layer" : "layers"} to ${path}` : "PNG export canceled",
        );
      } catch (e) {
        notify(errorText(e), true);
      } finally {
        busy = false;
      }
      return;
    }
    if (id === "undo" || id === "redo") {
      if (drafts) {
        notify("Apply or revert the macro draft first");
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
        notify("Select one key, then choose its swap target");
        return;
      }
      ui.swapMark = ui.selection.map((key) => ({ layer: ui.layer, key }));
      notify("Click the key to swap with. Escape cancels.");
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
      await compareWith(id === "diffGit" ? "git" : "disk");
      return;
    }
    if (id === "panelDiff") {
      openChanges();
      return;
    }
    const panels: Record<string, Panel> = {
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
    if (id === "zoomIn" || id === "zoomOut" || id === "zoomFit") {
      if (!canvasPanel) setZoom(id === "zoomFit" ? "fit" : nextStep(boardScale, id === "zoomIn" ? 1 : -1));
      return;
    }
  }
  function onkeydown(e: KeyboardEvent): void {
    if (canvasPanel && e.key.startsWith("Arrow")) return;
    if (e.key === "Escape" && (canvasBoardMenu)) { e.preventDefault();  canvasBoardMenu = false; return; }
    if (e.key === "Escape" && document.querySelector("[data-picker-open=true]")) return;
    if (e.key === "Escape" && dismissInteraction(false)) {
      e.preventDefault();
      return;
    }
    if (review && primaryModifierDown(e, navigator.platform) && e.key.toLowerCase() === "s") {
      e.preventDefault();
      void confirmSave();
      return;
    }
    if (menu || review || unsaved || rename || settings) return;
    const target = e.target;
    if (target instanceof Element) {
      if (target.closest("[role=dialog],input,textarea,select,[contenteditable]")) return;
      const control = target.closest("button,summary,a[href],[role=button],[role=menuitem]");
      // Arrows always reach the board: a focused toolbar button (a zoom button, say) must not swallow them.
      if (
        control &&
        !target.closest("[data-key-id]") &&
        !e.key.startsWith("Arrow") &&
        (!primaryModifierDown(e, navigator.platform) || !["s", "o", "k", "z", "=", "+", "-", "_", "0"].includes(e.key.toLowerCase()))
      )
        return;
    }
    if (ui.mode !== "normal" && ui.mode !== "visual") return;
    const action = actionFor(ui.mode, e, ui.vimKeys, ui.vimRemaps, navigator.platform);
    if (!action) return;
    e.preventDefault();
    if (action.kind === "zoomIn" || action.kind === "zoomOut" || action.kind === "zoomFit") {
      if (!canvasPanel) setZoom(action.kind === "zoomFit" ? "fit" : nextStep(boardScale, action.kind === "zoomIn" ? 1 : -1));
      return;
    }
    if (action.kind === "move") {
      const previous = ui.cursor;
      const keys = boardView === "matrix" ? matrixKeys : ui.view?.board?.keys ?? [];
      ui.cursor = previous ? neighbor(keys, previous, action.dir) : keys[0]?.id ?? "";
      ui.revealTick += 1;
      if (action.extend && ui.cursor) {
        // Shift+arrow keeps the anchor and selects the range it spans with the cursor.
        const anchor = ui.anchor ?? (previous || ui.cursor);
        ui.anchor = anchor;
        ui.selection = boardView === "matrix" ? matrixRange(keys, anchor, ui.cursor) : keyboardRange(keys, anchor, ui.cursor);
      } else {
        ui.anchor = ui.cursor || null;
        ui.selection = [ui.cursor].filter(Boolean);
      }
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
      switchPanel(PANELS[(PANELS.indexOf(ui.panel) + 1) % PANELS.length] ?? "combos");
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
      ui.cursor = "";
      ui.anchor = null;
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
    if (e.target instanceof Element && !e.target.closest(".popover-anchor")) {  canvasBoardMenu = false; }
    if (menu && e.target instanceof Element && !e.target.closest(".menu")) menu = null;
  }}
  bind:innerWidth={viewportWidth}
  bind:innerHeight={viewportHeight}
/>
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="app canvas-app"
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
  <header class="toolbar canvas-toolbar">
    <strong class="brand">vile<span class="brand-dot">.</span></strong>
    <div class="document-title"><Icon name="file"/><span>{ui.view?.path?.split(/[\\/]/).pop() || 'No layout open'}</span><span class="document-type">Vial layout</span></div>
    <div class="history-actions">
      <button class="icon-button" aria-label="Undo" title={`Undo · ${primaryModifierLabel(navigator.platform)}Z`} disabled={!!reason('undo')} onclick={() => void command('undo')}><Icon name="undo"/></button>
      <button class="icon-button" aria-label="Redo" title={`Redo · ${primaryModifierLabel(navigator.platform)}${primaryModifierLabel(navigator.platform) === '⌘' ? '⇧' : 'Shift+'}Z`} disabled={!!reason('redo')} onclick={() => void command('redo')}><Icon name="redo"/></button>
    </div>
    {#if drafts}<button class="draft-indicator" onclick={() => { switchPanel('macros'); ui.sideVisible = true; ui.revealMacro = ui.draftMacro; }}>Macro draft M{ui.draftMacro}</button>{/if}
    <span class="spacer"></span>
    <button class="command-button" onclick={() => (ui.mode = 'palette')}><Icon name="command"/>Commands<kbd>{primaryModifierLabel(navigator.platform)}K</kbd></button>
    <nav class="board-view-switch" aria-label="Board view">
      <button class:active={boardView === "keyboard"} aria-pressed={boardView === "keyboard"} title="Physical layout from the definition" onclick={() => setBoardView("keyboard")}>Keyboard</button>
      <button class:active={boardView === "matrix"} aria-pressed={boardView === "matrix"} title="Electrical rows and columns, as stored in the .vil file" onclick={() => setBoardView("matrix")}>Matrix</button>
    </nav>
    <button class="icon-button theme-button" aria-label={themeLabel} title={themeLabel} onclick={cycleTheme}><Icon name={themeIcon}/></button>
    <button class="pending-button" title={countTitle} aria-label={countText} onclick={openChanges}><span class:has-changes={ui.view?.dirty} class="status-dot"></span><span class="count-wide">{countText}</span><span class="count-narrow" aria-hidden="true">{total}</span></button>
    <button class="save-button" title={reason('save') || `Review the changes, then save · ${primaryModifierLabel(navigator.platform)}S`} disabled={!!reason('save')} onclick={() => void command('save')}><span class="save-wide">Review &amp; save</span><span class="save-narrow">Save</span><kbd>{primaryModifierLabel(navigator.platform)}S</kbd></button>
  </header>
  {#if ui.view && ui.view.board}
    <div class="canvas-context">
      <div class="context-actions">
        <nav class="workspace-switcher" aria-label="Workspace panels">
          <button class:active={!canvasPanel} aria-pressed={!canvasPanel} onclick={() => { canvasPanel = false; ui.inspectorPaths = []; ui.baseTarget = false; ui.comboEditing = null; }}>Keymap</button>
          {#each PANELS as panel}<button class:active={canvasPanel && ui.panel === panel} aria-pressed={canvasPanel && ui.panel === panel} onclick={() => { switchPanel(panel as Panel); ui.sideVisible = true; }}>{canvasPanelNames[panel]}</button>{/each}
        </nav>
      </div>
    </div>
    <div
      class="workspace" class:feature-workspace={canvasPanel} data-workspace={canvasPanel ? ui.panel : "keymap"}
      bind:this={workspaceEl}
    >
      <main class="board" class:hidden={canvasPanel}>
        <div class="board-heading"><div><span class="canvas-eyebrow">Layer {ui.layer}</span><h2>{ui.view.names.layers[ui.layer] || 'Layer ' + ui.layer}</h2></div><div class="canvas-layer-switcher">      <LayerBar
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
</div></div>
        {#if ui.comboEditing !== null}<p class="notice">
            Editing combo {ui.comboEditing} triggers. Click board keys to toggle their keycodes.
            <button onclick={() => (ui.comboEditing = null)}>Done</button>
          </p>{/if}
        <div class="keyboard-viewport" bind:this={viewportEl} style:padding-right={inspectorReserve ? `${inspectorReserve}px` : null} style:padding-top={pillPlacement?.reserve ? `${pillPlacement.reserve}px` : null}>
          <Keyboard
            keys={boardView === "matrix" ? matrixKeys : ui.view.board.keys}
            matrix={boardView === "matrix"}
            empties={boardView === "matrix" ? matrixEmpties : []}
            fit={{ maxWidth: 1040, maxHeight: compact ? 360 : 455 }}
            zoom={boardZoom}
            onzoom={setZoom}
            bind:scale={boardScale}
            occluders={boardOccluders}
            revealTick={ui.revealTick}
            {cells}
            cursor={ui.cursor}
            selection={comboSelection}
            {onkeyclick}
            locked={ui.comboEditing !== null || busy}
            onmarquee={(ids, additive, end) => {
              ui.selection = additive ? Array.from(new Set([...ui.selection, ...ids])) : ids;
              // The key the marquee ended nearest becomes both cursor and anchor.
              if (end) ui.cursor = end;
              ui.anchor = end;
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
            overlays={ui.showCombos && boardView === "keyboard" ? comboOverlays(ui.view.combos, cells, ui.view.board.keys) : []}
          />
        </div>
        <div class="board-caption">
          <span title={legendTitle}><i class="legend-dot"></i>{git ? "Changed since commit" : "Changed since save"}</span>
          <span class="board-hints">{`Shift- or ${isMacPlatform(navigator.platform) ? "⌘" : "Ctrl"}-click to add or remove`} <span class="hint-separator">·</span> Double-click to type a keycode</span>
          <div class="zoom-controls" role="group" aria-label="Zoom">
            <button class="icon-button" aria-label="Zoom out" title={`Zoom out · ${primaryModifierLabel(navigator.platform)}−`} disabled={boardScale <= 0.25 + 0.001} onclick={() => setZoom(nextStep(boardScale, -1))}><Icon name="minus" size={14} /></button>
            <button class="zoom-level" aria-pressed={boardZoom === "fit"} aria-label={`Zoom ${Math.round(boardScale * 100)}%. Fit to window`} title={`Fit to window · ${primaryModifierLabel(navigator.platform)}0`} onclick={() => setZoom("fit")}>{boardZoom === "fit" ? "Fit" : `${Math.round(boardScale * 100)}%`}</button>
            <button class="icon-button" aria-label="Zoom in" title={`Zoom in · ${primaryModifierLabel(navigator.platform)}=`} disabled={boardScale >= 4 - 0.001} onclick={() => setZoom(nextStep(boardScale, 1))}><Icon name="plus" size={14} /></button>
          </div>
        </div>
        {#if visibleEncoderPaths.length}<details
            ontoggle={(e) => {
              if (
                !e.currentTarget.open &&
                ui.inspectorPaths[0]?.startsWith(`encoder_layout[${ui.layer}]`)
              )
                ui.inspectorPaths = [];
            }}
          >
            <summary>Knobs</summary
            >{#if !Array.isArray(ui.view.board.encoderIndices)}<p class="hint">
                The keyboard definition is unavailable, so these encoder bindings are not confirmed as installed knobs.
              </p>{/if}{#each visibleEncoderPaths as path}<Field plain
                {path}
                label={encoderLabel(path)}
              />{/each}
          </details>{/if}
      </main>
      <div class="floating-picker-host" class:hidden={canvasPanel && !activeFeaturePath.length} style:right={`${inspectorReserve ? inspectorReserve + 22 : 22}px`}><Picker view={ui.view} onchoose={assign} choosingBase={ui.baseTarget} target={targetLabel} /></div>
      {#if !canvasPanel && !inspectorCollapsed}<div class="inspector-edge" style:right={`${sideWidth + 16 - 4}px`}><ResizeHandle
            orientation="vertical"
            label="Resize editor panel"
            value={sideWidth}
            min={240}
            max={sideMax}
            initial={390}
            onresize={(value) => (panelSizes.side = value)}
          /></div>{/if}
      <aside
        bind:this={asideEl}
        class:hidden={!ui.sideVisible}
        class:floating-card={!canvasPanel}
        class:collapsed={!canvasPanel && inspectorCollapsed}
        class:feature-editor={canvasPanel}
        class:has-feature-inspection={activeFeaturePath.length > 0}
        style:--inspector-w={`${sideWidth}px`}
        style:top={!canvasPanel && inspectorCollapsed && pillPlacement ? `${pillPlacement.top}px` : null}
      >
        <div class="inspector-heading">{#if canvasPanel}<h1>{canvasPanelNames[ui.panel]}</h1>{#if canvasLedes[ui.panel]}<p class="feature-lede">{canvasLedes[ui.panel]}</p>{/if}{:else}<span class="canvas-eyebrow">Key inspector</span>{#if inspectorCollapsed && ui.selection.length}<span class="hint">{ui.selection.length} selected</span>{/if}{/if}{#if !canvasPanel && ui.selection.length && !inspectorCollapsed}<button class="icon-button" aria-label="Clear selection" title="Clear selection · Esc" onclick={() => dismissInteraction(true)}><Icon name="close" size={14}/></button>{/if}{#if !canvasPanel}<button class="icon-button inspector-toggle" aria-expanded={!inspectorCollapsed} aria-label={inspectorCollapsed ? "Show inspector" : "Hide inspector"} title={inspectorCollapsed ? "Show inspector" : "Hide inspector"} onclick={() => setInspectorCollapsed(!inspectorCollapsed)}><Icon name="chevron" size={14} /></button>{/if}</div>
        <div class="canvas-selection-inspector" class:hidden={canvasPanel && !activeFeaturePath.length}>
        {#if inspected.length}<Inspector
            keys={inspected}
            layerCount={ui.view.layerCount}
            onpatch={patch}
            onbase={() => { ui.baseTarget = !ui.baseTarget; if (ui.baseTarget) window.dispatchEvent(new Event("canvas-find-key")); }}
            choosing={ui.baseTarget}
          />{:else}<p class="hint">
            Select a key on the board to edit its tap action, modifiers and hold behavior.
          </p>{/if}
        </div>
        <div class="canvas-feature-editor" class:hidden={!canvasPanel}>
        <div class:hidden={ui.panel !== "combos"}>
          <ComboEditor
            combos={ui.view.combos}
            onoperation={apply}
            onrename={(index) =>
              (rename = { kind: "combo", index, name: ui.view?.names.combos[index] ?? "" })}
          />
        </div>
        <div class:hidden={ui.panel !== "macros"}>
          {#key ui.generation}<MacroEditor macros={ui.view.macros} onsave={saveMacro} onoperation={apply} />{/key}
        </div>
        <div class:hidden={ui.panel !== "tapDances"}>
          <TapDanceEditor tapDances={ui.view.tapDances} onsave={saveTapDance} onoperation={apply} />
        </div>
        <div class:hidden={ui.panel !== "keyOverrides"}>
          <KeyOverrideEditor
            keyOverrides={ui.view.keyOverrides}
            layerCount={ui.view.layerCount}
            onsave={saveOverride}
            onoperation={apply}
          />
        </div>
        <div class:hidden={ui.panel !== "altRepeat"}>
          <AltRepeatEditor altRepeatKeys={ui.view.altRepeatKeys} onsave={saveAltRepeat} onoperation={apply} />
        </div>
        </div>
      </aside>
    </div>
  {:else}<main class="empty">
      <h1>Edit your keyboard visually.</h1>
      <button onclick={() => void command("open")}>Open a Vial layout <kbd>{primaryModifierLabel(navigator.platform)}O</kbd></button>
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
  <div class="toast" class:shown={toastVisible || busyShown} role="status" aria-live="polite">
    <span>{busy ? "Working…" : ui.message === "Definition refreshed" ? "" : ui.message}</span>{#if ui.messageError}<button class="toast-dismiss" aria-label="Dismiss message" onclick={() => (toastVisible = false)}>Dismiss</button>{/if}
  </div>
  {#if ui.view?.warnings.length}<div class="warnings" role="status">
      {ui.view.warnings.join(" · ")}
    </div>{/if}
  <DragPreview />
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
         data-menu-divider={["copyKeys", "swapKeys", "copyLayer", "mirrorLeftToRight"].includes(c.id)}><span class="menu-label">{canvasMenuLabels[c.id] || c.title}</span>{#if reason(c.id)}<small class="menu-reason">{reason(c.id)}</small>{/if}<kbd>{c.shortcut}</kbd></button
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
{#if review && ui.view}<ChangesSheet view={ui.view} {groups} {total} {review} {busy} {drafts}
    oncompare={(base) => void compareWith(base)} onjump={jump} onsave={() => void confirmSave()}
    onsaveas={() => { if (review) review = { ...review, saveAs: true }; }}
    oncancel={() => { review = null; afterSave = null; }} />{/if}
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
      <p>Arrows, digits, Delete and conventional platform shortcuts work by default.</p>
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
              notify(errorText(e), true);
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
    grid-template-columns: minmax(0, 1fr);
    position: relative;
    flex: 1;
    min-height: 0;
  }
  .inspector-edge {
    position: absolute;
    top: var(--space-4);
    bottom: calc(var(--space-4) + 40px);
    width: 8px;
    z-index: 6;
  }
  .board {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    padding: 16px;
    overflow: auto;
  }
  .keyboard-viewport {
    flex: 1 1 0;
    min-width: 0;
    min-height: 0;
    display: flex;
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
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
    margin-bottom: 8px;
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
