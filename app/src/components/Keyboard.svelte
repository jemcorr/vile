<script lang="ts">
  import { pointerDrag, pointerDrop } from "../lib/pointer-drag.svelte";
  import type { BoardKeyDto, KeyCellDto } from "../lib/types";
  import type { ComboOverlay } from "../lib/combos";
  import { bounds, hitKey, marqueeKeys, rotate, unrotate, type Point } from "../lib/interactions";
  import { matrixLabel, type GridPosition } from "../lib/matrix";
  import { isMacPlatform } from "../lib/keymap";
  import {
    clampCenter,
    clampScale,
    fitScale,
    viewBoxFor,
    zoomAt,
    type FitCap,
    type Size,
    type ZoomSetting,
  } from "../lib/zoom";
  import { untrack } from "svelte";
  import { nearestKey } from "../lib/range";
  import { revealShift, type Rect as ScreenRect } from "../lib/reveal";
  interface Props {
    keys: BoardKeyDto[];
    cells: KeyCellDto[];
    cursor: string;
    selection: string[];
    onkeyclick: (id: string, toggle: boolean) => void;
    /** `end` is the candidate key nearest the release point (the new cursor and anchor). */
    onmarquee: (ids: string[], additive: boolean, end: string | null) => void;
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
    /** Fill the viewport and scale the board to fit, within these CSS caps. `null` keeps the static viewBox. */
    fit?: FitCap | null;
    /** Zoom with `fit`: "fit", or an absolute scale (1 = 100%). */
    zoom?: ZoomSetting;
    onzoom?: (z: ZoomSetting) => void;
    /** Effective scale in CSS px per 1u key (60 SVG units), for the zoom controls. */
    scale?: number;
    /** Client rectangles that cover the board (card, pill, picker bar, zoom controls). Read when revealing. */
    occluders?: () => ScreenRect[];
    /** Bumped by the parent when arrows move the cursor, so the cursor key is revealed when zoomed. */
    revealTick?: number;
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
    fit = null,
    zoom = "fit",
    onzoom,
    scale: shownScale = $bindable(1),
    occluders = () => [],
    revealTick = 0,
  }: Props = $props();
  const U = 60,
    GAP = 4;
  const box = $derived(bounds(keys));
  const content = $derived({ x: box.x * U, y: box.y * U, w: box.w * U + GAP, h: box.h * U + GAP });
  let size: Size = $state({ w: 0, h: 0 });
  const fitS = $derived(fit ? fitScale(content, size, fit) : 0);
  // Scale in CSS px per SVG unit: the fit scale, or the chosen zoom clamped to the range.
  const s = $derived(!fit || zoom === "fit" ? fitS : clampScale(zoom));
  const live = $derived(s > 0 && size.w > 0 && size.h > 0);
  const contentCentre = $derived({ x: content.x + content.w / 2, y: content.y + content.h / 2 });
  // Pan centre in SVG units. null means the content centre; reset whenever the content or the zoom mode changes.
  let center: { x: number; y: number } | null = $state(null);
  const c = $derived(clampCenter(center ?? contentCentre, content, size, s));
  const vb = $derived(live ? viewBoxFor(c, size, s) : content);
  const zoomable = $derived(fit !== null);
  let pan: { pointer: number; x: number; y: number; from: { x: number; y: number }; s: number } | null =
    null;
  let gestureFrom = 1;
  $effect(() => {
    shownScale = s || 1;
  });
  $effect(() => {
    void content.x;
    void content.y;
    void content.w;
    void content.h;
    center = null;
  });
  $effect(() => {
    if (zoom === "fit") center = null;
  });
  let svg: SVGSVGElement;
  $effect(() => {
    if (!zoomable || !svg || typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver(([entry]) => {
      if (entry) size = { w: entry.contentRect.width, h: entry.contentRect.height };
    });
    observer.observe(svg);
    return () => observer.disconnect();
  });
  // Arrow moves reveal the cursor key while zoomed. Pointer selection never pans the view.
  let shownTick = 0;
  $effect(() => {
    const tick = revealTick;
    untrack(() => {
      if (tick !== shownTick) {
        shownTick = tick;
        revealCursor(cursor);
      }
    });
  });
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
  /** The rounded keycap rect (rx 7px), inset by the gap on its leading edges. Points in a corner's curve are blank. */
  const KEY_RADIUS = 7 / U;
  function inKeycap(q: Point, rect: { x: number; y: number; w: number; h: number }): boolean {
    const x0 = rect.x + GAP / U;
    const y0 = rect.y + GAP / U;
    const x1 = rect.x + rect.w;
    const y1 = rect.y + rect.h;
    if (q.x < x0 || q.x > x1 || q.y < y0 || q.y > y1) return false;
    const r = Math.min(KEY_RADIUS, (x1 - x0) / 2, (y1 - y0) / 2);
    const cx = Math.min(Math.max(q.x, x0 + r), x1 - r);
    const cy = Math.min(Math.max(q.y, y0 + r), y1 - r);
    return Math.hypot(q.x - cx, q.y - cy) <= r;
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
        ].some((rect) => inKeycap(q, rect));
      })?.id ?? null
    );
  }
  $effect(() => {
    const current = pointerDrag.current;
    if (current?.payload.kind === "keycode" && svg && !locked) drop = hitKey(keys, point(current));
    else if (!gesture) drop = null;
  });
  /** SVG units (the space of `content` and `center`), not key units. */
  function svgPoint(e: Pick<MouseEvent, "clientX" | "clientY">): Point {
    const p = point(e);
    return { x: p.x * U, y: p.y * U };
  }
  // Zoom set by the pointer (wheel, pinch) keeps the point under the pointer. Only other zooms
  // (buttons, shortcuts, palette) bring the cursor key into view.
  let pointerZoomTo: ZoomSetting | null = null;
  let shownZoom: ZoomSetting | undefined;
  $effect(() => {
    const z = zoom;
    untrack(() => {
      if (shownZoom !== undefined && z !== shownZoom && z !== pointerZoomTo) revealCursor(cursor);
      shownZoom = z;
      pointerZoomTo = null;
    });
  });
  /** The key's painted outline (rotated, with its secondary part) as a box in SVG units. */
  function keyBox(key: BoardKeyDto): { l: number; t: number; r: number; b: number } {
    const parts = [
      { x: key.x, y: key.y, w: key.w, h: key.h },
      ...(key.secondary
        ? [{ x: key.x + key.secondary.x, y: key.y + key.secondary.y, w: key.secondary.w, h: key.secondary.h }]
        : []),
    ];
    const pts = parts.flatMap((r) =>
      [
        [r.x, r.y],
        [r.x + r.w, r.y],
        [r.x, r.y + r.h],
        [r.x + r.w, r.y + r.h],
      ].map(([x, y]) => rotate(key, { x: x!, y: y! })),
    );
    return {
      l: Math.min(...pts.map((p) => p.x)) * U,
      t: Math.min(...pts.map((p) => p.y)) * U,
      r: Math.max(...pts.map((p) => p.x)) * U,
      b: Math.max(...pts.map((p) => p.y)) * U,
    };
  }
  /**
   * Brings the cursor key fully into view, 16px clear of the board's edges and of every occluder, when
   * zoomed past fit. A key that cannot fit is centred. Keys are never moved under an occluder.
   */
  function revealCursor(id: string): void {
    if (!fit || zoom === "fit" || !live || !id || !svg) return;
    const key = keys.find((k) => k.id === id);
    if (!key) return;
    const area = svg.getBoundingClientRect();
    // An SVG unit p sits at area.centre + (p - c) * s in client pixels, because the view is centred on c.
    const toClient = (x: number, y: number) => ({
      x: area.left + area.width / 2 + (x - c.x) * s,
      y: area.top + area.height / 2 + (y - c.y) * s,
    });
    const box = keyBox(key);
    const a = toClient(box.l, box.t);
    const b = toClient(box.r, box.b);
    const { dx, dy } = revealShift(
      { l: a.x, t: a.y, r: b.x, b: b.y },
      { l: area.left, t: area.top, r: area.right, b: area.bottom },
      occluders(),
    );
    if (dx === 0 && dy === 0) return;
    center = clampCenter({ x: c.x - dx / s, y: c.y - dy / s }, content, size, s);
  }
  $effect(() => {
    const el = svg;
    if (!fit || !el) return;
    const onWheel = (e: WheelEvent): void => {
      if (e.ctrlKey || e.metaKey) {
        // Cmd/Ctrl + wheel and trackpad pinch (Chromium) zoom around the pointer.
        e.preventDefault();
        const d = Math.max(-50, Math.min(50, e.deltaY));
        const s1 = clampScale(s * Math.exp(-d * 0.01));
        center = zoomAt(c, svgPoint(e), s, s1);
        pointerZoomTo = s1;
        onzoom?.(s1);
      } else if (s > fitS + 1e-6) {
        // Plain wheel and two-finger scroll pan only when the board is larger than the viewport.
        e.preventDefault();
        center = clampCenter({ x: c.x + e.deltaX / s, y: c.y + e.deltaY / s }, content, size, s);
      }
    };
    // WebKit reports trackpad pinch as gesture events; Chromium does not fire them.
    type GestureLike = Event & { scale: number; clientX: number; clientY: number };
    const onGestureStart = (e: Event): void => {
      e.preventDefault();
      gestureFrom = s;
    };
    const onGestureChange = (e: Event): void => {
      const g = e as GestureLike;
      g.preventDefault();
      const s1 = clampScale(gestureFrom * g.scale);
      center = zoomAt(c, svgPoint(g), s, s1);
      pointerZoomTo = s1;
      onzoom?.(s1);
    };
    const onGestureEnd = (e: Event): void => e.preventDefault();
    el.addEventListener("wheel", onWheel, { passive: false });
    el.addEventListener("gesturestart", onGestureStart);
    el.addEventListener("gesturechange", onGestureChange);
    el.addEventListener("gestureend", onGestureEnd);
    return () => {
      el.removeEventListener("wheel", onWheel);
      el.removeEventListener("gesturestart", onGestureStart);
      el.removeEventListener("gesturechange", onGestureChange);
      el.removeEventListener("gestureend", onGestureEnd);
    };
  });
  function down(e: PointerEvent): void {
    if (e.button === 1 && fit) {
      // Middle-button drag pans.
      e.preventDefault();
      pan = { pointer: e.pointerId, x: e.clientX, y: e.clientY, from: c, s };
      svg.setPointerCapture(e.pointerId);
      return;
    }
    if (e.button !== 0) return;
    const p = point(e);
    const key = paintedKey(p);
    if (locked && key) return;
    // A drag that starts on the board never starts a native text selection of key legends.
    e.preventDefault();
    // preventDefault stops the browser from moving focus, so a toolbar button clicked before would keep
    // it and swallow the arrow keys. Release it so the board receives them.
    if (document.activeElement instanceof HTMLElement && !svg.contains(document.activeElement))
      document.activeElement.blur();
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
    if (pan && pan.pointer === e.pointerId) {
      center = clampCenter(
        {
          x: pan.from.x - (e.clientX - pan.x) / pan.s,
          y: pan.from.y - (e.clientY - pan.y) / pan.s,
        },
        content,
        size,
        pan.s,
      );
      return;
    }
    if (!gesture || gesture.pointer !== e.pointerId) return;
    const p = point(e);
    gesture.now = p;
    if (Math.hypot(p.x - gesture.start.x, p.y - gesture.start.y) * U > 6) gesture.moved = true;
    drop = gesture.moved && gesture.key ? hitKey(keys, p) : null;
  }
  function up(e: PointerEvent): void {
    if (pan && pan.pointer === e.pointerId) {
      if (svg.hasPointerCapture(e.pointerId)) svg.releasePointerCapture(e.pointerId);
      pan = null;
      return;
    }
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
    } else {
      const ids = marqueeKeys(keys, g.start, g.now);
      onmarquee(ids, g.additive, nearestKey(keys, g.now, ids));
    }
    setTimeout(() => {
      suppress = false;
    }, 0);
  }
  function cancel(e?: PointerEvent): void {
    if (pan && (!e || pan.pointer === e.pointerId)) {
      if (svg.hasPointerCapture(pan.pointer)) svg.releasePointerCapture(pan.pointer);
      pan = null;
      return;
    }
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
  class:zoomable
  viewBox={`${vb.x} ${vb.y} ${vb.w} ${vb.h}`}
  preserveAspectRatio="xMidYMid meet"
  role="group"
  aria-label={matrix ? "Matrix layer" : "Keyboard layer"}
  onpointerdown={down}
  onpointermove={move}
  onpointerup={up}
  onpointercancel={cancel}
  onauxclick={(e) => {
    if (e.button === 1) e.preventDefault();
  }}
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
