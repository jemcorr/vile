<script lang="ts">
  import { pointerDrag, pointerDrop } from "../lib/pointer-drag.svelte";
  import type { BoardKeyDto, KeyCellDto } from "../lib/types";
  import type { ComboOverlay } from "../lib/combos";
  import { bounds, hitKey, marqueeKeys, unrotate, type Point } from "../lib/interactions";
  import { matrixLabel, type GridPosition } from "../lib/matrix";
  import { isMacPlatform } from "../lib/keymap";
  interface Props {
    keys: BoardKeyDto[];
    cells: KeyCellDto[];
    cursor: string;
    selection: string[];
    onkeyclick: (id: string, toggle: boolean) => void;
    onmarquee: (ids: string[], additive: boolean) => void;
    onblankclick: () => void;
    ondragkey: (from: string, to: string, copy: boolean) => void;
    onassign: (id: string, wire: string) => void;
    onedit: (id: string) => void;
    onmenu: (id: string, x: number, y: number) => void;
    overlays?: ComboOverlay[];
    locked?: boolean;
    /** Matrix view: keys are unit cells, labelled row,col, and unoccupied cells are drawn as outlines. */
    matrix?: boolean;
    empties?: GridPosition[];
  }
  let {
    keys,
    cells,
    cursor,
    selection,
    onkeyclick,
    onmarquee,
    onblankclick,
    ondragkey,
    onassign,
    onedit,
    onmenu,
    overlays = [],
    locked = false,
    matrix = false,
    empties = [],
  }: Props = $props();
  const U = 60,
    GAP = 4;
  const box = $derived(bounds(keys));
  let svg: SVGSVGElement;
  $effect(() => {
    const current = cursor;
    if (!svg || !(document.activeElement instanceof SVGElement) || !svg.contains(document.activeElement)) return;
    if (!current) { document.activeElement.blur(); return; }
    const next = svg.querySelector<SVGElement>('[data-key-id="' + current + '"]');
    if (next && next !== document.activeElement) next.focus({preventScroll:true});
  });
  let gesture: {
    start: Point;
    now: Point;
    key: string | null;
    additive: boolean;
    moved: boolean;
    pointer: number;
  } | null = $state(null);
  let drop: string | null = $state(null);
  let suppress = false;
  /** Shift, or Cmd on macOS and Ctrl elsewhere, adds or removes one key from the selection. */
  function toggles(e: { shiftKey: boolean; metaKey: boolean; ctrlKey: boolean }): boolean {
    return e.shiftKey || (isMacPlatform(navigator.platform) ? e.metaKey : e.ctrlKey);
  }
  function point(e: Pick<MouseEvent, "clientX" | "clientY">): Point {
    const matrix = svg.getScreenCTM();
    if (!matrix) return { x: 0, y: 0 };
    const p = new DOMPoint(e.clientX, e.clientY).matrixTransform(matrix.inverse());
    return { x: p.x / U, y: p.y / U };
  }
  function paintedKey(p: Point): string | null {
    return (
      [...keys].reverse().find((key) => {
        const q = unrotate(key, p);
        return [
          { x: key.x, y: key.y, w: key.w, h: key.h },
          ...(key.secondary
            ? [{ ...key.secondary, x: key.x + key.secondary.x, y: key.y + key.secondary.y }]
            : []),
        ].some(
          (rect) =>
            q.x >= rect.x + GAP / U &&
            q.x <= rect.x + rect.w &&
            q.y >= rect.y + GAP / U &&
            q.y <= rect.y + rect.h,
        );
      })?.id ?? null
    );
  }
  $effect(() => {
    const current = pointerDrag.current;
    if (current?.payload.kind === "keycode" && svg && !locked) drop = hitKey(keys, point(current));
    else if (!gesture) drop = null;
  });
  function down(e: PointerEvent): void {
    if (e.button !== 0) return;
    const p = point(e);
    const key = paintedKey(p);
    if (locked && key) return;
    // A drag that starts on the board never starts a native text selection of key legends.
    e.preventDefault();
    gesture = {
      start: p,
      now: p,
      key,
      additive: toggles(e),
      moved: false,
      pointer: e.pointerId,
    };
    svg.setPointerCapture(e.pointerId);
  }
  function move(e: PointerEvent): void {
    if (!gesture || gesture.pointer !== e.pointerId) return;
    const p = point(e);
    gesture.now = p;
    if (Math.hypot(p.x - gesture.start.x, p.y - gesture.start.y) * U > 6) gesture.moved = true;
    drop = gesture.moved && gesture.key ? hitKey(keys, p) : null;
  }
  function up(e: PointerEvent): void {
    if (!gesture || gesture.pointer !== e.pointerId) return;
    const g = gesture;
    const end = point(e);
    g.now = end;
    g.moved ||= Math.hypot(end.x - g.start.x, end.y - g.start.y) * U > 6;
    gesture = null;
    drop = null;
    if (svg.hasPointerCapture(e.pointerId)) svg.releasePointerCapture(e.pointerId);
    suppress = true;
    if (!g.moved) {
      if (g.key) onkeyclick(g.key, g.additive);
      else onblankclick();
    } else if (locked) {
      // Combo trigger editing allows empty clicks, but never drags or marquee edits.
    } else if (g.key) {
      const target = hitKey(keys, end);
      if (target && target !== g.key) ondragkey(g.key, target, e.altKey);
    } else onmarquee(marqueeKeys(keys, g.start, g.now), g.additive);
    setTimeout(() => {
      suppress = false;
    }, 0);
  }
  function cancel(e?: PointerEvent): void {
    if (e && gesture?.pointer !== e.pointerId) return;
    if (gesture && svg.hasPointerCapture(gesture.pointer))
      svg.releasePointerCapture(gesture.pointer);
    gesture = null;
    drop = null;
    suppress = true;
    setTimeout(() => (suppress = false), 0);
  }
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === "Escape") cancel();
  }}
/>
<!-- svelte-ignore a11y_no_static_element_interactions -->
<svg
  bind:this={svg}
  class="keyboard"
  viewBox={`${box.x * U} ${box.y * U} ${box.w * U + GAP} ${box.h * U + GAP}`}
  preserveAspectRatio="xMidYMid meet"
  role="group"
  aria-label={matrix ? "Matrix layer" : "Keyboard layer"}
  onpointerdown={down}
  onpointermove={move}
  onpointerup={up}
  onpointercancel={cancel}
  ondblclick={(e) => {
    const target = paintedKey(point(e));
    if (target && !locked) onedit(target);
  }}
  onlostpointercapture={(e) => {
    if (gesture?.pointer === e.pointerId) cancel();
  }}
  use:pointerDrop={(event) => {
    if (event.payload.kind !== "keycode" || locked) return false;
    const target = hitKey(keys, point(event));
    if (target) onassign(target, event.payload.wire);
    return true;
  }}
>
  {#if matrix}
    {#each empties as position (matrixLabel(position))}
      <g class="empty-cell" aria-hidden="true">
        <rect x={position.col * U + GAP} y={position.row * U + GAP} width={U - GAP} height={U - GAP} rx="7" />
        <text x={position.col * U + GAP + 5} y={position.row * U + GAP + 12}>{matrixLabel(position)}</text>
      </g>
    {/each}
  {/if}
  {#each keys as k, i (k.id)}
    {@const cell = cells[i]}
    {@const lines = (cell?.label ?? "").split("\n")}
    {@const hold = cell?.kind === "modTap" || cell?.kind === "layerTap"}
    <g
      data-key-id={k.id}
      class={`key kind-${cell?.kind ?? "no"}`}
      class:cursor={k.id === cursor}
      class:selected={selection.includes(k.id)}
      class:changed={cell?.changed ?? false}
      class:drop={drop === k.id}
      transform={`rotate(${k.r ?? 0} ${(k.rx ?? 0) * U} ${(k.ry ?? 0) * U})`}
      role="button"
      tabindex={k.id === (cursor || keys[0]?.id) ? 0 : -1}
      aria-pressed={selection.includes(k.id)}
      aria-label={`${k.name ?? k.id}: ${cell?.description ?? cell?.wire ?? ""}`}
      onclick={(e) => {
        if (!suppress) onkeyclick(k.id, toggles(e));
      }}
      onkeydown={(e) => {
        if (e.key === " ") {
          e.preventDefault();
          e.stopPropagation();
          onkeyclick(k.id, e.shiftKey);
        }
        if (e.key === "Enter") {
          e.preventDefault();
          e.stopPropagation();
          onedit(k.id);
        }
        if (((e.key === "F10" && e.shiftKey) || e.key === "ContextMenu") && !locked) {
          e.preventDefault();
          const rect = e.currentTarget.getBoundingClientRect();
          onmenu(k.id, rect.left, rect.bottom);
        }
      }}
      oncontextmenu={(e) => {
        e.preventDefault();
        if (!locked) onmenu(k.id, e.clientX, e.clientY);
      }}
    >
      <title
        >{cell?.tooltip ?? ""} · {cell?.wire ?? ""}{cell?.changed
          ? `\n${cell.previous} → ${cell.wire}`
          : ""}</title
      >
      <rect
        x={k.x * U + GAP}
        y={k.y * U + GAP}
        width={k.w * U - GAP}
        height={k.h * U - GAP}
        rx="7"
      />
      {#if k.secondary}<rect
          x={(k.x + k.secondary.x) * U + GAP}
          y={(k.y + k.secondary.y) * U + GAP}
          width={k.secondary.w * U - GAP}
          height={k.secondary.h * U - GAP}
          rx="7"
        />{/if}
      {#each hold ? [...lines].reverse() : lines as line, li (li)}<text
          class:hold-line={hold && li === 0}
          x={(k.x + k.w / 2) * U + GAP / 2}
          y={(k.y + k.h / 2) * U + GAP / 2 + (li - (lines.length - 1) / 2) * 17 + 5}>{line}</text
        >{/each}
      {#if matrix}<text class="matrix-label" x={k.x * U + GAP + 5} y={k.y * U + GAP + 12}>{matrixLabel({ row: k.row, col: k.col })}</text>{/if}
      {#if cell?.changed}<circle cx={(k.x + k.w) * U - 7} cy={k.y * U + 10} r="3" />{/if}
    </g>
  {/each}
  {#each overlays as overlay (overlay.index)}
    {#each overlay.keyIds as id (id)}{@const target = keys.find(
        (k) => k.id === id,
      )}{#if target}<line
          class="combo-line"
          x1={overlay.cx * U}
          y1={overlay.cy * U}
          x2={(target.x + target.w / 2) * U}
          y2={(target.y + target.h / 2) * U}
        />{/if}{/each}
    <g class="combo-badge"
      ><rect x={overlay.cx * U - 22} y={overlay.cy * U - 11} width="44" height="22" rx="6" /><text
        x={overlay.cx * U}
        y={overlay.cy * U + 4}>{overlay.label}</text
      ></g
    >
  {/each}
  {#if gesture?.moved && !gesture.key}<rect
      class="marquee"
      x={Math.min(gesture.start.x, gesture.now.x) * U}
      y={Math.min(gesture.start.y, gesture.now.y) * U}
      width={Math.abs(gesture.start.x - gesture.now.x) * U}
      height={Math.abs(gesture.start.y - gesture.now.y) * U}
    />{/if}
</svg>

<style>
  .keyboard {
    width: 100%;
    height: 100%;
    touch-action: none;
    user-select: none;
    -webkit-user-select: none;
  }
  .key {
    --kfill: var(--key-bg);
    cursor: pointer;
  }
  .key rect {
    fill: var(--kfill);
    stroke: var(--key-border);
    stroke-width: 1.5;
  }
  .key text {
    fill: var(--key-fg);
    font-size: 16px;
    text-anchor: middle;
    pointer-events: none;
  }
  .key text.hold-line {
    font-size: 10px;
    fill: var(--muted);
  }
  .kind-trns {
    --kfill: var(--key-trns-bg);
  }
  .kind-no {
    --kfill: var(--key-no-bg);
  }
  .key .matrix-label,
  .empty-cell text {
    font-size: 9px;
    font-weight: 450;
    fill: var(--muted);
    text-anchor: start;
    pointer-events: none;
  }
  .empty-cell rect {
    fill: none;
    stroke: var(--subtle-edge);
    stroke-width: 1px;
    stroke-dasharray: 4 3;
  }
  .empty-cell {
    pointer-events: none;
  }
  .kind-layer,
  .kind-layerTap,
  .kind-layerMod {
    --kfill: var(--key-layer);
  }
  .kind-modifier,
  .kind-modTap,
  .kind-mods,
  .kind-oneShotMod {
    --kfill: var(--key-mod);
  }
  .kind-macro,
  .kind-tapDance {
    --kfill: var(--key-macro);
  }
  .kind-media,
  .kind-mouse {
    --kfill: var(--key-media);
  }
  /* A selected key keeps its kind colour, tinted toward the accent, so no kind matches the selection. */
  .key.selected rect {
    fill: color-mix(in srgb, var(--kfill), var(--cursor) 30%);
  }
  .key.drop rect {
    stroke: var(--cursor);
    stroke-width: 4;
    fill: color-mix(in srgb, var(--kfill), var(--cursor) 30%);
  }
  .key circle {
    fill: var(--changed);
    pointer-events: none;
  }
  .marquee {
    fill: var(--selected);
    fill-opacity: 0.25;
    stroke: var(--cursor);
    pointer-events: none;
  }
  .combo-line {
    stroke: var(--cursor);
    pointer-events: none;
  }
  .combo-badge {
    pointer-events: none;
  }
  .combo-badge rect {
    fill: var(--selected);
  }
  .combo-badge text {
    text-anchor: middle;
    fill: var(--fg);
    font-size: 11px;
  }
</style>
