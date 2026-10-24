<script lang="ts" module>
  // Mirrors the board's kind assignment in crates/vile-session/src/session.rs
  // (kind_name and the Mouse/Media picker categories) so an editor keycap is
  // coloured like the same keycode on the board.
  export function keycapKind(code: number, category: string): string {
    if (category === "Mouse") return "mouse";
    if (category === "Media") return "media";
    if (code === 0) return "no";
    if (code === 1) return "trns";
    if (code < 0x0100) return code >= 0xe0 && code <= 0xe7 ? "modifier" : "basic";
    if (code < 0x2000) return "mods";
    if (code < 0x4000) return "modTap";
    if (code < 0x5000) return "layerTap";
    if (code < 0x5200) return "layerMod";
    if (code < 0x52a0 || (code >= 0x52c0 && code < 0x5300)) return "layer";
    if (code < 0x52c0) return "oneShotMod";
    if (code >= 0x5700 && code <= 0x57ff) return "tapDance";
    if (code >= 0x7700 && code <= 0x77ff) return "macro";
    if (code >= 0x7e00 && code <= 0x7e3f) return "user";
    return "other";
  }
</script>

<script lang="ts">
  let {
    kind = "no",
    label = "",
    size = "md",
    selected = false,
  }: { kind?: string; label?: string; size?: "md" | "sm"; selected?: boolean } = $props();
  const lines = $derived(label ? label.split("\n") : []);
  const hold = $derived(kind === "modTap" || kind === "layerTap");
  const shown = $derived(hold ? [...lines].reverse() : lines);
</script>

<svg class="keycap" class:sm={size === "sm"} viewBox="4 4 56 56" aria-hidden="true" focusable="false">
  <g class={`key kind-${kind}`} class:cursor={selected}>
    <rect x="4" y="4" width="56" height="56" rx="7" />
    {#each shown as line, li (li)}<text
        class:hold-line={hold && li === 0}
        x="32"
        y={37 + (li - (shown.length - 1) / 2) * 17}>{line}</text
      >{/each}
  </g>
</svg>

<style>
  /* Copied from the board's key rules in app/src/components/Keyboard.svelte so
     editor keycaps share its fills, strokes, labels and selection outline. */
  .keycap {
    display: block;
    width: 60px;
    height: 60px;
    overflow: visible;
  }
  .keycap.sm {
    width: 46px;
    height: 46px;
  }
  .key rect {
    fill: var(--key-bg);
    stroke: var(--key-border);
    stroke-width: 1.5;
  }
  .key text {
    fill: var(--key-fg);
    font-size: 16px;
    text-anchor: middle;
  }
  .key text.hold-line {
    font-size: 10px;
    fill: var(--muted);
  }
  .kind-trns rect {
    fill: var(--key-trns-bg);
  }
  .kind-layer rect,
  .kind-layerTap rect,
  .kind-layerMod rect {
    fill: var(--key-layer);
  }
  .kind-modifier rect,
  .kind-modTap rect,
  .kind-mods rect,
  .kind-oneShotMod rect {
    fill: var(--key-mod);
  }
  .kind-macro rect,
  .kind-tapDance rect {
    fill: var(--key-macro);
  }
  .kind-media rect,
  .kind-mouse rect {
    fill: var(--key-media);
  }
  .key.cursor rect {
    stroke: var(--cursor);
    stroke-width: 2;
  }
</style>
