<script lang="ts">
  import type { KeycodeView, KeyPartsPatch, HoldBehavior } from "../lib/types";
  import { api } from "../lib/api";
  import { pointerDrop } from "../lib/pointer-drag.svelte";
  import Modifiers from "./Modifiers.svelte";
  let {
    keys,
    layerCount,
    onpatch,
    onbase,
    choosing = false,
  }: {
    keys: KeycodeView[];
    layerCount: number;
    onpatch: (patch: KeyPartsPatch) => void;
    onbase: () => void;
    choosing?: boolean;
  } = $props();
  function shared<T>(get: (key: KeycodeView) => T): T | null {
    const first = keys[0] ? get(keys[0]) : null;
    return keys.every((key) => JSON.stringify(get(key)) === JSON.stringify(first)) ? first : null;
  }
  const taps = $derived(keys.length > 0 && keys.every((key) => key.parts.kind === "tap"));
  const base = $derived(shared((key) => (key.parts.kind === "tap" ? key.parts.base : null)));
  const wrap = $derived(shared((key) => (key.parts.kind === "tap" ? key.parts.wrapMods : null)));
  const hold = $derived(shared((key) => (key.parts.kind === "tap" ? key.parts.hold : null)));
  const layer = $derived(shared((key) => ("layer" in key.parts ? key.parts.layer : null)));
  const mods = $derived(shared((key) => ("mods" in key.parts ? key.parts.mods : null)));
  const basic = $derived(keys.every((key) => key.parts.kind === "tap" && key.parts.base <= 255));
  const canWrap = $derived(
    taps &&
      basic &&
      keys.every((key) => key.parts.kind === "tap" && key.parts.hold.kind === "none"),
  );
  const canHold = $derived(
    taps && basic && keys.every((key) => key.parts.kind === "tap" && key.parts.wrapMods === 0),
  );
  let baseView: KeycodeView | null = $state(null);
  $effect(() => {
    const code = base;
    let current = true;
    baseView = null;
    if (code !== null)
      void api.keycodeView(`0x${code.toString(16)}`).then((view) => {
        if (current) baseView = view;
      });
    return () => {
      current = false;
    };
  });
  function setHold(kind: string): void {
    const value: HoldBehavior =
      kind === "mod"
        ? { kind: "mod", mods: 8 }
        : kind === "layer"
          ? { kind: "layer", layer: 0 }
          : { kind: "none" };
    onpatch({ hold: value });
  }
</script>

<section class="inspector" aria-label="Key inspector">
  <h2>{keys.length === 1 ? keys[0]!.description : `${keys.length} keys selected`}</h2>
  {#if keys.length === 1}<small class="raw">{keys[0]!.wire}</small>{/if}
  {#if taps}
    <div class="control">
      <span>Base</span><button
        class:active={choosing}
        onclick={onbase}
        title="Choose a base from the picker, or drop a tile here"
        use:pointerDrop={(event) => {
          if (event.payload.kind !== "keycode" || event.payload.code === null) return false;
          onpatch({ base: event.payload.code });
          return true;
        }}>{baseView?.label || (base === null ? "Mixed bases" : "Choose base")}</button
      ><small>{baseView?.description ?? "Each base is preserved"}</small>
    </div>
    <div class="control">
      <span>Wrap modifiers</span><Modifiers
        value={wrap}
        onchange={(wrapMods) => onpatch({ wrapMods })}
        disabled={!canWrap}
        reason={!basic
          ? "Modifiers need a basic base key"
          : !canWrap
            ? "Remove the hold behavior before adding wrap modifiers"
            : ""}
      />
    </div>
    <label
      >Hold <select value={hold?.kind ?? "mixed"} onchange={(e) => setHold(e.currentTarget.value)}
        ><option disabled value="mixed">Mixed</option><option value="none">None</option><option
          value="mod"
          disabled={!canHold}>Mod{!canHold ? " (needs basic base without wrap mods)" : ""}</option
        ><option value="layer" disabled={!canHold}
          >Layer{!canHold ? " (needs basic base without wrap mods)" : ""}</option
        ></select
      ></label
    >
    {#if hold?.kind === "mod"}<Modifiers
        value={hold.mods}
        onchange={(value) =>
          onpatch({ hold: value & 15 ? { kind: "mod", mods: value } : { kind: "none" } })}
      />{/if}
    {#if hold?.kind === "layer"}<div class="layer-buttons">
        {#each Array.from({ length: Math.min(layerCount, 16) }, (_, i) => i) as i}<button
            class:active={hold.layer === i}
            onclick={() => onpatch({ hold: { kind: "layer", layer: i } })}>L{i}</button
          >{/each}
      </div>{/if}
  {:else if keys.length && keys.every((key) => key.parts.kind === "oneShot" || key.parts.kind === "layerMod")}
    <Modifiers value={mods} onchange={(value) => onpatch({ mods: value })} />
  {:else if keys.some((key) => key.parts.kind === "raw")}
    <p>This numeric encoding is preserved. Choose a picker tile to replace the complete key.</p>
  {:else if keys.length > 1}<p>
      These kinds have no common base or hold fields. A picker tile replaces all selected keys.
    </p>{/if}
  {#if keys.length && keys.every((key) => key.parts.kind === "layer" || key.parts.kind === "layerMod")}
    <div class="layer-buttons">
      {#each Array.from({ length: Math.min(layerCount, keys.some((key) => key.parts.kind === "layerMod") ? 16 : 32) }, (_, i) => i) as i}<button
          class:active={layer === i}
          onclick={() => onpatch({ layer: i })}>L{i}</button
        >{/each}
    </div>
  {/if}
</section>

<style>
  .inspector {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--key-border);
    margin-bottom: 16px;
  }
  h2 {
    font-size: 16px;
    margin: 0;
  }
  .raw {
    font-family: monospace;
    color: var(--muted);
  }
  .control {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .control > span {
    font-size: 12px;
    color: var(--muted);
  }
  .control > button {
    align-self: flex-start;
    font-size: 20px;
    min-width: 70px;
  }
  .active {
    background: var(--selected);
  }
  .layer-buttons {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }
  small {
    color: var(--muted);
  }
</style>
