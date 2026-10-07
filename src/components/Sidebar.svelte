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
  <button class="brand" onclick={() => (store.page = "home")} aria-label="Home">
    <svg viewBox="0 0 64 64" width="40" height="40" aria-hidden="true">
      <defs>
        <linearGradient id="lg" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0" stop-color="#ff2e63" />
          <stop offset="1" stop-color="#ff8a00" />
        </linearGradient>
      </defs>
      <rect x="2" y="2" width="60" height="60" rx="14" fill="#111119" stroke="url(#lg)" stroke-opacity=".6" />
      <path
        transform="skewX(-14) translate(10 0)"
        d="M21 16h15c7 0 11 3.6 11 9 0 3.6-2 6.3-5.2 7.6 4 1.1 6.6 4.2 6.6 8.3 0 6.1-4.9 10.1-12.4 10.1H21zm6.8 5.8v8h7.4c2.9 0 4.6-1.5 4.6-4s-1.7-4-4.6-4zm0 13.8v9.2h8.5c3.2 0 5.1-1.7 5.1-4.6s-1.9-4.6-5.1-4.6z"
        fill="url(#lg)"
        fill-rule="evenodd"
      />
      <g fill="#ff2e63"><rect x="9" y="27" width="11" height="2.6" rx="1.3" /><rect x="7" y="32" width="12" height="2.6" rx="1.3" opacity=".7" /></g>
    </svg>
  </button>

  <div class="items">
    {#each items as item}
      <button class="item" class:active={store.page === item.id} onclick={() => (store.page = item.id)} title={item.label}>
        <Icon name={item.icon} size={21} />
        <span>{item.label}</span>
      </button>
    {/each}
  </div>

  <div class="foot">
    <div class="state" title={session ? `On ${session.name}` : running ? "BeamMP running" : "BeamMP not running"}>
      <span class="dot" class:good={!!session} class:warn={running && !session}></span>
    </div>
    <button
      class="play"
      disabled={!store.canPlay}
      onclick={() => store.play()}
      title={store.canPlay ? "Launch BeamMP" : "BeamNG.drive is not available on this OS"}
    >
      <Icon name="play" size={20} />
    </button>
  </div>
</nav>

<style>
  .sidebar {
    width: 92px;
    flex: none;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 18px 0 16px;
    background: linear-gradient(180deg, rgba(14, 14, 22, 0.96), rgba(8, 8, 12, 0.98));
    border-right: 1px solid var(--line);
    z-index: 5;
  }
  .brand {
    margin-bottom: 26px;
    filter: drop-shadow(0 6px 18px rgba(255, 46, 99, 0.35));
  }
  .items {
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex: 1;
  }
  .item {
    position: relative;
    width: 70px;
    padding: 10px 0 8px;
    border-radius: 12px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 5px;
    color: var(--muted);
    font-family: var(--display);
    font-weight: 700;
    font-size: 12px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    transition: color 0.15s, background 0.15s;
  }
  .item:hover {
    color: var(--text);
    background: rgba(255, 255, 255, 0.04);
  }
  .item.active {
    color: #fff;
    background: linear-gradient(180deg, rgba(255, 46, 99, 0.16), rgba(255, 46, 99, 0.04));
  }
  .item.active::before {
    content: "";
    position: absolute;
    left: -11px;
    top: 12px;
    bottom: 12px;
    width: 3px;
    border-radius: 3px;
    background: var(--grad);
    box-shadow: 0 0 12px var(--accent);
  }
  .item.active :global(svg) {
    color: var(--accent);
  }
  .foot {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
  }
  .play {
    width: 52px;
    height: 52px;
    border-radius: 16px;
    display: grid;
    place-items: center;
    background: var(--grad);
    color: #fff;
    box-shadow: 0 10px 30px -8px var(--accent-glow);
    transition: transform 0.15s;
  }
  .play:hover:not(:disabled) {
    transform: scale(1.06);
  }
</style>
