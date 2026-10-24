<script lang="ts">
  import type { KeyOverrideDto } from "../lib/types";
  import Field from "./Field.svelte";
  import { ui } from "../lib/state.svelte";
  import { MOD_BITS, KEY_OVERRIDE_OPTION_BITS, hasBit, toggleBit } from "../lib/bits";
  const modFields: [string, "triggerMods" | "negativeModMask" | "suppressedMods"][] = [
    ["Trigger mods", "triggerMods"],
    ["Negative mods", "negativeModMask"],
    ["Suppress mods", "suppressedMods"],
  ];
  let {
    keyOverrides,
    layerCount,
    onsave,
  }: {
    keyOverrides: KeyOverrideDto[];
    layerCount: number;
    onsave: (value: KeyOverrideDto) => void;
  } = $props();
</script>

<section>
  <h2>Key overrides</h2>
  {#each keyOverrides as ko (ko.index)}<details
      ontoggle={(e) => {
        if (!e.currentTarget.open && ui.inspectorPaths[0]?.startsWith(`key_override[${ko.index}]`))
          ui.inspectorPaths = [];
      }}
    >
      <summary>Override {ko.index}</summary><Field
        path={`key_override[${ko.index}].trigger`}
        label="Trigger"
      /><Field path={`key_override[${ko.index}].replacement`} label="Replace" />
      <fieldset>
        <legend>Layers</legend
        >{#each Array.from({ length: Math.min(layerCount, 16) }, (_, i) => i) as i}<label
            ><input
              type="checkbox"
              checked={hasBit(ko.layers, i)}
              onchange={() => onsave({ ...ko, layers: toggleBit(ko.layers, i) })}
            />L{i}</label
          >{/each}
      </fieldset>
      {#each modFields as [name, field]}<fieldset>
          <legend>{name}</legend>{#each MOD_BITS as bit}<label
              ><input
                type="checkbox"
                checked={hasBit(ko[field as "triggerMods"], bit.bit)}
                onchange={() =>
                  onsave({ ...ko, [field]: toggleBit(ko[field as "triggerMods"], bit.bit) })}
              />{bit.name}</label
            >{/each}
        </fieldset>{/each}
      <fieldset>
        <legend>Options</legend>{#each KEY_OVERRIDE_OPTION_BITS as bit}<label
            ><input
              type="checkbox"
              checked={hasBit(ko.options, bit.bit)}
              onchange={() => onsave({ ...ko, options: toggleBit(ko.options, bit.bit) })}
            />{bit.name}</label
          >{/each}
      </fieldset>
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
