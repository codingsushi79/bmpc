<script lang="ts">
  import HeroArt from "./HeroArt.svelte";
  import Icon from "./Icon.svelte";
  import ServerCard from "./ServerCard.svelte";
  import BeamText from "./BeamText.svelte";
  import { store } from "../lib/store.svelte";
  import { ago, mapName, serverKey } from "../lib/beam";
  import type { Server } from "../lib/types";

  const featured = $derived(
    store.servers.filter((s) => (s.featured || s.partner || s.official) && s.players > 0).slice(0, 6),
  );
  const popular = $derived(
    store.servers.filter((s) => !s.featured && !s.partner && !s.official && !s.password).slice(0, 6),
  );
  const byKey = $derived(new Map(store.servers.map((s) => [serverKey(s), s] as [string, Server])));
  const recents = $derived((store.view?.settings.recents ?? []).slice(0, 4));
  const running = $derived(store.view?.launcher_running ?? false);
  const account = $derived(store.view?.account);
</script>

<div class="home">
  <section class="hero">
    <HeroArt />
    <div class="shade"></div>
    <div class="hero-body">
      <div class="label">BeamNG.drive multiplayer</div>
      <h1>{store.inSession ? "You're on a server" : "Ready to drive"}</h1>
      {#if store.inSession}
        <p class="sub"><BeamText text={store.inSession.name ?? "Unknown server"} /></p>
      {:else}
        <p class="sub">Pick a server and join in one click, or launch BeamMP and browse in game.</p>
      {/if}
      <div class="cta">
        <button class="btn btn-primary btn-lg" disabled={!store.canPlay} onclick={() => store.play()}>
          <Icon name="play" size={16} />
          {running ? "BeamMP is running" : "Play"}
        </button>
        <button class="btn btn-secondary btn-lg" onclick={() => (store.page = "servers")}>Browse servers</button>
      </div>
      <dl class="stats">
        <div><dt>Drivers online</dt><dd>{store.totals.players.toLocaleString()}</dd></div>
        <div><dt>Servers</dt><dd>{store.totals.servers.toLocaleString()}</dd></div>
        <div><dt>Account</dt><dd>{account?.signed_in ? account.username : "Guest"}</dd></div>
      </dl>
      {#if !store.canPlay}
        <p class="notice">
          BeamNG.drive doesn't run on {store.view?.platform === "macos" ? "macOS" : "this system"}. You can browse and save
          servers here, then play from a Windows or Linux PC.
        </p>
      {/if}
    </div>
  </section>

  <div class="content">
    {#if recents.length}
      <section>
        <div class="head"><h2 class="section-title">Jump back in</h2></div>
        <div class="recents">
          {#each recents as r}
            {@const live = byKey.get(serverKey(r))}
            <button class="recent" onclick={() => store.join(r)} disabled={!store.canPlay}>
              <div class="r-main">
                <div class="r-name"><BeamText text={r.name} /></div>
                <div class="r-sub">{mapName(r.map)} · {ago(r.at)}</div>
              </div>
              <span class="r-players" class:off={!live}>{live ? `${live.players}/${live.max_players}` : "Offline"}</span>
            </button>
          {/each}
        </div>
      </section>
    {/if}

    <section>
      <div class="head">
        <h2 class="section-title">Featured</h2>
        <span class="muted">Official, featured and partner servers with players</span>
        <button class="btn btn-quiet btn-sm more" onclick={() => (store.page = "servers")}>View all <Icon name="chevron" size={13} /></button>
      </div>
      {#if store.loadingServers && !store.servers.length}
        <div class="grid">{#each Array(3) as _}<div class="placeholder"></div>{/each}</div>
      {:else if featured.length}
        <div class="grid">{#each featured as s (serverKey(s))}<ServerCard server={s} />{/each}</div>
      {:else}
        <p class="muted">{store.serversError ?? "Nothing featured right now."}</p>
      {/if}
    </section>

    <section>
      <div class="head"><h2 class="section-title">Most popular</h2><span class="muted">Community servers by player count</span></div>
      <div class="grid">{#each popular as s (serverKey(s))}<ServerCard server={s} />{/each}</div>
    </section>
  </div>
</div>

<style>
  .home {
    height: 100%;
    overflow-y: auto;
  }
  .hero {
    position: relative;
    height: 320px;
    overflow: hidden;
    border-bottom: 1px solid var(--line);
  }
  .shade {
    position: absolute;
    inset: 0;
    background:
      linear-gradient(90deg, var(--bg) 0%, rgba(11, 12, 15, 0.85) 38%, rgba(11, 12, 15, 0.2) 75%),
      linear-gradient(0deg, var(--bg) 0%, transparent 35%);
  }
  .hero-body {
    position: relative;
    padding: 44px 40px 0;
    max-width: 640px;
  }
  h1 {
    margin: 8px 0 6px;
    font-family: var(--display);
    font-weight: 700;
    font-size: 44px;
    line-height: 1;
    letter-spacing: 0.01em;
    text-transform: uppercase;
  }
  .sub {
    margin: 0 0 22px;
    font-size: 14.5px;
    color: var(--text-2);
  }
  .cta {
    display: flex;
    gap: 8px;
  }
  .stats {
    display: flex;
    margin: 26px 0 0;
  }
  .stats div {
    padding-right: 22px;
    margin-right: 22px;
    border-right: 1px solid var(--line-hi);
  }
  .stats div:last-child {
    border-right: none;
  }
  dt {
    font-size: 11.5px;
    color: var(--muted);
  }
  dd {
    margin: 2px 0 0;
    font-size: 17px;
    font-weight: 650;
    font-variant-numeric: tabular-nums;
    max-width: 180px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .notice {
    margin: 18px 0 0;
    font-size: 12.5px;
    color: var(--warn);
  }
  .content {
    padding: 8px 40px 40px;
  }
  section {
    margin-top: 26px;
  }
  .head {
    display: flex;
    align-items: baseline;
    gap: 12px;
    margin-bottom: 12px;
  }
  .more {
    margin-left: auto;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(290px, 1fr));
    gap: 10px;
  }
  .placeholder {
    height: 136px;
    border-radius: var(--radius-lg);
    background: var(--surface);
    border: 1px solid var(--line);
  }
  .recents {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(290px, 1fr));
    gap: 10px;
  }
  .recent {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    text-align: left;
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: var(--radius-lg);
    transition: border-color 0.12s;
  }
  .recent:hover:not(:disabled) {
    border-color: #3a3f4a;
  }
  .r-main {
    flex: 1;
    min-width: 0;
  }
  .r-name {
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .r-sub {
    font-size: 12px;
    color: var(--muted);
    margin-top: 2px;
  }
  .r-players {
    font-size: 12.5px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .r-players.off {
    color: var(--muted);
  }
</style>
