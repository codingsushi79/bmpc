<script lang="ts">
  import Sidebar from "./components/Sidebar.svelte";
  import Home from "./components/Home.svelte";
  import Browser from "./components/Browser.svelte";
  import Library from "./components/Library.svelte";
  import Mods from "./components/Mods.svelte";
  import Account from "./components/Account.svelte";
  import SettingsPage from "./components/Settings.svelte";
  import ServerDetail from "./components/ServerDetail.svelte";
  import Setup from "./components/Setup.svelte";
  import LogDrawer from "./components/LogDrawer.svelte";
  import DirectConnect from "./components/DirectConnect.svelte";
  import BeamText from "./components/BeamText.svelte";
  import Icon from "./components/Icon.svelte";
  import { api } from "./lib/api";
  import { store } from "./lib/store.svelte";

  let booted = $state(false);

  $effect(() => {
    (async () => {
      await store.refresh();
      // First run (or the game moved): walk through setup before anything else.
      if (store.view && store.view.game_supported && !store.view.ready) store.setupOpen = true;
      booted = true;
      store.loadServers();
      try {
        await api.resumeAccount();
        await store.refresh();
      } catch {}
    })();

    // Light polls: state is a few small file reads in Rust; the server list
    // is cached there too, so this never hammers the BeamMP backend.
    const state = setInterval(() => {
      if (!document.hidden) store.refresh();
    }, 2500);
    const list = setInterval(() => {
      if (!document.hidden && (store.page === "servers" || store.page === "home" || store.page === "library")) store.loadServers();
    }, 60_000);
    return () => {
      clearInterval(state);
      clearInterval(list);
    };
  });

  const view = $derived(store.view);
  const launcherState = $derived(
    !view ? "…" : view.launcher_running ? (view.game?.launcher ? "Game connected" : "Starting game…") : view.launcher_exit ?? "Not running",
  );
</script>

<div class="app">
  <Sidebar />
  <div class="main">
    <div class="page">
      {#if booted}
        {#if store.page === "home"}<Home />
        {:else if store.page === "servers"}<Browser />
        {:else if store.page === "library"}<Library />
        {:else if store.page === "mods"}<Mods />
        {:else if store.page === "account"}<Account />
        {:else if store.page === "settings"}<SettingsPage />{/if}
      {/if}
      <ServerDetail />
      <DirectConnect />
    </div>
    <LogDrawer />
    <footer class="status">
      <span class="item">
        <span class="dot" class:good={!!view?.game?.launcher} class:warn={!!view?.launcher_running && !view?.game?.launcher}></span>
        BeamMP: {launcherState}
      </span>
      {#if store.inSession}
        <span class="item session">On <BeamText text={store.inSession.name ?? "server"} /></span>
      {:else if view?.join_pending}
        <span class="item pending">
          <span class="spin"><Icon name="refresh" size={12} /></span> Join queued — waiting for the game
          <button class="link" onclick={() => store.run(async () => { await api.cancelJoin(); return "Join cancelled"; })}>cancel</button>
        </span>
      {/if}
      <span class="spacer"></span>
      {#if view && !view.ready && view.game_supported}
        <button class="item warnlink" onclick={() => (store.setupOpen = true)}><Icon name="alert" size={13} /> Setup incomplete</button>
      {/if}
      <button class="item" class:on={store.logOpen} onclick={() => (store.logOpen = !store.logOpen)}><Icon name="terminal" size={13} /> Logs</button>
      <span class="item muted">v{view?.version ?? ""}</span>
    </footer>
  </div>

  {#if store.setupOpen}<Setup />{/if}

  <div class="toasts">
    {#each store.toasts as t (t.id)}
      <div class="toast {t.kind} fade-in">
        <Icon name={t.kind === "err" ? "alert" : t.kind === "info" ? "spark" : "check"} size={16} />
        <span>{t.text}</span>
      </div>
    {/each}
  </div>
</div>

<style>
  .app {
    position: relative;
    height: 100%;
    display: flex;
    background:
      radial-gradient(900px 500px at 85% -10%, rgba(255, 46, 99, 0.09), transparent 60%),
      radial-gradient(700px 500px at 10% 110%, rgba(46, 230, 255, 0.05), transparent 60%),
      var(--bg);
  }
  .main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .page {
    position: relative;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
  .status {
    height: 32px;
    flex: none;
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 10px;
    border-top: 1px solid var(--line);
    background: rgba(8, 8, 12, 0.95);
    font-size: 12px;
    color: var(--text-2);
  }
  .item {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 24px;
    padding: 0 9px;
    border-radius: 7px;
    font-size: 12px;
    white-space: nowrap;
  }
  button.item:hover,
  .item.on {
    background: rgba(255, 255, 255, 0.07);
  }
  .session {
    color: var(--good);
    max-width: 380px;
    overflow: hidden;
  }
  .pending {
    color: var(--warn);
  }
  .link {
    color: var(--text-2);
    text-decoration: underline;
    font-size: 12px;
    padding: 0;
  }
  .warnlink {
    color: var(--warn);
  }
  .spacer {
    flex: 1;
  }
  .toasts {
    position: absolute;
    right: 20px;
    bottom: 46px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 60;
    pointer-events: none;
  }
  .toast {
    display: flex;
    align-items: center;
    gap: 10px;
    max-width: 440px;
    padding: 11px 15px;
    border-radius: 11px;
    background: rgba(20, 20, 30, 0.96);
    border: 1px solid var(--line-hi);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.5);
    font-size: 13px;
    pointer-events: auto;
  }
  .toast.ok :global(svg) {
    color: var(--good);
  }
  .toast.info :global(svg) {
    color: var(--cyan);
  }
  .toast.err {
    border-color: rgba(255, 77, 109, 0.45);
  }
  .toast.err :global(svg) {
    color: var(--bad);
    flex: none;
  }
</style>
