<script lang="ts">
  import BeamText from "./BeamText.svelte";
  import Icon from "./Icon.svelte";
  import { store } from "../lib/store.svelte";
  import { ago, mapName, serverKey } from "../lib/beam";
  import type { SavedServer, Server } from "../lib/types";

  const live = $derived(new Map(store.servers.map((s) => [serverKey(s), s] as [string, Server])));
  const favorites = $derived(store.view?.settings.favorites ?? []);
  const recents = $derived(store.view?.settings.recents ?? []);
  const onlineFavs = $derived(favorites.filter((f) => (live.get(serverKey(f))?.players ?? 0) > 0).length);

  function open(saved: SavedServer) {
    const s = live.get(serverKey(saved));
    if (s) store.selected = s;
    else store.toast("That server is not on the public list right now", "info");
  }
</script>

{#snippet list(items: SavedServer[], empty: string, kind: "fav" | "recent")}
  {#if items.length}
    <div class="list">
      {#each items as item (serverKey(item))}
        {@const s = live.get(serverKey(item))}
        <div class="item panel" role="button" tabindex="0" onclick={() => open(item)} onkeydown={(e) => e.key === "Enter" && open(item)}>
          <span class="dot" class:good={!!s && s.players > 0} class:warn={!!s && s.players === 0}></span>
          <div class="main">
            <div class="name"><BeamText text={s?.name ?? item.name} /></div>
            <div class="sub muted">
              {mapName(s?.map ?? item.map)} · {kind === "fav" ? `added ${ago(item.at)}` : `played ${ago(item.at)}`}
              {#if !s} · <span class="off">not listed</span>{/if}
            </div>
          </div>
          <div class="players">
            {#if s}<b>{s.players}</b><span class="muted">/{s.max_players}</span>{:else}<span class="muted">—</span>{/if}
          </div>
          {#if kind === "fav"}
            <button class="icon-btn star" title="Unfavorite" onclick={(e) => { e.stopPropagation(); store.toggleFavorite(item); }}><Icon name="star-fill" size={16} /></button>
          {/if}
          <button class="btn btn-primary btn-sm" disabled={!store.canPlay} onclick={(e) => { e.stopPropagation(); store.join(s ?? item); }}>Join</button>
        </div>
      {/each}
    </div>
  {:else}
    <div class="empty muted">{empty}</div>
  {/if}
{/snippet}

<div class="library">
  <h1 class="page-title">Library</h1>
  <div class="cols">
    <section>
      <div class="head">
        <h2 class="section-title">Favorites</h2>
        <span class="muted">{favorites.length} saved · {onlineFavs} with drivers now</span>
      </div>
      {@render list(favorites, "Star a server in the browser and it lands here. Favorites also show up in BeamMP's in-game list.", "fav")}
    </section>
    <section>
      <div class="head"><h2 class="section-title">Recently played</h2></div>
      {@render list(recents, "Servers you join from BeamLink show up here.", "recent")}
    </section>
  </div>
</div>

<style>
  .library {
    height: 100%;
    overflow-y: auto;
    padding: 26px 32px 30px;
  }
  h1 {
    margin: 0 0 18px;
  }
  .cols {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 26px;
  }
  .head {
    display: flex;
    align-items: baseline;
    gap: 12px;
    margin-bottom: 12px;
  }
  h2 {
    margin: 0;
    font-size: 15px;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    cursor: pointer;
    transition: border-color 0.15s;
  }
  .item:hover {
    border-color: #3a3f4a;
  }
  .main {
    flex: 1;
    min-width: 0;
  }
  .name {
    font-weight: 650;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .sub {
    font-size: 12px;
    margin-top: 3px;
  }
  .off {
    color: var(--warn);
  }
  .players {
    font-weight: 700;
    min-width: 54px;
    text-align: right;
  }
  .players b {
    color: var(--good);
  }
  .star {
    color: var(--warn);
  }
  .empty {
    padding: 22px;
    border: 1px dashed var(--line-hi);
    border-radius: var(--radius);
    line-height: 1.5;
  }
</style>
