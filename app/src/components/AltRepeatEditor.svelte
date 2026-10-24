<script lang="ts">
  import type { AltRepeatDto } from "../lib/types";
  import Field from "./Field.svelte";
  import { ui } from "../lib/state.svelte";
  import { MOD_BITS, ALT_REPEAT_OPTION_BITS, hasBit, toggleBit } from "../lib/bits";
  const masks = [
    { name: "Allowed modifiers", bits: MOD_BITS },
    { name: "Options", bits: ALT_REPEAT_OPTION_BITS },
  ];
  let {
    altRepeatKeys,
    onsave,
  }: { altRepeatKeys: AltRepeatDto[]; onsave: (value: AltRepeatDto) => void } = $props();
</script>

<section>
  <h2>Alt repeat</h2>
  {#each altRepeatKeys as ar (ar.index)}<details
      ontoggle={(e) => {
        if (
          !e.currentTarget.open &&
          ui.inspectorPaths[0]?.startsWith(`alt_repeat_key[${ar.index}]`)
        )
          ui.inspectorPaths = [];
      }}
    >
      <summary>Alt repeat {ar.index}</summary><Field
        path={`alt_repeat_key[${ar.index}].keycode`}
        label="Key"
      /><Field path={`alt_repeat_key[${ar.index}].alt_keycode`} label="Alternate" />
      {#each masks as mask, index}<fieldset>
          <legend>{mask.name}</legend>{#each mask.bits as bit}<label
              ><input
                type="checkbox"
                checked={hasBit(index === 0 ? ar.allowedMods : ar.options, bit.bit)}
                onchange={() =>
                  onsave({
                    ...ar,
                    ...(index === 0
                      ? { allowedMods: toggleBit(ar.allowedMods, bit.bit) }
                      : { options: toggleBit(ar.options, bit.bit) }),
                  })}
              />{bit.name}</label
            >{/each}
        </fieldset>{/each}
    </details>{/each}
</section>

<style>
  h2 {
    font-size: 15px;
  }
  details {
    padding: 8px 0;
    border-bottom: 1px solid var(--key-border);
  }
  summary {
    cursor: pointer;
    margin-bottom: 8px;
  }
  fieldset {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    font-size: 12px;
    border: 1px solid var(--key-border);
    margin-top: 8px;
  }
  label {
    display: flex;
    align-items: center;
  }
</style>
