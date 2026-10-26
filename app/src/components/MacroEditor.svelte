<script lang="ts">
  import { usesOf } from "../lib/slots";
  import { pointerDrop, startPointerDrag } from "../lib/pointer-drag.svelte";
  import type { MacroActionDto, MacroActionKind, MacroDto, MacroUsage } from "../lib/types";
  import { api, errorText } from "../lib/api";
  import { notify, ui } from "../lib/state.svelte";
  import Field from "./Field.svelte";
  import FeatureSlotList from "./FeatureSlotList.svelte";
  import type { OpDto } from "../lib/types";
  import Icon from "./Icon.svelte";
  import { moveAction, macroSummary } from "../lib/macros";
  import { createPendingTracker } from "../lib/pending";
  import { currentLabels, macroSet, macroText } from "../lib/slots";
  import SlotUses from "./SlotUses.svelte";
  import { changedSlots } from "../lib/changes";
  let {
    macros,
    onsave,
    onoperation,
  }: {
    macros: MacroDto[];
    onsave: (index: number, actions: MacroActionDto[]) => Promise<boolean>;
    onoperation: (op: OpDto) => Promise<boolean>;
  } = $props();
  let selected = $state(0);
  const changed = $derived(changedSlots(ui.view).macro);
  let draft: Record<number, string> = $state({});
  let ids: string[] = $state([]);
  let nextId = 0;
  let usage: MacroUsage | null = $state(null);
  let usagePending = $state(false);
  // The last usage stays rendered until the next one arrives. usagePending turns on
  // only after PENDING_DELAY_MS, so fast edits never flash "Calculating".
  const usageTracker = createPendingTracker((pending) => (usagePending = pending));
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
    if (
      actions.some(
        (a) => a.kind === "delay" && (!Number.isInteger(a.delay) || a.delay < 0 || a.delay > 65024),
      )
    ) {
      usageTracker.cancel();
      error = "Delay must be an integer from 0 to 65024 ms";
      return;
    }
    error = "";
    const request = usageTracker.start();
    void api.macroUsage(actions).then(
      (next) => {
        if (usageTracker.finish(request)) usage = next;
      },
      (e) => {
        if (usageTracker.finish(request)) error = errorText(e);
      },
    );
    if (ids.length !== actions.length) ids = actions.map(() => `action-${nextId++}`);
    return () => usageTracker.cancel();
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
      notify("Apply or revert the macro draft first");
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
</script>

<section class="slot-editor macro-slot-editor" data-slot-editor="macro"><!-- svelte-ignore a11y_no_noninteractive_element_interactions --><FeatureSlotList kind="macro" label="Macro" perColumn={8} selected={selected} onselect={select} onoperation={onoperation} onrename={(index) => window.dispatchEvent(new CustomEvent("canvas-rename-macro", { detail: index }))} items={macros.map((slot) => ({ index: slot.index, set: macroSet(slot), text: macroText(slot, currentLabels(ui.view), ui.view?.names.macros[slot.index]), changed: changed.has(slot.index), uses: usesOf(ui.view, "macro", slot.index).length }))} dirty={dirty} /><div class="slot-form">{#if macro}<article class="slot-edit-panel" aria-label={`Editing macro M${selected}`}><div class="slot-edit-title"><div><span class="slot-kicker">Macro {selected}</span>{#if ui.view?.names.macros[selected]}<h3>{ui.view.names.macros[selected]}</h3>{/if}</div><div class="slot-title-actions"><button class="quiet-button" onclick={() => window.dispatchEvent(new CustomEvent("canvas-rename-macro", { detail: selected }))}>Rename</button>{#if dirty}<span class="slot-draft">Draft</span>{/if}</div></div><SlotUses feature="macro" index={selected} /><p class="macro-usage">{dirty && projectedTotal === null ? (usagePending ? "Calculating bytes" : "— bytes total projected") :`${dirty ? projectedTotal : (ui.view?.macroBytes ?? 0)} bytes total${dirty ? " projected" : ""}`} <span aria-hidden="true">·</span> {ui.view?.macroCapacity === null ? "capacity unknown" : `${ui.view?.macroCapacity} byte capacity`}</p>{#if ui.view?.macroCapacity !== null && (dirty ? (projectedTotal ?? 0) : (ui.view?.macroBytes ?? 0)) > (ui.view?.macroCapacity ?? Infinity)}<p class="macro-alert" role="alert">Macro memory exceeded. No actions will be truncated.</p>{/if}<p class="macro-slot-usage">{usage ? `${usage.bytes} bytes in this slot${dirty ? " projected" : ""}` : usagePending ? "Calculating slot size" : "— bytes in this slot"}</p><div class="macro-actions">{#each macro.actions as action, i (ids[i] ?? `row-${i}`)}<div class="action"><div class="row macro-action-head"><button class="action-grip" disabled={dirty} title="Drag to reorder action" aria-label={`Drag action ${i + 1}`} style:touch-action="none" onpointerdown={(e) => startPointerDrag(e, { kind: "macro", slot: selected, index: i }, action.kind)}><Icon name="grip" size={14} /></button><span class="action-index">{i + 1}</span><span class="action-title"><strong>{action.kind}</strong><small>{usage?.actionBytes[i] ?? 0} bytes</small></span><span class="action-spacer"></span><button disabled={dirty || i === 0} title="Move action up" aria-label={`Move action ${i + 1} up`} onclick={() => reorder(i, i - 1)}>↑</button><button disabled={dirty || i === macro!.actions.length - 1} title="Move action down" aria-label={`Move action ${i + 1} down`} onclick={() => reorder(i, i + 1)}>↓</button><button disabled={dirty} onclick={() => { ui.inspectorPaths = []; void commit(macro!.actions.filter((_, j) => j !== i), ids.filter((_, j) => j !== i)); }}>Delete</button></div>{#if action.kind === "text"}<label>Text <input value={draft[i] ?? action.text} oninput={(e) => (draft[i] = e.currentTarget.value)} /></label>{:else if action.kind === "delay"}<label>Delay <input type="number" min="0" max="65024" value={draft[i] ?? action.delay} oninput={(e) => (draft[i] = e.currentTarget.value)} /> ms</label>{:else}<div class="macro-keys">{#each action.keys as _, k}<div class="macro-key"><Field path={`macro[${selected}][${i}][${k + 1}]`} label={`Key ${k + 1}`} size="sm" /><button class="macro-remove" disabled={dirty} aria-label={`Remove key ${k + 1}`} title="Remove key from action" onclick={() => void commit(macro!.actions.map((a, j) => j === i ? { ...a, keys: a.keys.filter((_, n) => n !== k) } : a))}>−</button></div>{/each}<button class="keycap-add" disabled={dirty} aria-label="Add key" title="Add key" onclick={() => void commit(macro!.actions.map((a, j) => (j === i ? { ...a, keys: [...a.keys, "KC_NO"] } : a)))}>+</button></div>{/if}</div>{/each}</div><div class="row macro-add-actions">{#each ["tap", "down", "up", "text", "delay"] as kind}<button disabled={dirty} onclick={() => add(kind as MacroActionKind)}>+ {kind}</button>{/each}</div>{#if dirty}<div class="row macro-draft-actions"><button onclick={() => void applyDraft()}>Apply draft</button><button onclick={() => { draft = {}; error = ""; }}>Revert draft</button></div>{/if}{#if error}<p class="macro-alert" role="alert">{error}</p>{/if}</article>{/if}</div></section>


