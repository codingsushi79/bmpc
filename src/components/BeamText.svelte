<script lang="ts">
  // Renders BeamMP `^x` colour/format codes as styled text.
  import { parseCodes } from "../lib/beam";

  let { text, multiline = false }: { text: string; multiline?: boolean } = $props();
  const segments = $derived(parseCodes(multiline ? text : text.replace(/\n/g, " ")));
</script>

<span class="beam" class:multiline
  >{#each segments as s}<span
      style:color={s.color}
      style:font-weight={s.bold ? 800 : undefined}
      style:font-style={s.italic ? "italic" : undefined}
      style:text-decoration={s.underline ? "underline" : s.strike ? "line-through" : undefined}
      >{s.text}</span
    >{/each}</span
>

<style>
  .beam {
    min-width: 0;
  }
  .multiline {
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
