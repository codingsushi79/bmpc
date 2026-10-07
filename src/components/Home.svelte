<script lang="ts">
  import BeamText from "./BeamText.svelte";
  import Icon from "./Icon.svelte";
  import ServerRow from "./ServerRow.svelte";
  import { store } from "../lib/store.svelte";
  import { ago, parseAddress, serverKey } from "../lib/beam";
  import type { Server } from "../lib/types";

  let quick = $state("");
  let quickError = $state<string | null>(null);

  const byKey = $derived(new Map(store.servers.map((s) => [serverKey(s), s] as [string, Server])));
  const recents = $derived((store.view?.settings.recents ?? []).slice(0, 5));
  const favorites = $derived(store.view?.settings.favorites ?? []);
  const popular = $derived(store.servers.filter((s) => !s.password).slice(0, 8));
  const view = $derived(store.view);

  const status = $derived.by(() => {
    if (!view) return "";
    if (store.inSession) return "playing";
    if (view.game?.launcher) return "menu";
    if (view.launcher_running) return "starting";
    return "idle";
  });

  async function quickJoin(e: SubmitEvent) {
    e.preventDefault();
    const target = parseAddress(quick);
    if (!target) {
      quickError = "Enter an address like 192.168.1.20 or host:30814";
      return;
    }
    await store.join({ ...target, name: `${target.ip}:${target.port}`, map: "", at: 0 });
  }
</script>

<div class="home">
  <div class="main">
    <section class="panel status">
      <div class="s-text">
        <div class="label">BeamMP</div>
        {#if status === "playing"}
          <div class="s-line">Playing on <BeamText text={store.inSession?.name ?? "a server"} /></div>
        {:else if status === "menu"}
          <div class="s-line">In the game menu. Pick a server here and it joins automatically.</div>
        {:else if status === "starting"}
          <div class="s-line">Starting BeamNG.drive…</div>
        {:else if !store.canPlay}
          <div class="s-line">BeamNG.drive doesn't run on {view?.platform === "macos" ? "macOS" : "this system"}. You can browse and save servers here.</div>
        {:else if view && !view.ready}
          <div class="s-line">Setup isn't finished. <button class="link" onclick={() => (store.setupOpen = true)}>Finish setup</button></div>
        {:else}
          <div class="s-line">Not running</div>
        {/if}
      </div>
      <form class="quick" onsubmit={quickJoin}>
        <input bind:value={quick} placeholder="Connect to address" spellcheck="false" oninput={() => (quickError = null)} />
        <button class="btn btn-secondary" disabled={!quick.trim() || !store.canPlay}>Connect</button>
      </form>
      {#if quickError}<p class="q-error">{quickError}</p>{/if}
    </section>

    {#if recents.length}
      <section>
        <h2 class="section-title">Recent</h2>
        <div class="panel">
          {#each recents as r (serverKey(r))}
            {@const live = byKey.get(serverKey(r)) ?? null}
            <ServerRow name={r.name} server={live} detail={ago(r.at)} onopen={live ? () => (store.selected = live) : undefined}>
              {#snippet actions()}
                <button class="btn btn-secondary btn-sm" disabled={!store.canPlay} onclick={(e) => { e.stopPropagation(); store.join(live ?? r); }}>Join</button>
              {/snippet}
            </ServerRow>
          {/each}
        </div>
      </section>
    {/if}

    <section>
      <div class="head">
        <h2 class="section-title">Popular right now</h2>
        <button class="btn btn-quiet btn-sm" onclick={() => (store.page = "servers")}>All servers <Icon name="chevron" size={12} /></button>
      </div>
      <div class="panel">
        {#if !popular.length}
          <p class="empty muted">{store.loadingServers ? "Loading servers…" : (store.serversError ?? "No servers to show.")}</p>
        {/if}
        {#each popular as s (serverKey(s))}
          <ServerRow name={s.name} server={s} detail={s.official ? "Official" : s.tags.slice(0, 2).join(", ")} ping={store.pings[serverKey(s)]} onopen={() => (store.selected = s)}>
            {#snippet actions()}
              <button class="btn btn-secondary btn-sm" disabled={!store.canPlay} onclick={(e) => { e.stopPropagation(); store.join(s); }}>Join</button>
            {/snippet}
          </ServerRow>
        {/each}
      </div>
    </section>
  </div>

  <aside class="side">
    <section class="panel box">
      <div class="label">Online now</div>
      <div class="kv"><span>Players</span><b>{store.totals.players.toLocaleString()}</b></div>
      <div class="kv"><span>Servers</span><b>{store.totals.servers.toLocaleString()}</b></div>
    </section>

    <section class="panel box">
      <div class="label">Favorites</div>
      {#if favorites.length}
        <ul class="favs">
          {#each favorites.slice(0, 8) as f (serverKey(f))}
            {@const live = byKey.get(serverKey(f))}
            <li>
              <button onclick={() => live && (store.selected = live)} disabled={!live}>
                <span class="f-name"><BeamText text={live?.name ?? f.name} /></span>
                <span class="f-count">{live ? `${live.players}/${live.max_players}` : "Offline"}</span>
              </button>
            </li>
          {/each}
        </ul>
        {#if favorites.length > 8}<button class="btn btn-quiet btn-sm" onclick={() => (store.page = "library")}>All {favorites.length}</button>{/if}
      {:else}
        <p class="muted small">Star servers in the browser to keep them here.</p>
      {/if}
    </section>
  </aside>
</div>

<style>
  .home {
    height: 100%;
    overflow-y: auto;
    display: grid;
    grid-template-columns: minmax(0, 1fr) 300px;
    gap: 20px;
    padding: 24px 28px 40px;
    align-content: start;
  }
  .main {
    min-width: 0;
  }
  section + section {
    margin-top: 24px;
  }
  .status {
    padding: 16px;
  }
  .s-line {
    margin-top: 4px;
    font-size: 14px;
  }
  .link {
    color: var(--accent);
    font-weight: 550;
  }
  .link:hover {
    text-decoration: underline;
  }
  .quick {
    display: flex;
    gap: 8px;
    margin-top: 14px;
  }
  .quick input {
    flex: 1;
  }
  .q-error {
    margin: 8px 0 0;
    font-size: 12.5px;
    color: var(--bad);
  }
  h2 {
    margin-bottom: 10px;
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 10px;
  }
  .head h2 {
    margin: 0;
  }
  .empty {
    margin: 0;
    padding: 16px;
  }
  .side {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .side section + section {
    margin-top: 0;
  }
  .box {
    padding: 14px;
  }
  .box .label {
    margin-bottom: 8px;
  }
  .kv {
    display: flex;
    justify-content: space-between;
    padding: 4px 0;
    color: var(--text-2);
  }
  .kv b {
    color: var(--text);
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .favs {
    list-style: none;
    margin: 0 -6px;
    padding: 0;
  }
  .favs button {
    width: 100%;
    display: flex;
    justify-content: space-between;
    gap: 10px;
    padding: 6px;
    border-radius: var(--radius);
    text-align: left;
  }
  .favs button:hover:not(:disabled) {
    background: var(--surface-3);
  }
  .favs button:disabled {
    opacity: 0.6;
    cursor: default;
  }
  .f-name {
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .f-count {
    flex: none;
    font-size: 12.5px;
    color: var(--text-2);
    font-variant-numeric: tabular-nums;
  }
  .small {
    margin: 0;
    font-size: 12.5px;
  }
</style>
