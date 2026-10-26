<script lang="ts">
  let {
    value,
    onchange,
    disabled = false,
    reason = "",
  }: {
    value: number | null;
    onchange: (mods: number) => void;
    disabled?: boolean;
    reason?: string;
  } = $props();
  function toggle(bit: number): void {
    const next = (value ?? 0) ^ bit;
    onchange((next & 15) === 0 ? 0 : next);
  }
</script>

<div class="mods" title={reason}>
  {#if disabled && reason}<small>{reason}</small>{/if}
  {#if value === null}<small>Mixed</small>{/if}
  <div class="segmented" role="group" aria-label="Modifiers">
    {#each [[1, "Ctrl"], [2, "Shift"], [4, "Alt"], [8, "Gui"]] as [bit, name]}
      <button
        type="button"
        {disabled}
        aria-pressed={value === null ? "mixed" : (value & Number(bit)) !== 0}
        onclick={() => toggle(Number(bit))}>{name}</button
      >
    {/each}
  </div>
  <div class="side">
    <span>Side</span>
    <div class="segmented" role="radiogroup" aria-label="Modifier side">
      {#each [["left", "Left"], ["right", "Right"]] as [side, name]}<button
          type="button"
          role="radio"
          {disabled}
          aria-checked={((value ?? 0) & 16 ? "right" : "left") === side}
          class:active={((value ?? 0) & 16 ? "right" : "left") === side}
          title={`${name}-hand modifiers`}
          onclick={() =>
            onchange(((value ?? 0) & 15) === 0 ? 0 : ((value ?? 0) & 15) | (side === "right" ? 16 : 0))}
          >{name}</button
        >{/each}
    </div>
  </div>
</div>

<style>
  .mods {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    align-items: center;
  }
  button[aria-pressed="true"] {
    background: var(--selected);
    color: var(--fg);
  }
  small {
    color: var(--muted);
  }
  .side {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
  }
</style>
