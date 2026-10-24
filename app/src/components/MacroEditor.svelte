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
</script>

<section>
  <h2>Macros</h2>
  <p>
    {dirty && projectedTotal === null
      ? "Calculating projected bytes"
      : `${dirty ? projectedTotal : (ui.view?.macroBytes ?? 0)} bytes total${dirty ? " projected from draft" : ""}`}
    · {ui.view?.macroCapacity === null
      ? "capacity unknown"
      : `${ui.view?.macroCapacity} bytes capacity`}
  </p>
  {#if ui.view?.macroCapacity !== null && (dirty ? (projectedTotal ?? 0) : (ui.view?.macroBytes ?? 0)) > (ui.view?.macroCapacity ?? Infinity)}<p
      role="alert"
    >
      Macro memory exceeded. No actions will be truncated.
    </p>{/if}
  <select
    aria-label="Macro slot"
    value={selected}
    onchange={(e) => select(Number(e.currentTarget.value))}
    >{#each macros as m}<option value={m.index}
        >M{m.index} {ui.view?.names.macros[m.index] ?? ""} · {macroSummary(m.actions)}</option
      >{/each}</select
  >
  {#if macro}<p>
      {usage
        ? `${usage.bytes} bytes in this slot${dirty ? " projected" : ""}, including its terminator`
        : "Calculating bytes in this slot"}
    </p>
    {#each macro.actions as action, i (ids[i] ?? `row-${i}`)}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="action"
        use:pointerDrop={(event) => {
          if (dirty || event.payload.kind !== "macro" || event.payload.slot !== selected)
            return false;
          reorder(event.payload.index, i);
          return true;
        }}
      >
        <div class="row">
          <button
            disabled={dirty}
            title="Drag to reorder action"
            style:touch-action="none"
            onpointerdown={(e) =>
              startPointerDrag(e, { kind: "macro", slot: selected, index: i }, action.kind)}
            >↕</button
          >
          <strong>{action.kind}</strong><small>{usage?.actionBytes[i] ?? 0} bytes</small><button
            disabled={dirty || i === 0}
            title="Move action up"
            onclick={() => reorder(i, i - 1)}>↑</button
          ><button
            disabled={dirty || i === macro!.actions.length - 1}
            title="Move action down"
            onclick={() => reorder(i, i + 1)}>↓</button
          ><button
            disabled={dirty}
            onclick={() => {
              ui.inspectorPaths = [];
              void commit(
                macro!.actions.filter((_, j) => j !== i),
                ids.filter((_, j) => j !== i),
              );
            }}>Delete</button
          >
        </div>
        {#if action.kind === "text"}<label
            >Text <input
              value={draft[i] ?? action.text}
              oninput={(e) => (draft[i] = e.currentTarget.value)}
            /></label
          >
        {:else if action.kind === "delay"}<label
            >Delay <input
              type="number"
              min="0"
              max="65024"
              value={draft[i] ?? action.delay}
              oninput={(e) => (draft[i] = e.currentTarget.value)}
            /> ms</label
          >
        {:else}{#each action.keys as _, k}<div class="row">
              <Field path={`macro[${selected}][${i}][${k + 1}]`} label={`Key ${k + 1}`} /><button
                disabled={dirty}
                title="Remove key from action"
                onclick={() =>
                  void commit(
                    macro!.actions.map((a, j) =>
                      j === i ? { ...a, keys: a.keys.filter((_, n) => n !== k) } : a,
                    ),
                  )}>−</button
              >
            </div>{/each}<button
            disabled={dirty}
            onclick={() =>
              void commit(
                macro!.actions.map((a, j) => (j === i ? { ...a, keys: [...a.keys, "KC_NO"] } : a)),
              )}>Add key</button
          >{/if}
      </div>{/each}
    <div class="row">
      {#each ["tap", "down", "up", "text", "delay"] as kind}<button
          disabled={dirty}
          onclick={() => add(kind as MacroActionKind)}>+ {kind}</button
        >{/each}
    </div>
    {#if dirty}<div class="row">
        <button onclick={() => void applyDraft()}>Apply draft</button><button
          onclick={() => {
            draft = {};
            error = "";
          }}>Revert draft</button
        >
      </div>{/if}
    {#if error}<p role="alert">{error}</p>{/if}{/if}
</section>

<style>
  h2 {
    font-size: 15px;
  }
  p,
  small {
    font-size: 12px;
    color: var(--muted);
  }
  select {
    width: 100%;
  }
  .action {
    padding: 10px 0;
    border-bottom: 1px solid var(--key-border);
  }
  .row {
    display: flex;
    align-items: center;
    gap: 5px;
    flex-wrap: wrap;
    margin: 5px 0;
  }
  .row :global(.field) {
    flex: 1;
  }
  input {
    max-width: 220px;
  }
  [role="alert"] {
    color: var(--changed);
  }
</style>
