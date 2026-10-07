<script lang="ts">
  import HeroArt from "./HeroArt.svelte";
  import Icon from "./Icon.svelte";
  import ServerCard from "./ServerCard.svelte";
  import BeamText from "./BeamText.svelte";
  import { store } from "../lib/store.svelte";
  import { ago, mapName, serverKey } from "../lib/beam";
  import type { Server } from "../lib/types";

  const spotlight = $derived(
    store.servers
      .filter((s) => (s.featured || s.partner || s.official) && s.players > 0)
      .slice(0, 8),
  );
  const busiest = $derived(
    store.servers.filter((s) => !s.featured && !s.partner && !s.official && !s.password).slice(0, 8),
  );
  const byKey = $derived(new Map(store.servers.map((s) => [serverKey(s), s] as [string, Server])));
  const recents = $derived((store.view?.settings.recents ?? []).slice(0, 4));
  const running = $derived(store.view?.launcher_running ?? false);
  const account = $derived(store.view?.account);
</script>

<div class="home fade-in">
  <section class="hero">
    <HeroArt />
    <div class="hero-shade"></div>
    <div class="hero-body">
      <div class="eyebrow">BeamNG.drive · Multiplayer</div>
      <h1 class="display">Beam<span>Link</span></h1>
      <p class="tag">Every BeamMP server, one click away. Find a lobby, hit join, drive.</p>
      <div class="cta">
        {#if store.canPlay}
          <button class="btn btn-primary big" onclick={() => store.play()}>
            <Icon name="play" size={22} />
            {running ? "BeamMP running" : "Play BeamMP"}
          </button>
        {:else}
          <button class="btn btn-primary big" disabled title="BeamNG.drive runs on Windows and Linux">
            <Icon name="play" size={22} /> Play BeamMP
          </button>
        {/if}
        <button class="btn btn-ghost big" onclick={() => (store.page = "servers")}>
          <Icon name="servers" size={20} /> Browse servers
        </button>
      </div>
      {#if store.inSession}
        <div class="session"><span class="dot good"></span> Driving on <BeamText text={store.inSession.name ?? "a server"} /></div>
      {/if}
      {#if !store.canPlay}
        <div class="notice">
          <Icon name="alert" size={16} /> BeamNG.drive doesn't run on {store.view?.platform === "macos" ? "macOS" : "this system"}.
          Browse servers and build your favorites here, then play from a Windows or Linux PC.
        </div>
      {/if}
    </div>
    <div class="stats">
      <div class="stat">
        <div class="num display">{store.totals.players.toLocaleString()}</div>
        <div class="eyebrow">Drivers online</div>
      </div>
      <div class="stat">
        <div class="num display">{store.totals.servers.toLocaleString()}</div>
        <div class="eyebrow">Servers</div>
      </div>
      <div class="stat">
        <div class="num display acct">{account?.signed_in ? account.username : "Guest"}</div>
        <div class="eyebrow">{account?.signed_in ? account.role ?? "Signed in" : "Not signed in"}</div>
      </div>
    </div>
  </section>

  {#if recents.length}
    <section class="block">
      <div class="head"><h2 class="display">Jump back in</h2></div>
      <div class="recents">
        {#each recents as r}
          {@const live = byKey.get(serverKey(r))}
          <button class="recent panel" onclick={() => store.join(r)}>
            <div class="r-main">
              <div class="r-name"><BeamText text={r.name} /></div>
              <div class="muted r-sub">{mapName(r.map)} · {ago(r.at)}</div>
            </div>
            <div class="r-side">
              {#if live}<span class="r-players">{live.players}/{live.max_players}</span>{:else}<span class="muted">offline</span>{/if}
              <span class="r-join"><Icon name="play" size={14} /></span>
            </div>
          </button>
        {/each}
      </div>
    </section>
  {/if}

  <section class="block">
    <div class="head">
      <h2 class="display">Spotlight</h2>
      <span class="muted">Featured, partner and official servers</span>
      <button class="btn btn-ghost btn-sm more" onclick={() => (store.page = "servers")}>All servers <Icon name="chevron" size={14} /></button>
    </div>
    {#if store.loadingServers && !store.servers.length}
      <div class="grid">{#each Array(4) as _}<div class="skeleton panel"></div>{/each}</div>
    {:else if spotlight.length}
      <div class="grid">{#each spotlight as s (serverKey(s))}<ServerCard server={s} />{/each}</div>
    {:else}
      <div class="empty muted">{store.serversError ?? "Nothing in the spotlight right now."}</div>
    {/if}
  </section>

  <section class="block">
    <div class="head"><h2 class="display">Busiest community servers</h2></div>
    <div class="grid">{#each busiest as s (serverKey(s))}<ServerCard server={s} />{/each}</div>
  </section>
</div>

<style>
  .home {
    height: 100%;
    overflow-y: auto;
    padding-bottom: 40px;
  }
  .hero {
    position: relative;
    height: 430px;
    overflow: hidden;
    border-bottom: 1px solid var(--line);
  }
  .hero-shade {
    position: absolute;
    inset: 0;
    background:
      linear-gradient(90deg, rgba(7, 7, 11, 0.92) 0%, rgba(7, 7, 11, 0.55) 45%, rgba(7, 7, 11, 0) 75%),
      linear-gradient(0deg, var(--bg) 0%, rgba(7, 7, 11, 0) 40%);
  }
  .hero-body {
    position: relative;
    padding: 56px 48px 0;
    max-width: 720px;
  }
  h1 {
    margin: 6px 0 6px;
    font-size: 104px;
    line-height: 0.88;
    text-shadow: 0 10px 40px rgba(0, 0, 0, 0.6);
  }
  h1 span {
    background: var(--grad);
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
    padding-right: 0.06em;
  }
  .tag {
    font-size: 17px;
    color: var(--text-2);
    margin: 10px 0 26px;
  }
  .cta {
    display: flex;
    gap: 12px;
  }
  .big {
    height: 54px;
    padding: 0 28px;
    font-size: 20px;
    border-radius: 12px;
  }
  .session,
  .notice {
    margin-top: 18px;
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text-2);
  }
  .notice {
    max-width: 560px;
    padding: 10px 14px;
    border-radius: 10px;
    background: rgba(255, 176, 32, 0.08);
    border: 1px solid rgba(255, 176, 32, 0.25);
    color: #ffd38a;
    font-size: 13px;
  }
  .stats {
    position: absolute;
    right: 40px;
    bottom: 34px;
    display: flex;
    gap: 14px;
  }
  .stat {
    min-width: 150px;
    padding: 14px 18px;
    border-radius: 14px;
    background: rgba(10, 10, 16, 0.6);
    border: 1px solid var(--line-hi);
    backdrop-filter: blur(10px);
  }
  .num {
    font-size: 36px;
    line-height: 1;
    margin-bottom: 4px;
  }
  .acct {
    font-size: 26px;
    max-width: 180px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    line-height: 1.38;
  }
  .block {
    padding: 26px 48px 0;
  }
  .head {
    display: flex;
    align-items: baseline;
    gap: 14px;
    margin-bottom: 14px;
  }
  h2 {
    margin: 0;
    font-size: 28px;
  }
  .more {
    margin-left: auto;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 14px;
  }
  .skeleton {
    height: 168px;
    background: linear-gradient(90deg, rgba(255, 255, 255, 0.03), rgba(255, 255, 255, 0.07), rgba(255, 255, 255, 0.03));
    background-size: 200% 100%;
    animation: shimmer 1.3s infinite;
  }
  @keyframes shimmer {
    to {
      background-position: -200% 0;
    }
  }
  .recents {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 12px;
  }
  .recent {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px 16px;
    text-align: left;
    transition: border-color 0.15s, transform 0.15s;
  }
  .recent:hover {
    border-color: rgba(255, 46, 99, 0.45);
    transform: translateY(-2px);
  }
  .r-main {
    flex: 1;
    min-width: 0;
  }
  .r-name {
    font-weight: 700;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .r-sub {
    font-size: 12px;
    margin-top: 3px;
  }
  .r-side {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .r-players {
    font-weight: 700;
    color: var(--good);
  }
  .r-join {
    width: 30px;
    height: 30px;
    border-radius: 9px;
    display: grid;
    place-items: center;
    background: var(--grad);
  }
  .empty {
    padding: 24px 0;
  }
</style>
