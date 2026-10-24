<script lang="ts">
  import { pointerDrop, startPointerDrag } from "../lib/pointer-drag.svelte";
  import type { MacroActionDto, MacroActionKind, MacroDto, MacroUsage } from "../lib/types";
  import { api, errorText } from "../lib/api";
  import { ui } from "../lib/state.svelte";
  import Field from "./Field.svelte";
  import { moveAction, macroSummary } from "../lib/macros";
  let {
    macros,
    onsave,
  }: {
    macros: MacroDto[];
    onsave: (index: number, actions: MacroActionDto[]) => Promise<boolean>;
  } = $props();
  let selected = $state(0);
  let draft: Record<number, string> = $state({});
  let ids: string[] = $state([]);
  let nextId = 0;
  let usage: MacroUsage | null = $state(null);
  let committedUsage: MacroUsage | null = $state(null);
  let error = $state("");
  const macro = $derived(macros.find((m) => m.index === selected));
  const dirty = $derived(Object.keys(draft).length > 0);
  const projectedTotal = $derived.by(() =>
    committedUsage && usage
      ? (ui.view?.macroBytes ?? 0) - committedUsage.bytes + usage.bytes
      : null,
  );
  const projectedActions = $derived(
    (macro?.actions ?? []).map((a, i) =>
      draft[i] === undefined
        ? a
        : a.kind === "text"
          ? { ...a, text: draft[i] }
          : { ...a, delay: Number(draft[i]) },
    ),
  );
  $effect(() => {
    ui.drafts.macros = dirty;
    ui.draftMacro = dirty ? selected : null;
  });
  $effect(() => {
    if (ui.revealMacro !== null) {
      selected = ui.revealMacro;
      ui.revealMacro = null;
    }
  });
  $effect(() => {
    const actions = projectedActions;
    let current = true;
    usage = null;
    if (
      actions.some(
        (a) => a.kind === "delay" && (!Number.isInteger(a.delay) || a.delay < 0 || a.delay > 65024),
      )
    ) {
      error = "Delay must be an integer from 0 to 65024 ms";
      return () => {
        current = false;
      };
    }
    error = "";
    void api.macroUsage(actions).then(
      (next) => {
        if (current) usage = next;
      },
      (e) => {
        if (current) error = errorText(e);
      },
    );
    if (ids.length !== actions.length) ids = actions.map(() => `action-${nextId++}`);
    return () => {
      current = false;
    };
  });
  $effect(() => {
    const actions = macro?.actions ?? [];
    let current = true;
    void api.macroUsage(actions).then(
      (next) => {
        if (current) committedUsage = next;
      },
      (e) => {
        if (current) error = errorText(e);
      },
    );
    return () => {
      current = false;
    };
  });
  function select(index: number): void {
    if (dirty) {
      ui.message = "Apply or revert the macro draft first";
      return;
    }
    selected = index;
    ids = [];
    ui.inspectorPaths = [];
  }
  async function commit(actions: MacroActionDto[], nextIds?: string[]): Promise<void> {
    if (await onsave(selected, actions)) {
      draft = {};
      if (nextIds) ids = nextIds;
      error = "";
    }
  }
  async function applyDraft(): Promise<void> {
    if (!macro) return;
    const actions = projectedActions;
    if (
      actions.some(
        (a) => a.kind === "delay" && (!Number.isInteger(a.delay) || a.delay < 0 || a.delay > 65024),
      )
    ) {
      error = "Delay must be an integer from 0 to 65024 ms";
      return;
    }
    try {
      await commit(actions);
    } catch (e) {
      error = errorText(e);
    }
  }
  function add(kind: MacroActionKind): void {
    if (!macro || dirty) return;
    void commit(
      [...macro.actions, { kind, keys: ["KC_NO"], text: "", delay: 1 }],
      [...ids, `action-${nextId++}`],
    );
  }
  function reorder(from: number, to: number): void {
    if (!macro || dirty) return;
    const next = [...ids];
    const [id] = next.splice(from, 1);
    next.splice(to, 0, id!);
    ui.inspectorPaths = [];
    void commit(moveAction(macro.actions, from, to), next);
  }
  function slotSet(m: MacroDto): boolean {
    return m.actions.length > 0;
  }
  function slotText(m: MacroDto): string {
    if (!slotSet(m)) return "";
    const name = ui.view?.names.macros[m.index];
    if (name) return name;
    return macroParts(m).join(" ");
  }
  function macroParts(m: MacroDto): string[] {
    if (!slotSet(m)) return [];
    return m.actions.slice(0, 4).flatMap((action, i) => macroPart(m.index, action, i));
  }
  function macroPart(index: number, action: MacroActionDto, i: number): string[] {
    if (action.kind === "text") return ['"' + action.text + '"'];
    if (action.kind === "delay") return [action.delay + " ms"];
    const prefix = action.kind === "down" ? "↓" : action.kind === "up" ? "↑" : "";
    return action.keys
      .map((key, k) => (key === "KC_NO" ? "" : ui.view?.keycodes["macro[" + index + "][" + i + "][" + (k + 1) + "]"]?.label || key))
      .filter(Boolean)
      .map((label) => prefix + label);
  }
  function slotKeys(e: KeyboardEvent, perColumn: number, pick: (index: number) => void): void {
    const row = selected % perColumn, column = Math.floor(selected / perColumn);
    const next = e.key === "ArrowUp" && row > 0 ? selected - 1
      : e.key === "ArrowDown" && row < perColumn - 1 ? selected + 1
      : e.key === "ArrowLeft" && column > 0 ? selected - perColumn
      : e.key === "ArrowRight" && column < 1 ? selected + perColumn
      : -1;
    if (next < 0) return;
    e.preventDefault();
    pick(next);
    if (selected === next) (e.currentTarget as HTMLElement).querySelector<HTMLElement>('[data-slot-index="' + next + '"]')?.focus();
  }</script>

<section class="slot-editor macro-slot-editor" data-slot-editor="macro"><!-- svelte-ignore a11y_no_noninteractive_element_interactions --><div class="slot-list" role="group" style="--per-column:8" aria-label="Macro slots" onkeydown={(e) => slotKeys(e, 8, select)}>{#each macros as m}<button class="slot-cell" data-slot-index={m.index} data-state={slotSet(m) ? "set" : "empty"} class:active={selected === m.index} class:dirty={dirty && selected === m.index} aria-pressed={selected === m.index} aria-label={"Macro slot M" + m.index} title={dirty && selected === m.index ? "Unapplied draft" : slotText(m) || "Empty"} onclick={() => select(m.index)}><span class="slot-id">{m.index}</span><span class="slot-desc">{#if ui.view?.names.macros[m.index]}{slotText(m)}{:else}{#each macroParts(m) as part}<span class="macro-chip">{part}</span>{/each}{/if}</span>{#if dirty && selected === m.index}<i class="draft-mark" aria-hidden="true"></i>{/if}</button>{/each}</div><div class="slot-form">{#if macro}<article class="slot-edit-panel" aria-label={`Editing macro M${selected}`}><div class="slot-edit-title"><div><span class="slot-kicker">Macro {selected}</span>{#if ui.view?.names.macros[selected]}<h3>{ui.view.names.macros[selected]}</h3>{/if}</div><div class="slot-title-actions"><button class="quiet-button" onclick={() => window.dispatchEvent(new CustomEvent("canvas-rename-macro", { detail: selected }))}>Rename</button>{#if dirty}<span class="slot-draft">Draft</span>{/if}</div></div><p class="macro-usage">{dirty && projectedTotal === null ? "Calculating bytes" : `${dirty ? projectedTotal : (ui.view?.macroBytes ?? 0)} bytes total${dirty ? " projected" : ""}`} <span aria-hidden="true">·</span> {ui.view?.macroCapacity === null ? "capacity unknown" : `${ui.view?.macroCapacity} byte capacity`}</p>{#if ui.view?.macroCapacity !== null && (dirty ? (projectedTotal ?? 0) : (ui.view?.macroBytes ?? 0)) > (ui.view?.macroCapacity ?? Infinity)}<p class="macro-alert" role="alert">Macro memory exceeded. No actions will be truncated.</p>{/if}<p class="macro-slot-usage">{usage ? `${usage.bytes} bytes in this slot${dirty ? " projected" : ""}` : "Calculating slot size"}</p><div class="macro-actions">{#each macro.actions as action, i (ids[i] ?? `row-${i}`)}<div class="action"><div class="row macro-action-head"><span class="action-index">{i + 1}</span><button disabled={dirty} title="Drag to reorder action" aria-label={`Drag action ${i + 1}`} style:touch-action="none" onpointerdown={(e) => startPointerDrag(e, { kind: "macro", slot: selected, index: i }, action.kind)}>↕</button><span class="action-title"><strong>{action.kind}</strong><small>{usage?.actionBytes[i] ?? 0} bytes</small></span><span class="action-spacer"></span><button disabled={dirty || i === 0} title="Move action up" aria-label={`Move action ${i + 1} up`} onclick={() => reorder(i, i - 1)}>↑</button><button disabled={dirty || i === macro!.actions.length - 1} title="Move action down" aria-label={`Move action ${i + 1} down`} onclick={() => reorder(i, i + 1)}>↓</button><button disabled={dirty} onclick={() => { ui.inspectorPaths = []; void commit(macro!.actions.filter((_, j) => j !== i), ids.filter((_, j) => j !== i)); }}>Delete</button></div>{#if action.kind === "text"}<label>Text <input value={draft[i] ?? action.text} oninput={(e) => (draft[i] = e.currentTarget.value)} /></label>{:else if action.kind === "delay"}<label>Delay <input type="number" min="0" max="65024" value={draft[i] ?? action.delay} oninput={(e) => (draft[i] = e.currentTarget.value)} /> ms</label>{:else}<div class="macro-keys">{#each action.keys as _, k}<div class="macro-key"><Field path={`macro[${selected}][${i}][${k + 1}]`} label={`Key ${k + 1}`} size="sm" /><button class="macro-remove" disabled={dirty} aria-label={`Remove key ${k + 1}`} title="Remove key from action" onclick={() => void commit(macro!.actions.map((a, j) => j === i ? { ...a, keys: a.keys.filter((_, n) => n !== k) } : a))}>−</button></div>{/each}<button class="keycap-add" disabled={dirty} aria-label="Add key" title="Add key" onclick={() => void commit(macro!.actions.map((a, j) => (j === i ? { ...a, keys: [...a.keys, "KC_NO"] } : a)))}>+</button></div>{/if}</div>{/each}</div><div class="row macro-add-actions">{#each ["tap", "down", "up", "text", "delay"] as kind}<button disabled={dirty} onclick={() => add(kind as MacroActionKind)}>+ {kind}</button>{/each}</div>{#if dirty}<div class="row macro-draft-actions"><button onclick={() => void applyDraft()}>Apply draft</button><button onclick={() => { draft = {}; error = ""; }}>Revert draft</button></div>{/if}{#if error}<p class="macro-alert" role="alert">{error}</p>{/if}</article>{/if}</div></section>


