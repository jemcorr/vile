<script lang="ts">
  import type { TapDanceDto } from "../lib/types";
  import Field from "./Field.svelte";
  import { ui } from "../lib/state.svelte";
  let { tapDances, onsave }: { tapDances: TapDanceDto[]; onsave: (value: TapDanceDto) => void } =
    $props();
</script>

<section>
  <h2>Tap dances</h2>
  {#each tapDances as td (td.index)}<details
      ontoggle={(e) => {
        if (!e.currentTarget.open && ui.inspectorPaths[0]?.startsWith(`tap_dance[${td.index}]`))
          ui.inspectorPaths = [];
      }}
    >
      <summary>Tap dance {td.index}</summary
      >{#each ["Tap", "Hold", "Double tap", "Tap and hold"] as name, i}<Field
          path={`tap_dance[${td.index}][${i}]`}
          label={name}
        />{/each}<label
        >Tapping term <input
          type="number"
          min="0"
          max="65535"
          value={td.tappingTerm}
          onchange={(e) => onsave({ ...td, tappingTerm: Number(e.currentTarget.value) })}
        /> ms</label
      >
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
  input {
    width: 80px;
  }
  details :global(.field) {
    margin-bottom: 4px;
  }
</style>
