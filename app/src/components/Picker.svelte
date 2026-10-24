<script lang="ts">
  import { api, errorText } from "../lib/api";
  import type { KeycodeView, SearchHitDto, KeyParts, ViewModel } from "../lib/types";
  import { pointerDrag, startPointerDrag, suppressDragClick } from "../lib/pointer-drag.svelte";
  import Modifiers from "./Modifiers.svelte";
  let {
    view,
    onchoose,
    choosingBase = false,
    target,
  }: {
    view: ViewModel;
    onchoose: (key: KeycodeView) => void;
    choosingBase?: boolean;
    target: string;
  } = $props();
  const buildLabels: Record<string, string> = {
    MO: "Momentary layer (MO)",
    TO: "Switch layer (TO)",
    TG: "Toggle layer (TG)",
    TT: "Tap toggle layer (TT)",
    OSL: "One-shot layer (OSL)",
    DF: "Default layer (DF)",
    PDF: "Persistent default layer (PDF)",
    LT: "Layer when held, key when tapped (LT)",
    LM: "Layer with modifiers (LM)",
    MT: "Modifier when held, key when tapped (MT)",
    WRAP: "Key with modifiers (WRAP)",
    OSM: "One-shot modifiers (OSM)",
  };
  const buildSummaries: Record<string, string> = {
    MO: "While held",
    TO: "Switch layer",
    TG: "Toggle layer",
    TT: "Hold / tap-toggle",
    OSL: "Next keypress",
    DF: "Until reset",
    PDF: "Save default",
    LT: "Hold layer / tap key",
    LM: "Layer + modifiers",
    MT: "Hold mods / tap key",
    WRAP: "Key + modifiers",
    OSM: "Next keypress",
  };
  const categories = [
    "Basic",
    "Symbols",
    "Numbers and F-keys",
    "Modifiers",
    "Layers",
    "Mouse",
    "Media",
    "Macros",
    "Quantum",
    "Special",
  ];
  let category = $state("Basic");
  let query = $state("");
  let hits: SearchHitDto[] = $state([]);
  let error = $state("");
  let searching = $state(false);
  let buildKind = $state("MO");
  let layer = $state(0);
  let mods = $state(8);
  let tap = $state(4);
  let builderBase = $state(false);
  let preview: KeycodeView | null = $state(null);
  let previewError = $state("");
  $effect(() => {
    const search = query;
    const revision = view.revision;
    void revision;
    let current = true;
    searching = true;
    void api
      .searchKeycodes(search, 4096)
      .then((next) => {
        if (current) {
          hits = next;
          searching = false;
          error = "";
        }
      })
      .catch((e) => {
        if (current) {
          error = errorText(e);
          searching = false;
        }
      });
    return () => {
      current = false;
    };
  });
  const visible = $derived(
    hits.filter((hit) => hit.complete && (query.trim() || hit.category === category)),
  );
  // Search preserves the core's relevance order. Group only category browsing.
  const sections = $derived.by(() => {
    if (query.trim()) return [{ name: "", hits: visible }];
    const groups = new Map<string, SearchHitDto[]>();
    for (const hit of visible) {
      const name = hit.pickerGroup;
      const group = groups.get(name);
      if (group) group.push(hit);
      else groups.set(name, [hit]);
    }
    return [...groups].map(([name, hits]) => ({ name, hits }));
  });
  const building = $derived(category === "Layers" || category === "Modifiers");
  $effect(() => {
    const kind = buildKind;
    const n = layer;
    const bits = mods;
    const base = tap;
    let current = true;
    preview = null;
    let parts: KeyParts;
    if (kind === "LT")
      parts = { kind: "tap", base, wrapMods: 0, hold: { kind: "layer", layer: n } };
    else if (kind === "MT")
      parts = { kind: "tap", base, wrapMods: 0, hold: { kind: "mod", mods: bits } };
    else if (kind === "WRAP") parts = { kind: "tap", base, wrapMods: bits, hold: { kind: "none" } };
    else if (kind === "OSM") parts = { kind: "oneShot", mods: bits };
    else if (kind === "LM") parts = { kind: "layerMod", layer: n, mods: bits };
    else {
      const ops = {
        MO: "Mo",
        TO: "To",
        TG: "Tg",
        TT: "Tt",
        OSL: "Osl",
        DF: "Df",
        PDF: "Pdf",
      } as const;
      parts = { kind: "layer", op: ops[kind as keyof typeof ops] ?? "Mo", layer: n };
    }
    void api
      .composeKeycode(parts)
      .then((next) => {
        if (current) {
          preview = next;
          previewError = "";
        }
      })
      .catch((e) => {
        if (current) previewError = errorText(e);
      });
    return () => {
      current = false;
    };
  });
  async function choose(hit: SearchHitDto): Promise<void> {
    try {
      const next = await api.keycodeView(hit.name);
      if (builderBase) {
        tap = next.code;
        builderBase = false;
        category = ["MT", "WRAP", "OSM"].includes(buildKind) ? "Modifiers" : "Layers";
      } else onchoose(next);
    } catch (e) {
      error = errorText(e);
    }
  }
  function selectCategory(next: string): void {
    category = next;
    if (next === "Layers") buildKind = "MO";
    if (next === "Modifiers") buildKind = "MT";
  }
  function macroTooltip(hit: SearchHitDto): string {
    const index = Number(hit.name.slice(1));
    return hit.category === "Macros"
      ? `${hit.description}\n${hit.name}\n${view.names.macros[index] ?? ""}\n${view.macros[index]?.actions.map((a) => (a.kind === "text" ? a.text : a.kind === "delay" ? `${a.delay} ms` : `${a.kind} ${a.keys.join(" + ")}`)).join("; ") ?? ""}`
      : `${hit.description}\n${hit.name}`;
  }
</script>

<section class="picker" aria-label="Keycode picker">
  <div class="picker-head">
    <input
      type="search"
      aria-label="Search keycodes"
      placeholder="Search keys, names or aliases"
      bind:value={query}
    /><span role="status"
      >{builderBase
        ? "Choose a builder tap key"
        : choosingBase
          ? "Choose the inspector base key"
          : `Picker target: ${target}`}</span
    >
  </div>
  <nav aria-label="Keycode categories">
    {#each categories as name}<button
        class:active={category === name}
        aria-pressed={category === name}
        onclick={() => selectCategory(name)}>{name}</button
      >{/each}
  </nav>
  {#if building}
    <div class="builder">
      <label
        >Build <select bind:value={buildKind}
          >{#each category === "Layers" ? ["MO", "TO", "TG", "TT", "OSL", "DF", "PDF", "LT", "LM"] : ["MT", "WRAP", "OSM"] as kind}<option
              value={kind}>{buildLabels[kind]}</option
            >{/each}</select
        ></label
      >
      {#if ["MO", "TO", "TG", "TT", "OSL", "DF", "PDF", "LT", "LM"].includes(buildKind)}<div
          class="layer-buttons"
        >
          {#each Array.from({ length: view.layerCount }, (_, i) => i) as i}<button
              disabled={i > (["LT", "LM"].includes(buildKind) ? 15 : 31)}
              class:active={layer === i}
              onclick={() => (layer = i)}
              >{view.names.layers[i] ? `${view.names.layers[i]} (L${i})` : `Layer ${i}`}</button
            >{/each}
        </div>{/if}
      {#if ["MT", "WRAP", "OSM", "LM"].includes(buildKind)}<Modifiers
          value={mods}
          onchange={(value) => (mods = value)}
        />{/if}
      {#if ["MT", "WRAP", "LT"].includes(buildKind)}<button
          class:active={builderBase}
          onclick={() => {
            builderBase = !builderBase;
            category = "Basic";
          }}>Choose tap key</button
        >{/if}
      {#if preview}<button
          class="tile preview"
          title={`${preview.tooltip}\n${preview.wire}`}
          onpointerdown={(e) =>
            startPointerDrag(
              e,
              { kind: "keycode", wire: preview!.wire, code: preview!.code },
              preview!.label,
            )}
          onclick={() => {
            if (!suppressDragClick()) onchoose(preview!);
          }}
          ><span>{preview.label.replaceAll("\n", " ")}</span><small
            >{buildSummaries[buildKind]}</small
          ></button
        >{/if}
      {#if previewError}<span role="alert">{previewError}</span>{/if}
      <span class="build-target"
        >{buildLabels[buildKind]}{["MO", "TO", "TG", "TT", "OSL", "DF", "PDF", "LT", "LM"].includes(
          buildKind,
        )
          ? `, target ${view.names.layers[layer] || `layer ${layer}`}`
          : ""}</span
      >
    </div>
  {/if}
  {#if error}<p role="alert">{error}</p>{/if}
  <p class="search-count" role="status">
    {searching
      ? "Searching…"
      : `${visible.length} results${query.trim() ? ` for “${query.trim()}”` : ` in ${category}`}`}
  </p>
  {#if !searching && !error && visible.length === 0}<p>
      No matching keycodes. Try a name, alias, or wire code.
    </p>{/if}
  <div class="results">
    {#each sections as section (section.name)}
      <section class="key-group" aria-label={section.name || "Search results"}>
        {#if section.name}<h3>{section.name}</h3>{/if}
        <div class="tiles">
          {#each section.hits as hit (hit.name)}<button
              class="tile"
              class:word-label={hit.pickerLabel.length > 5}
              title={macroTooltip(hit)}
              aria-label={`${hit.pickerLabel}. ${hit.description} (${hit.name})`}
              onpointerdown={(e) =>
                startPointerDrag(
                  e,
                  { kind: "keycode", wire: hit.name, code: hit.code },
                  hit.pickerLabel,
                )}
              onclick={() => {
                if (!suppressDragClick()) void choose(hit);
              }}><span>{hit.pickerLabel}</span><small>{hit.summary}</small></button
            >{/each}
        </div>
      </section>
    {/each}
  </div>
  {#if pointerDrag.current}<div
      class="drag-preview"
      style:left={`${pointerDrag.current.clientX + 12}px`}
      style:top={`${pointerDrag.current.clientY + 12}px`}
    >
      {pointerDrag.current.label.replaceAll("\n", " ")}
    </div>{/if}
</section>

<style>
  .drag-preview {
    position: fixed;
    z-index: 1000;
    pointer-events: none;
    padding: 8px 12px;
    border: 1px solid var(--selected);
    border-radius: 6px;
    background: var(--bg);
  }
  .tile {
    touch-action: none;
  }
  .picker {
    border-top: 1px solid var(--key-border);
    padding: 10px 16px;
    background: var(--bg);
    height: 100%;
    min-height: 0;
    overflow: auto;
    scrollbar-gutter: stable;
  }
  .picker-head {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 8px;
  }
  input {
    width: min(280px, 100%);
    min-width: 0;
    flex: 1 1 180px;
  }
  .picker-head > span {
    color: var(--muted);
    font-size: 12px;
  }
  .picker-head {
    flex-wrap: wrap;
  }
  .search-count,
  .build-target {
    color: var(--muted);
    font-size: 12px;
    margin: 6px 0;
  }
  nav,
  .builder,
  .layer-buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
    align-items: center;
  }
  nav {
    margin-bottom: 10px;
  }
  .active {
    background: var(--selected);
  }
  .results {
    display: grid;
    gap: 18px;
    padding-bottom: 6px;
  }
  .key-group {
    min-width: 0;
  }
  .key-group h3 {
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 0 0 8px;
    color: var(--muted);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.03em;
  }
  .key-group h3::after {
    content: "";
    flex: 1;
    border-top: 1px solid var(--key-border);
    opacity: 0.6;
  }
  .tiles {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(min(128px, 100%), 1fr));
    gap: 7px;
  }
  .tile {
    min-width: 0;
    min-height: 82px;
    padding: 9px 8px;
    display: grid;
    grid-template-rows: minmax(32px, auto) minmax(24px, auto);
    align-content: center;
    justify-items: center;
    gap: 5px;
    border-bottom-width: 3px;
    color: var(--key-fg);
  }
  .tile > span {
    align-self: center;
    max-width: 100%;
    font-size: 19px;
    line-height: 1.15;
    font-weight: 500;
    text-align: center;
    white-space: pre-line;
    overflow-wrap: anywhere;
  }
  .tile.word-label > span {
    font-size: 13px;
  }
  .tile small {
    max-width: 100%;
    color: var(--muted);
    font-size: 10px;
    line-height: 1.2;
    text-align: center;
    overflow-wrap: anywhere;
  }
  .preview {
    width: 160px;
  }
  .preview > span {
    font-size: 14px;
  }
  .preview small {
    font-size: 10px;
  }
  .builder {
    padding-bottom: 10px;
  }
  [role="alert"] {
    color: var(--changed);
  }
</style>
