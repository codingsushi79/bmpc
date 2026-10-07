<script lang="ts">
  import Icon from "./Icon.svelte";
  import { api } from "../lib/api";
  import { store } from "../lib/store.svelte";
  import type { LogLine } from "../lib/types";

  let lines = $state<LogLine[]>([]);
  let box = $state<HTMLDivElement>();
  let follow = $state(true);

  // Poll only while the drawer is open; ask for new lines only.
  $effect(() => {
    if (!store.logOpen) return;
    let alive = true;
    const tick = async () => {
      const after = lines.length ? lines[lines.length - 1].seq : 0;
      try {
        const fresh = await api.launcherLog(after);
        if (fresh.length) {
          lines = [...lines, ...fresh].slice(-2000);
          if (follow) queueMicrotask(() => box && (box.scrollTop = box.scrollHeight));
        }
      } catch {}
      if (alive) setTimeout(tick, 700);
    };
    tick();
    return () => (alive = false);
  });

  function tone(text: string) {
    if (/\[(ERROR|FATAL)\]/i.test(text)) return "bad";
    if (/\[WARN/i.test(text)) return "warn";
    if (text.startsWith("──")) return "note";
    return "";
  }
</script>

{#if store.logOpen}
  <div class="drawer">
    <div class="bar">
      <Icon name="terminal" size={16} />
      <span class="eyebrow">BeamMP launcher output</span>
      <label class="follow"><input type="checkbox" bind:checked={follow} /> Follow</label>
      <button class="btn btn-ghost btn-sm" disabled={!store.view?.launcher_running} onclick={() => store.run(async () => { await api.stopLauncher(); return "BeamMP launcher stopped"; })}>
        <Icon name="stop" size={13} /> Stop launcher
      </button>
      <button class="icon-btn" onclick={() => (store.logOpen = false)} aria-label="Close"><Icon name="close" size={16} /></button>
    </div>
    <div class="log mono" bind:this={box}>
      {#each lines as line (line.seq)}<div class={tone(line.text)}>{line.text}</div>{:else}<div class="muted">Nothing yet. Output appears here when BeamLink starts the launcher.</div>{/each}
    </div>
  </div>
{/if}

<style>
  .drawer {
    height: 240px;
    flex: none;
    display: flex;
    flex-direction: column;
    border-top: 1px solid var(--line-hi);
    background: rgba(6, 6, 10, 0.96);
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 10px 6px 16px;
    border-bottom: 1px solid var(--line);
  }
  .bar .eyebrow {
    flex: 1;
  }
  .follow {
    display: flex;
    gap: 6px;
    align-items: center;
    font-size: 12.5px;
    color: var(--muted);
  }
  .log {
    flex: 1;
    overflow-y: auto;
    padding: 8px 16px;
    line-height: 1.55;
    color: var(--text-2);
    user-select: text;
  }
  .bad {
    color: #ff8fa3;
  }
  .warn {
    color: #ffd38a;
  }
  .note {
    color: #ff8fab;
  }
</style>
