<script lang="ts">
  // The window's title bar. On Windows and Linux the system frame is off and
  // this draws the caption buttons; on macOS the native traffic lights sit
  // over the left edge and this only provides the drag area.
  import { inTauri } from "../lib/api";
  import { store } from "../lib/store.svelte";
  import { updater } from "../lib/updater.svelte";

  const mac = $derived(store.view?.platform === "macos");
  let maximized = $state(false);

  async function win() {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    return getCurrentWindow();
  }

  $effect(() => {
    if (!inTauri) return;
    let off: (() => void) | undefined;
    (async () => {
      const w = await win();
      maximized = await w.isMaximized();
      off = await w.onResized(async () => (maximized = await w.isMaximized()));
    })();
    return () => off?.();
  });

  const minimize = async () => inTauri && (await win()).minimize();
  const toggle = async () => inTauri && (await win()).toggleMaximize();
  const close = async () => inTauri && (await win()).close();

  const pct = $derived(updater.total ? Math.round((updater.done / updater.total) * 100) : 0);
</script>

<header class="titlebar" class:mac data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region>
    <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
      <rect width="16" height="16" rx="3.5" fill="var(--accent)" />
      <path d="M5 3.5h3.7c1.6 0 2.6.8 2.6 2 0 .8-.4 1.4-1.1 1.7.9.3 1.5 1 1.5 2 0 1.4-1.1 2.3-2.8 2.3H5zm1.6 1.4v2h1.8c.7 0 1.1-.4 1.1-1s-.4-1-1.1-1zm0 3.3v2.4h2c.8 0 1.2-.4 1.2-1.2s-.4-1.2-1.2-1.2z" fill="#fff" />
    </svg>
    <span data-tauri-drag-region>BeamLink</span>
  </div>

  <div class="spacer" data-tauri-drag-region></div>

  {#if updater.status === "available"}
    <button class="update" onclick={() => updater.install()} title={updater.notes ?? undefined}>
      Update to {updater.version} — restart
    </button>
  {:else if updater.status === "downloading"}
    <span class="update busy">Downloading update {pct}%</span>
  {:else if updater.status === "ready"}
    <span class="update busy">Restarting…</span>
  {/if}

  {#if !mac}
    <div class="controls">
      <button class="ctl" onclick={minimize} aria-label="Minimize">
        <svg viewBox="0 0 10 10" width="10" height="10"><path d="M0 5h10" stroke="currentColor" /></svg>
      </button>
      <button class="ctl" onclick={toggle} aria-label={maximized ? "Restore" : "Maximize"}>
        {#if maximized}
          <svg viewBox="0 0 10 10" width="10" height="10" fill="none" stroke="currentColor">
            <path d="M2.5 2.5V.5h7v7h-2" /><rect x=".5" y="2.5" width="7" height="7" />
          </svg>
        {:else}
          <svg viewBox="0 0 10 10" width="10" height="10" fill="none" stroke="currentColor"><rect x=".5" y=".5" width="9" height="9" /></svg>
        {/if}
      </button>
      <button class="ctl close" onclick={close} aria-label="Close">
        <svg viewBox="0 0 10 10" width="10" height="10"><path d="M0 0l10 10M10 0 0 10" stroke="currentColor" /></svg>
      </button>
    </div>
  {/if}
</header>

<style>
  .titlebar {
    height: 34px;
    flex: none;
    display: flex;
    align-items: center;
    background: var(--chrome);
    border-bottom: 1px solid var(--line);
    padding-left: 12px;
    -webkit-app-region: drag;
  }
  .titlebar.mac {
    /* Room for the traffic lights. */
    padding-left: 80px;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-2);
  }
  .spacer {
    flex: 1;
    height: 100%;
  }
  .update {
    margin-right: 10px;
    height: 22px;
    padding: 0 9px;
    border-radius: 4px;
    font-size: 11.5px;
    font-weight: 600;
    background: var(--accent);
    color: #fff;
    -webkit-app-region: no-drag;
    display: inline-flex;
    align-items: center;
  }
  .update.busy {
    background: var(--surface-3);
    color: var(--text-2);
  }
  .controls {
    display: flex;
    height: 100%;
    -webkit-app-region: no-drag;
  }
  .ctl {
    width: 46px;
    height: 100%;
    display: grid;
    place-items: center;
    color: var(--text-2);
    transition: background 0.1s, color 0.1s;
  }
  .ctl svg {
    shape-rendering: crispEdges;
  }
  .ctl:hover {
    background: rgba(255, 255, 255, 0.07);
    color: var(--text);
  }
  .ctl.close:hover {
    background: #c42b1c;
    color: #fff;
  }
</style>
