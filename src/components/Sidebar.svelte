<script lang="ts">
  import Icon from "./Icon.svelte";
  import { store, type Page } from "../lib/store.svelte";

  const items: { id: Page; label: string; icon: string }[] = [
    { id: "home", label: "Home", icon: "home" },
    { id: "servers", label: "Servers", icon: "servers" },
    { id: "library", label: "Library", icon: "library" },
    { id: "mods", label: "Mods", icon: "mods" },
    { id: "account", label: "Account", icon: "account" },
    { id: "settings", label: "Settings", icon: "settings" },
  ];

  const running = $derived(store.view?.launcher_running ?? false);
  const session = $derived(store.inSession);
</script>

<nav class="sidebar">
  <div class="items">
    {#each items as item}
      <button class="item" class:active={store.page === item.id} onclick={() => (store.page = item.id)}>
        <Icon name={item.icon} size={19} />
        <span>{item.label}</span>
      </button>
    {/each}
  </div>

  <div class="foot">
    <span
      class="dot"
      class:good={!!session}
      class:warn={running && !session}
      title={session ? `On ${session.name}` : running ? "BeamMP running" : "BeamMP not running"}
    ></span>
    <button
      class="play"
      disabled={!store.canPlay}
      onclick={() => store.play()}
      title={store.canPlay ? "Launch BeamMP" : "BeamNG.drive is not available on this OS"}
      aria-label="Play"
    >
      <Icon name="play" size={18} />
    </button>
  </div>
</nav>

<style>
  .sidebar {
    width: 76px;
    flex: none;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 10px 0 14px;
    background: var(--chrome);
    border-right: 1px solid var(--line);
  }
  .items {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
  }
  .item {
    position: relative;
    width: 64px;
    padding: 9px 0 7px;
    border-radius: var(--radius);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    color: var(--muted);
    font-size: 11px;
    font-weight: 550;
    transition: color 0.12s, background 0.12s;
  }
  .item:hover {
    color: var(--text);
  }
  .item.active {
    color: var(--text);
    background: var(--surface-2);
  }
  .item.active::before {
    content: "";
    position: absolute;
    left: -6px;
    top: 10px;
    bottom: 10px;
    width: 2px;
    border-radius: 2px;
    background: var(--accent);
  }
  .foot {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
  }
  .play {
    width: 44px;
    height: 44px;
    border-radius: var(--radius);
    display: grid;
    place-items: center;
    background: var(--accent);
    color: #fff;
    transition: background 0.12s;
  }
  .play:hover:not(:disabled) {
    background: var(--accent-hover);
  }
</style>
