<script lang="ts">
  // Replaces app/src/components/Field.svelte in the preview only (presentation.mjs).
  // Its imports are written for this folder; presentation.mjs rewrites them to be
  // relative to app/src/components before Vite compiles the replacement.
  // Same state path, click handler and picker wiring as the production field.
  // Feature editors show a keycap; `plain` keeps the original wide row for the
  // encoder list in the Keymap panel.
  import { ui } from "../lib/state.svelte";
  import Keycap, { keycapKind } from "./Keycap.svelte";
  import { shortLabel } from "../lib/changes";
  let {
    path,
    label,
    size = "md",
    plain = false,
  }: { path: string; label: string; size?: "md" | "sm"; plain?: boolean } = $props();
  const key = $derived(ui.view?.keycodes[path]);
  const active = $derived(ui.inspectorPaths.length === 1 && ui.inspectorPaths[0] === path);
  const kind = $derived(key ? keycapKind(key.code, key.category) : "no");
  const empty = $derived(!key || key.wire === "KC_NO");
  const was = $derived(ui.view?.baseline?.keycodes[path]);
  const changed = $derived(!!was && !!key);
  function select(): void {
    ui.inspectorPaths = [path];
    ui.baseTarget = false;
  }
</script>

{#if plain}
  <button
    class="field"
    class:active
    aria-pressed={active}
    title={key ? `${key.tooltip}\n${key.wire}` : path}
    onclick={select}
  >
    <span>{label}</span>{#if changed}<i class="change-dot" aria-hidden="true"></i>{/if}<strong>{key?.label || key?.description || "No key"}</strong><small
      >{key?.description ?? ""}</small
    ><code>{key?.wire ?? ""}</code>
  </button>
{:else}
  <button
    class="field keycap-field"
    class:sm={size === "sm"}
    class:empty
    aria-pressed={active}
    aria-label={`${label}: ${key?.description || "No key"}${changed ? ", changed" : ""}`}
    title={key ? `${key.tooltip}\n${key.wire}${changed ? `\nWas ${shortLabel(was)}` : ""}` : path}
    data-kind={kind}
    onclick={select}
  >
    <Keycap {kind} label={key?.label ?? ""} {size} selected={active} {changed} />
    <span class="keycap-caption">{label}</span>
  </button>
{/if}

<style>
  .field {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    width: 100%;
    text-align: left;
  }
  .active {
    background: var(--selected);
  }
  .field > span {
    color: var(--muted);
    font-size: 12px;
    min-width: 55px;
  }
  .field > strong {
    font-size: 15px;
    font-weight: 500;
  }
  .field > small,
  .field > code {
    color: var(--muted);
    font-size: 11px;
  }
  .keycap-field {
    display: inline-flex;
    flex-direction: column;
    flex-wrap: nowrap;
    align-items: center;
    gap: var(--space-1);
    width: 88px;
    min-width: 0;
    padding: 0;
    border: 0;
    background: transparent;
    color: inherit;
    font: inherit;
    text-align: center;
  }
  .keycap-field.sm {
    width: 56px;
  }
  .keycap-field.empty :global(.keycap) {
    opacity: 0.4;
  }
  .keycap-caption {
    max-width: 100%;
    overflow: hidden;
    color: var(--muted);
    font-size: 12px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .keycap-field.sm .keycap-caption {
    font-size: 11px;
  }
</style>
