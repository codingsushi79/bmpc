<script lang="ts">
  import Icon from "./Icon.svelte";
  import { store } from "../lib/store.svelte";

  let address = $state("");
  let error = $state<string | null>(null);

  function parse(text: string): { ip: string; port: number } | null {
    const t = text.trim();
    if (!t) return null;
    // [v6]:port, host:port, or host (default port).
    const v6 = t.match(/^\[([^\]]+)\](?::(\d+))?$/);
    if (v6) return { ip: v6[1], port: Number(v6[2] ?? 30814) };
    const parts = t.split(":");
    if (parts.length > 2) return { ip: t, port: 30814 };
    const port = parts[1] ? Number(parts[1]) : 30814;
    if (!Number.isInteger(port) || port < 1 || port > 65535) return null;
    return { ip: parts[0], port };
  }

  async function connect(e: SubmitEvent) {
    e.preventDefault();
    const target = parse(address);
    if (!target) {
      error = "Enter an address like 192.168.1.20:30814";
      return;
    }
    store.directOpen = false;
    await store.join({ ...target, name: `${target.ip}:${target.port}`, map: "", at: 0 });
  }
</script>

{#if store.directOpen}
  <div class="scrim" role="presentation" onclick={() => (store.directOpen = false)}></div>
  <form class="modal panel" onsubmit={connect}>
    <h2 class="section-title">Direct connect</h2>
    <p class="muted">For private and LAN servers that aren't on the public list.</p>
    <!-- svelte-ignore a11y_autofocus -->
    <input bind:value={address} placeholder="host:port (default port 30814)" spellcheck="false" autofocus oninput={() => (error = null)} />
    {#if error}<div class="err"><Icon name="alert" size={14} /> {error}</div>{/if}
    <div class="actions">
      <button class="btn btn-primary" disabled={!store.canPlay}><Icon name="play" size={15} /> Connect</button>
      <button type="button" class="btn btn-secondary" onclick={() => (store.directOpen = false)}>Cancel</button>
    </div>
  </form>
{/if}

<style>
  .scrim {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    z-index: 30;
  }
  .modal {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 440px;
    padding: 24px;
    z-index: 31;
    background: var(--surface);
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  h2 {
    margin: 0;
    font-size: 15px;
  }
  p {
    margin: 0;
  }
  input {
    height: 44px;
  }
  .err {
    display: flex;
    gap: 6px;
    align-items: center;
    color: var(--bad);
    font-size: 12.5px;
  }
  .actions {
    display: flex;
    gap: 8px;
  }
</style>
