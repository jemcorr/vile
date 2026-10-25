<script lang="ts">
  let { name, size = 16 }: { name: string; size?: number } = $props();
  const paths: Record<string, string> = {
    undo: "M8 4 3 9l5 5M3 9h11a6 6 0 0 1 0 12",
    redo: "m16 4 5 5-5 5M21 9H10a6 6 0 0 0 0 12",
    keyboard:
      "M4 5h16a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2M6 9h.01M10 9h.01M14 9h.01M18 9h.01M6 13h.01M10 13h.01M14 13h.01M18 13h.01M7 16h10",
    chevron: "m8 10 4 4 4-4",
    close: "m6 6 12 12M6 18 18 6",
    plus: "M12 5v14M5 12h14",
    minus: "M5 12h14",
    search: "M21 21 16 16M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0",
    sliders: "M4 7h16M4 17h16M9 4v6M15 14v6",
    sun: "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8zM12 2v2M12 20v2M2 12h2M20 12h2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4",
    moon: "M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5z",
    file: "M14 2H5a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9zM14 2v7h7",
    command:
      "M8 9H6a3 3 0 1 1 3-3v12a3 3 0 1 1-3-3h12a3 3 0 1 1-3 3V6a3 3 0 1 1 3 3H8",
  };
  // Six-dot grip (2 columns by 3 rows). Drawn as filled circles, not a path.
  const gripDots = [6, 12, 18].flatMap((cy) => [9, 15].map((cx) => ({ cx, cy })));
</script>

{#if name === "half"}
<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.5"
  aria-hidden="true"><circle cx="12" cy="12" r="8" /><path d="M12 4a8 8 0 0 1 0 16z" fill="currentColor" stroke="none" /></svg>
{:else if name === "grip"}
<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="currentColor"
  stroke="none"
  aria-hidden="true">{#each gripDots as dot}<circle cx={dot.cx} cy={dot.cy} r="1.6" />{/each}</svg>
{:else}
<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.5"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"><path d={paths[name] || paths.sliders} /></svg
>
{/if}
