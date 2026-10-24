<script lang="ts">
  // Read-only layer 0 board for the Combos editor. It renders the production Keyboard
  // component, so geometry and keycap labels match the main board. The wrapper is
  // inert, which removes pointer and keyboard interaction.
  import Keyboard from "./Keyboard.svelte";
  import { ui } from "../lib/state.svelte";
  import { rotate } from "../lib/interactions";
  import type { BoardKeyDto } from "../lib/types";

  let { index }: { index: number } = $props();

  const view = $derived(ui.view);
  const rawKeys = $derived(view?.board?.keys ?? []);
  // The shared board bounds clamp the viewBox origin at 0, so the drawn keys sit off-centre
  // when a rotated thumb key lies beyond it. Shifting every key so its rotated minimum is at
  // the origin makes the viewBox tight, which gives equal margins on all four sides.
  const offset = $derived.by(() => {
    let minX = Infinity,
      minY = Infinity;
    for (const key of rawKeys) {
      const rects = [{ x: 0, y: 0, w: key.w, h: key.h }, ...(key.secondary ? [key.secondary] : [])];
      for (const rect of rects)
        for (const [dx, dy] of [[0, 0], [rect.w, 0], [0, rect.h], [rect.w, rect.h]] as [number, number][]) {
          const p = rotate(key, { x: key.x + rect.x + dx, y: key.y + rect.y + dy });
          minX = Math.min(minX, p.x);
          minY = Math.min(minY, p.y);
        }
    }
    return { x: Number.isFinite(minX) ? minX : 0, y: Number.isFinite(minY) ? minY : 0 };
  });
  const keys = $derived(
    rawKeys.map(
      (key): BoardKeyDto => ({
        ...key,
        x: key.x - offset.x,
        y: key.y - offset.y,
        rx: (key.rx ?? 0) - offset.x,
        ry: (key.ry ?? 0) - offset.y,
      }),
    ),
  );
  const base = $derived(view?.layers[0] ?? []);
  const combo = $derived(view?.combos.find((c) => c.index === index));
  const emptySlot = $derived(
    !combo || (combo.keys.every((k) => k === "KC_NO") && combo.output === "KC_NO"),
  );

  // A trigger matches a layer 0 key when the keycode values are equal. KC_NO is skipped.
  function codeAt(path: string): number | null {
    const key = view?.keycodes[path];
    return key && key.wire !== "KC_NO" && key.code !== 0 ? key.code : null;
  }
  const codes = $derived.by((): number[] => {
    if (emptySlot) return [];
    return Array.from({ length: 4 }, (_, i) => codeAt(`combo[${index}][${i}]`)).filter(
      (code): code is number => code !== null,
    );
  });

  const highlight = $derived(base.filter((cell) => codes.includes(cell.code)).map((cell) => cell.id));
  const layerName = $derived(view?.names.layers[0] || "Base");
  const note = $derived.by(() => {
    if (emptySlot) return "Empty slot";
    const count = highlight.length;
    return count === 0 ? "No matching key on layer 0" : `${count} matching key${count === 1 ? "" : "s"} on layer 0`;
  });
  const noop = (): void => {};
</script>

<div class="mini-board" role="group" aria-label="Layer 0 keyboard preview">
  <div class="mini-board-head"><span>Layer 0 · {layerName}</span><span>{note}</span></div>
  <div class="mini-board-keys" inert>
    <Keyboard
      {keys}
      cells={base}
      cursor=""
      selection={highlight}
      locked
      onkeyclick={noop}
      onmarquee={noop}
      onblankclick={noop}
      ondragkey={noop}
      onassign={noop}
      onedit={noop}
      onmenu={noop}
    />
  </div>
</div>

<style>
  /* The main board scales with the viewport (about 70vw wide), so 40vw keeps the mini
     board near 55 to 60 percent of its key size at 1440px and 1040px. */
  .mini-board {
    width: 100%;
    margin-top: var(--space-4);
    padding-top: var(--space-3);
    border-top: 1px solid var(--key-border);
    min-width: 0;
  }
  .mini-board-head {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    gap: var(--space-2);
    margin-bottom: var(--space-2);
    color: var(--muted);
    font-size: 12px;
  }
  .mini-board-keys {
    pointer-events: none;
    width: min(100%, 40vw);
    margin: 0 auto;
  }
</style>
