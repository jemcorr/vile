<script lang="ts">
  import type { BoardKeyDto, KeyCellDto } from "../lib/types";
  import type { ComboOverlay } from "../lib/combos";

  interface Props {
    keys: BoardKeyDto[];
    cells: KeyCellDto[];
    cursor: string;
    selection: string[];
    onkeyclick: (id: string, shift: boolean) => void;
    overlays?: ComboOverlay[];
  }

  let { keys, cells, cursor, selection, onkeyclick, overlays = [] }: Props = $props();

  // One key unit in SVG user units.
  const U = 60;
  const GAP = 4;

  const width = $derived(Math.max(...keys.map((k) => k.x + k.w)) * U + GAP);
  const height = $derived(Math.max(...keys.map((k) => k.y + k.h)) * U + GAP);
</script>

<svg class="keyboard" viewBox={`0 0 ${width} ${height}`} role="img" aria-label="keyboard layer">
  {#each keys as k, i (k.id)}
    {@const cell = cells[i]}
    {@const lines = (cell?.label ?? "").split("\n")}
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <g
      data-key-id={k.id}
      class={`key kind-${cell?.kind ?? "no"}`}
      class:cursor={k.id === cursor}
      class:selected={selection.includes(k.id)}
      class:changed={cell?.changed ?? false}
      onclick={(e: MouseEvent) => onkeyclick(k.id, e.shiftKey)}
    >
      <title>{k.id}: {cell?.wire ?? ""}</title>
      <rect x={k.x * U + GAP} y={k.y * U + GAP} width={k.w * U - GAP} height={k.h * U - GAP} rx="7" />
      {#each lines as line, li (li)}
        <text
          x={(k.x + k.w / 2) * U + GAP / 2}
          y={(k.y + k.h / 2) * U + GAP / 2 + (li - (lines.length - 1) / 2) * 15 + 5}
        >{line}</text>
      {/each}
    </g>
  {/each}
  {#each overlays as overlay (overlay.index)}
    {#each overlay.keyIds as id (id)}
      {@const target = keys.find((k) => k.id === id)}
      {#if target}
        <line
          class="combo-line"
          x1={overlay.cx * U + GAP / 2}
          y1={overlay.cy * U + GAP / 2}
          x2={(target.x + target.w / 2) * U + GAP / 2}
          y2={(target.y + target.h / 2) * U + GAP / 2}
        />
      {/if}
    {/each}
    <g class="combo-badge">
      <rect x={overlay.cx * U + GAP / 2 - 22} y={overlay.cy * U + GAP / 2 - 11} width="44" height="22" rx="6" />
      <text x={overlay.cx * U + GAP / 2} y={overlay.cy * U + GAP / 2 + 4}>{overlay.label}</text>
    </g>
  {/each}
</svg>

<style>
  .keyboard {
    width: 100%;
    max-height: 70vh;
  }
  .key rect {
    fill: var(--key-bg);
    stroke: var(--key-border);
    stroke-width: 1.5;
  }
  .key text {
    fill: var(--key-fg);
    font-size: 13px;
    text-anchor: middle;
    pointer-events: none;
  }
  .kind-trns text,
  .kind-no text {
    fill: var(--muted);
  }
  .kind-layer rect,
  .kind-layerTap rect {
    fill: var(--key-layer);
  }
  .kind-modTap rect,
  .kind-mods rect,
  .kind-oneShotMod rect {
    fill: var(--key-mod);
  }
  .kind-macro rect,
  .kind-tapDance rect {
    fill: var(--key-macro);
  }
  .key.selected rect {
    fill: var(--selected);
  }
  .key.changed rect {
    stroke: var(--changed);
    stroke-width: 2.5;
    stroke-dasharray: 5 3;
  }
  .key.cursor rect {
    stroke: var(--cursor);
    stroke-width: 3;
    stroke-dasharray: none;
  }
  .combo-line {
    stroke: var(--cursor);
    stroke-width: 1;
    pointer-events: none;
  }
  .combo-badge rect {
    fill: var(--selected);
    stroke: var(--key-border);
    pointer-events: none;
  }
  .combo-badge text {
    fill: var(--key-fg);
    font-size: 11px;
    text-anchor: middle;
    pointer-events: none;
  }
</style>
