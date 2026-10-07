<script lang="ts">
  import Icon from "./Icon.svelte";
  import BeamText from "./BeamText.svelte";
  import { store, type Page } from "../lib/store.svelte";

  const tabs: { id: Page; label: string }[] = [
    { id: "home", label: "Home" },
    { id: "servers", label: "Servers" },
    { id: "direct", label: "Direct connect" },
    { id: "library", label: "Library" },
    { id: "mods", label: "Mods" },
  ];

  const view = $derived(store.view);
  const running = $derived(view?.launcher_running ?? false);
  const account = $derived(view?.account);
</script>

<nav class="nav">
  <div class="tabs" role="tablist">
    {#each tabs as tab}
      <button role="tab" aria-selected={store.page === tab.id} class:active={store.page === tab.id} onclick={() => (store.page = tab.id)}>
        {tab.label}
      </button>
    {/each}
  </div>

  <div class="right">
    {#if store.inSession}
      <span class="status on"><span class="dot good"></span><BeamText text={store.inSession.name ?? "On a server"} /></span>
    {:else if running}
      <span class="status"><span class="dot warn"></span>BeamMP running</span>
    {/if}
    <button class="account" class:active={store.page === "account"} onclick={() => (store.page = "account")}>
      <Icon name="account" size={15} />
      {account?.signed_in ? account.username : "Sign in"}
    </button>
    <button class="icon-btn" class:active={store.page === "settings"} onclick={() => (store.page = "settings")} aria-label="Settings" title="Settings">
      <Icon name="settings" size={17} />
    </button>
    <button class="btn btn-primary play" disabled={!store.canPlay} onclick={() => store.play()} title={store.canPlay ? "Start BeamMP" : "BeamNG.drive doesn't run on this OS"}>
      <Icon name="play" size={13} /> {running ? "Running" : "Play"}
    </button>
  </div>
</nav>

<style>
  .nav {
    height: 48px;
    flex: none;
    display: flex;
    align-items: stretch;
    justify-content: space-between;
    padding: 0 12px 0 20px;
    border-bottom: 1px solid var(--line);
    background: var(--bg);
  }
  .tabs {
    display: flex;
    gap: 22px;
  }
  .tabs button {
    position: relative;
    font-size: 13.5px;
    font-weight: 550;
    color: var(--muted);
    transition: color 0.12s;
  }
  .tabs button:hover {
    color: var(--text);
  }
  .tabs button.active {
    color: var(--text);
  }
  .tabs button.active::after {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    bottom: -1px;
    height: 2px;
    background: var(--accent);
  }
  .right {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .status {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    max-width: 260px;
    margin-right: 6px;
    font-size: 12.5px;
    color: var(--text-2);
    white-space: nowrap;
    overflow: hidden;
  }
  .account {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 30px;
    padding: 0 10px;
    border-radius: var(--radius);
    font-size: 13px;
    color: var(--text-2);
    max-width: 200px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .account:hover,
  .account.active,
  .icon-btn.active {
    background: var(--surface-3);
    color: var(--text);
  }
  .play {
    margin-left: 4px;
    min-width: 92px;
  }
</style>
