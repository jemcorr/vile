<script lang="ts">
  import { focusScope } from "../lib/focus";
  import { primaryModifierLabel } from "../lib/keymap";
  import type { ChangeGroup, ChangeJump } from "../lib/changes";
  import type { DiffBase, ViewModel } from "../lib/types";

  interface Props {
    view: ViewModel;
    groups: ChangeGroup[];
    total: number;
    review: { revision: number; saveAs: boolean; error: string; focusSave: boolean };
    busy: boolean;
    drafts: boolean;
    oncompare: (base: DiffBase) => void;
    onjump: (target: ChangeJump) => void;
    onsave: () => void;
    onsaveas: () => void;
    oncancel: () => void;
  }
  let { view, groups, total, review, busy, drafts, oncompare, onjump, onsave, onsaveas, oncancel }: Props = $props();

  const CHIP_LIMIT = 18;
  const ROW_LIMIT = 8;
  let expanded: Record<string, boolean> = $state({});

  const file = $derived(view.path?.split(/[\\/]/).pop() || "this layout");
  const git = $derived(view.diffBase === "git");
  const stale = $derived(review.revision < 0 || review.revision !== view.revision);
  const saveAs = $derived(review.saveAs || !view.path);
  const primary = $derived(
    review.revision < 0
      ? "Refresh review"
      : stale
        ? "Review current changes"
        : saveAs
          ? "Save as…"
          : "Save",
  );
  const disabledReason = $derived(
    busy
      ? "An operation is in progress"
      : drafts && !stale
        ? "Apply or revert the macro draft first"
        : !stale && !saveAs && !view.dirty
          ? "No unsaved changes"
          : "",
  );
  const consequence = $derived(
    drafts
      ? "Apply or revert the macro draft to save."
      : saveAs
        ? "Writes a new file. Load it in Vial to update the keyboard."
        : !view.dirty
          ? `${file} is up to date.`
          : `Writes ${file}. Load it in Vial to update the keyboard.`,
  );

  function shownChips(group: ChangeGroup) {
    return expanded[group.id] ? group.chips : group.chips.slice(0, CHIP_LIMIT);
  }
  function shownRows(group: ChangeGroup) {
    return expanded[group.id] ? group.rows : group.rows.slice(0, ROW_LIMIT);
  }
  function hidden(group: ChangeGroup): number {
    return expanded[group.id]
      ? 0
      : Math.max(0, group.chips.length - CHIP_LIMIT) + Math.max(0, group.rows.length - ROW_LIMIT);
  }
</script>

<div class="sheet-scrim" aria-hidden="true" onclick={oncancel}></div>
<div
  class="changes-sheet"
  role="dialog"
  aria-modal="true"
  aria-label="Review changes before save"
  tabindex="-1"
  data-autofocus={review.focusSave ? undefined : ""}
  use:focusScope={oncancel}
>
  <header class="sheet-head">
    <h2>{total ? `${total} ${total === 1 ? "change" : "changes"}` : "No changes"}</h2>
    <div class="compare" role="group" aria-label="Compared with">
      <span class="compare-label">Compared with</span>
      <button aria-pressed={!git} title={`Compare with ${file} as last saved`} onclick={() => oncompare("disk")}>Last save</button>
      <button
        aria-pressed={git}
        title={`Compare with ${file} in the last Git commit. Names are always compared with the last save.`}
        onclick={() => oncompare("git")}>Last commit</button
      >
    </div>
  </header>
  {#if review.error}<p class="sheet-alert" role="alert">{review.error}</p>{/if}
  <div class="sheet-body">
    {#if !groups.length}<p class="sheet-empty">{git ? "Everything matches the last commit." : "Everything matches the last save."}</p>{/if}
    {#each groups as group (group.id)}
      <section class="change-group" aria-labelledby={`changes-${group.id}`}>
        <h3 id={`changes-${group.id}`}>
          {#if group.jump}<button
              class="group-link"
              title={group.jump.kind === "slot" ? `Open ${group.title}` : `Show ${group.title.split(" · ")[0]!.toLowerCase()}`}
              onclick={() => onjump(group.jump!)}>{group.title}</button
            >{:else}<span>{group.title}</span>{/if}
          <span class="group-count">{group.count}</span>
        </h3>
        {#if group.chips.length}
          <div class="change-chips">
            {#each shownChips(group) as row (row.id)}
              <button class="change-chip" title={row.title} onclick={() => row.jump && onjump(row.jump)}>{row.text}</button>
            {/each}
          </div>
        {/if}
        {#if group.rows.length}
          <ul class="change-rows">
            {#each shownRows(group) as row (row.id)}
              <li>
                {#if row.jump}
                  <button class="change-row" title={row.title} onclick={() => onjump(row.jump!)}>
                    <span class="row-text">{row.text}</span>{#if row.detail}<span class="row-detail">{row.detail}</span>{/if}
                  </button>
                {:else}
                  <div class="change-row static" title={row.title}>
                    <span class="row-text">{row.text}</span>{#if row.detail}<span class="row-detail">{row.detail}</span>{/if}
                  </div>
                {/if}
              </li>
            {/each}
          </ul>
        {/if}
        {#if hidden(group)}<button class="more" onclick={() => (expanded[group.id] = true)}>Show {hidden(group)} more</button>{/if}
      </section>
    {/each}
  </div>
  <footer class="sheet-foot">
    <p class="consequence" title={`vile never writes to the keyboard itself. Names are saved next to the layout in ${file}.vile.json.`}>{consequence}</p>
    <div class="sheet-actions">
      {#if review.error && !review.saveAs}<button class="quiet" onclick={onsaveas}>Save to another file…</button>{/if}
      <button class="quiet" onclick={oncancel}>Cancel</button>
      <button
        class="sheet-primary"
        data-autofocus={review.focusSave ? "" : undefined}
        disabled={!!disabledReason}
        title={disabledReason || `${primary} · ${primaryModifierLabel(navigator.platform)}S`}
        onclick={onsave}>{primary}<kbd>{primaryModifierLabel(navigator.platform)}S</kbd></button
      >
    </div>
  </footer>
</div>

<style>
  .sheet-scrim {
    position: fixed;
    inset: 0;
    z-index: 20;
  }
  /* The sheet itself takes focus when opened from the count; the controls inside show the focus ring. */
  .changes-sheet:focus {
    outline: none;
  }
  .changes-sheet {
    position: fixed;
    z-index: 21;
    top: calc(60px + var(--space-2));
    right: max(var(--space-5), calc((100vw - 1920px) / 2 + var(--space-5)));
    width: min(560px, calc(100vw - 2 * var(--space-4)));
    max-height: calc(100vh - 60px - var(--space-2) - var(--space-5));
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--chrome);
    border: 1px solid var(--edge);
    border-radius: var(--r-panel);
    box-shadow: 0 12px 30px #0004;
    color: var(--fg);
    font-size: 12px;
  }
  .sheet-head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-4) var(--space-4) var(--space-3);
    border-bottom: 1px solid var(--subtle-edge);
  }
  h2 {
    margin: 0;
    font-size: 15px;
    font-weight: 550;
    letter-spacing: -0.2px;
  }
  .compare {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    padding: var(--space-1);
    background: var(--well);
    border: 1px solid var(--subtle-edge);
    border-radius: 9px;
  }
  .compare-label {
    padding: 0 var(--space-2);
    font-size: 11px;
    color: var(--muted);
  }
  .compare button {
    min-height: 24px;
    padding: 0 var(--space-2);
    border-color: transparent;
    background: transparent;
    color: var(--muted);
    font-size: 11px;
  }
  .compare button[aria-pressed="true"] {
    border-color: var(--edge);
    background: var(--control);
    color: var(--fg);
  }
  .sheet-alert {
    margin: var(--space-3) var(--space-4) 0;
    color: var(--changed);
  }
  .sheet-body {
    flex: 1 1 auto;
    min-height: 0;
    overflow: auto;
    padding: 0 var(--space-4);
  }
  .sheet-empty {
    margin: 0;
    padding: var(--space-4) 0;
    color: var(--muted);
    font-size: 13px;
  }
  .change-group {
    padding: var(--space-3) 0;
    border-bottom: 1px solid var(--subtle-edge);
  }
  .change-group:last-child {
    border-bottom: 0;
  }
  h3 {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
    margin: 0 0 var(--space-2);
    font-size: 12px;
    font-weight: 550;
  }
  .group-link {
    min-height: 0;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--fg);
    font: inherit;
  }
  .group-link:hover {
    color: var(--cursor);
  }
  .group-count {
    color: var(--muted);
    font-size: 11px;
    font-weight: 400;
  }
  .change-chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }
  .change-chip {
    min-height: 24px;
    padding: 0 var(--space-2);
    border: 1px solid var(--subtle-edge);
    border-radius: 6px;
    background: var(--panel);
    font-size: 12px;
  }
  .change-rows {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: var(--space-1);
  }
  .change-row {
    width: 100%;
    min-height: 28px;
    display: flex;
    align-items: baseline;
    justify-content: flex-start;
    gap: var(--space-2);
    padding: var(--space-1) var(--space-2);
    border: 0;
    border-radius: 6px;
    background: transparent;
    text-align: left;
    font-size: 12px;
  }
  .row-text {
    flex: none;
  }
  .row-detail {
    min-width: 0;
    color: var(--muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .change-chip:hover,
  .change-row:not(.static):hover {
    background: var(--hover);
  }
  .more {
    margin-top: var(--space-2);
    min-height: 24px;
    padding: 0 var(--space-2);
    border-color: transparent;
    background: transparent;
    color: var(--muted);
    font-size: 11px;
  }
  .sheet-foot {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    border-top: 1px solid var(--subtle-edge);
  }
  .consequence {
    margin: 0;
    color: var(--muted);
  }
  .sheet-actions {
    display: flex;
    gap: var(--space-2);
    margin-left: auto;
  }
  .quiet {
    border-color: transparent;
    background: transparent;
    color: var(--muted);
  }
  .sheet-primary {
    background: var(--fg);
    color: var(--on-primary);
    border-color: transparent;
    padding: var(--space-2) var(--space-3);
    font-weight: 550;
  }
  .sheet-primary:hover:not(:disabled) {
    opacity: 0.86;
    background: var(--fg);
  }
  .sheet-primary kbd {
    color: inherit;
    opacity: 0.65;
  }
  @media (max-width: 760px) {
    .changes-sheet {
      right: var(--space-4);
    }
  }
</style>
