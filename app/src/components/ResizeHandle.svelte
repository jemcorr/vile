<script lang="ts">
  let {
    orientation,
    label,
    value,
    min,
    max,
    initial,
    onresize,
  }: {
    orientation: "vertical" | "horizontal";
    label: string;
    value: number;
    min: number;
    max: number;
    initial: number;
    onresize: (value: number) => void;
  } = $props();
  let drag: { pointer: number; position: number; value: number } | null = $state(null);
  const clamp = (next: number) => Math.min(max, Math.max(min, Math.round(next)));
  function finish(node: HTMLElement, cancel = false) {
    if (!drag) return;
    const previous = drag;
    drag = null;
    if (cancel) onresize(clamp(previous.value));
    if (node.hasPointerCapture(previous.pointer)) node.releasePointerCapture(previous.pointer);
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions (The separator is keyboard adjustable with arrow keys.) -->
<div
  class="resize-handle"
  class:vertical={orientation === "vertical"}
  class:dragging={drag !== null}
  role="separator"
  tabindex="0"
  aria-label={label}
  aria-orientation={orientation}
  aria-valuenow={Math.round(value)}
  aria-valuemin={min}
  aria-valuemax={max}
  title={`${label}. Drag or use arrow keys. Double-click to reset.`}
  onpointerdown={(e) => {
    if (e.button !== 0) return;
    e.preventDefault();
    e.currentTarget.focus();
    drag = {
      pointer: e.pointerId,
      position: orientation === "vertical" ? e.clientX : e.clientY,
      value,
    };
    e.currentTarget.setPointerCapture(e.pointerId);
  }}
  onpointermove={(e) => {
    if (!drag || drag.pointer !== e.pointerId) return;
    const position = orientation === "vertical" ? e.clientX : e.clientY;
    onresize(clamp(drag.value + drag.position - position));
  }}
  onpointerup={(e) => {
    if (drag?.pointer === e.pointerId) finish(e.currentTarget);
  }}
  onpointercancel={(e) => finish(e.currentTarget, true)}
  onlostpointercapture={(e) => finish(e.currentTarget, true)}
  ondblclick={() => onresize(clamp(initial))}
  onkeydown={(e) => {
    if (e.key === "Escape" && drag) {
      e.stopPropagation();
      finish(e.currentTarget, true);
      return;
    }
    const increase = orientation === "vertical" ? "ArrowLeft" : "ArrowUp";
    const decrease = orientation === "vertical" ? "ArrowRight" : "ArrowDown";
    if (![increase, decrease, "Home", "End"].includes(e.key)) return;
    e.preventDefault();
    e.stopPropagation();
    onresize(
      clamp(
        e.key === "Home"
          ? min
          : e.key === "End"
            ? max
            : value + (e.key === increase ? 1 : -1) * (e.shiftKey ? 40 : 10),
      ),
    );
  }}
></div>

<style>
  .resize-handle {
    flex: 0 0 8px;
    height: 8px;
    position: relative;
    cursor: row-resize;
    touch-action: none;
    user-select: none;
  }
  .resize-handle.vertical {
    width: 8px;
    height: auto;
    cursor: col-resize;
  }
  .resize-handle::after {
    content: "";
    position: absolute;
    inset: 3px 0;
    background: var(--key-border);
  }
  .resize-handle.vertical::after {
    inset: 0 3px;
  }
  .resize-handle:hover::after,
  .resize-handle:focus-visible::after,
  .resize-handle.dragging::after {
    background: var(--cursor);
  }
</style>
