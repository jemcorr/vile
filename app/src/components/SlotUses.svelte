<script lang="ts">
  import type { ChangeJump } from "../lib/changes";
  import { NOUNS } from "../lib/slot-menu";
  import { PANEL_OF, usesOf } from "../lib/slots";
  import { ui } from "../lib/state.svelte";
  import type { SlotUseDto } from "../lib/types";

  let { feature, index }: { feature: "macro" | "tapDance"; index: number } = $props();

  /** Chips shown before the "+N" chip that stands for the rest. */
  const LIMIT = 8;
  const FIELD_LABEL: Record<string, string> = {
    trigger1: "Trigger 1",
    trigger2: "Trigger 2",
    trigger3: "Trigger 3",
    trigger4: "Trigger 4",
    output: "Output",
    tap: "Tap",
    hold: "Hold",
    doubleTap: "Double tap",
    tapHold: "Tap and hold",
    trigger: "Trigger",
    replacement: "Replacement",
    key: "Key",
    alternate: "Alternate",
  };
  const uses = $derived(usesOf(ui.view, feature, index));

  function fieldLabel(field: string): string {
    const action = /^action(\d+)$/.exec(field);
    return action ? `Action ${action[1]}` : (FIELD_LABEL[field] ?? field);
  }
  /** The board's name for a key, else on other layers its layer 0 label, else its id. */
  function keyName(layer: number, keyId: string): string {
    const key = ui.view?.board?.keys.find((candidate) => candidate.id === keyId);
    if (key?.name) return key.name;
    if (layer !== 0) {
      const base = ui.view?.layers[0]?.find((cell) => cell.id === keyId);
      if (base && !["no", "trns", "macro", "tapDance"].includes(base.kind)) return base.label;
    }
    return keyId;
  }
  function chipText(use: SlotUseDto): string {
    const place = use.place;
    if (place.kind === "key") return `Layer ${place.layer} · ${keyName(place.layer, place.keyId)}`;
    if (place.kind === "knob") {
      return `Layer ${place.layer} · knob ${place.knob + 1} ${place.clockwise ? "↻" : "↺"}`;
    }
    const noun = NOUNS[place.feature][0];
    return `${noun.charAt(0).toUpperCase()}${noun.slice(1)} ${place.index} · ${fieldLabel(place.field)}`;
  }
  function titleOf(use: SlotUseDto): string {
    const place = use.place;
    if (place.kind !== "key") return `${chipText(use)}. Show it`;
    const name = ui.view?.names.layers[String(place.layer)];
    return `Layer ${place.layer}${name ? ` (${name})` : ""}, key ${place.keyId}. Show it`;
  }
  function jumpOf(use: SlotUseDto): ChangeJump {
    const place = use.place;
    if (place.kind === "key") return { kind: "keys", layer: place.layer, keyIds: [place.keyId] };
    if (place.kind === "knob") return { kind: "layer", layer: place.layer };
    return { kind: "slot", panel: PANEL_OF[place.feature], index: place.index };
  }
  function show(use: SlotUseDto): void {
    ui.jump = jumpOf(use);
  }
</script>

<div class="slot-uses">
  {#if uses.length === 0}
    <span
      class="slot-uses-none"
      title="No key, knob, combo, tap dance, override or macro plays this slot"
    >Unused</span>
  {:else}
    <span class="slot-uses-label">Used by</span>
    {#each uses.slice(0, LIMIT) as use (use.path)}
      <button class="use-chip" title={titleOf(use)} onclick={() => show(use)}>{chipText(use)}</button>
    {/each}
    {#if uses.length > LIMIT}
      <button
        class="use-chip"
        title={`Show the next of the ${uses.length - LIMIT} more: ${chipText(uses[LIMIT]!)}`}
        onclick={() => show(uses[LIMIT]!)}>+{uses.length - LIMIT}</button
      >
    {/if}
  {/if}
</div>
