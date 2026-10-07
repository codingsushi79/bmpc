<script lang="ts">
  import BeamText from "./BeamText.svelte";
  import Icon from "./Icon.svelte";
  import { store } from "../lib/store.svelte";
  import { REGIONS, bytes, mapName, plain, region, serverKey } from "../lib/beam";
  import type { Server } from "../lib/types";

  type Sort = "players" | "name" | "map" | "mods" | "ping";

  let query = $state("");
  let hideEmpty = $state(false);
  let hideFull = $state(false);
  let noMods = $state(false);
  let officialOnly = $state(false);
  let spotlightOnly = $state(false);
  let hideLocked = $state(true);
  let favoritesOnly = $state(false);
  let map = $state("");
  let continent = $state("");
  let sort = $state<Sort>("players");
  let desc = $state(true);

  // Lower-cased search text per server, built once per list fetch rather
  // than on every keystroke.
  const haystacks = $derived.by(() => {
    const m = new Map<Server, string>();
    for (const s of store.servers) {
      m.set(
        s,
        `${plain(s.name)} ${s.map} ${mapName(s.map)} ${s.owner} ${s.tags.join(" ")} ${s.player_names.join(" ")} ${s.ip}`.toLowerCase(),
      );
    }
    return m;
  });

  const maps = $derived.by(() => {
    const counts = new Map<string, number>();
    for (const s of store.servers) counts.set(s.map, (counts.get(s.map) ?? 0) + 1);
    return [...counts.entries()].sort((a, b) => b[1] - a[1]).slice(0, 40);
  });

  const filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    const terms = q ? q.split(/\s+/) : [];
    const favs = store.favoriteKeys;
    const list = store.servers.filter((s) => {
      if (hideEmpty && s.players === 0) return false;
      if (hideFull && s.max_players > 0 && s.players >= s.max_players) return false;
      if (noMods && s.mods.length > 0) return false;
      if (officialOnly && !s.official) return false;
      if (spotlightOnly && !(s.featured || s.partner)) return false;
      if (hideLocked && s.password) return false;
      if (favoritesOnly && !favs.has(serverKey(s))) return false;
      if (map && s.map !== map) return false;
      if (continent && region(s.location) !== continent) return false;
      if (terms.length) {
        const hay = haystacks.get(s) ?? "";
        for (const t of terms) if (!hay.includes(t)) return false;
      }
      return true;
    });
    const dir = desc ? -1 : 1;
    const pings = store.pings;
    const ping = (s: Server) => {
      const p = pings[serverKey(s)];
      return p == null || p < 0 ? 1e9 : p;
    };
    const key: Record<Sort, (a: Server, b: Server) => number> = {
      players: (a, b) => (a.players - b.players) * dir || (a.max_players - b.max_players) * dir,
      name: (a, b) => plain(a.name).localeCompare(plain(b.name)) * dir,
      map: (a, b) => mapName(a.map).localeCompare(mapName(b.map)) * dir,
      mods: (a, b) => (a.mods_size - b.mods_size) * dir,
      // Unmeasured servers always sink, whichever way the sort goes.
      ping: (a, b) => (ping(a) - ping(b)) * -dir || b.players - a.players,
    };
    return list.sort(key[sort]);
  });

  /** Which player matched the search, so "find my friend" shows the hit. */
  function playerHit(s: Server): string | null {
    const q = query.trim().toLowerCase();
    if (q.length < 3) return null;
    return s.player_names.find((p) => p.toLowerCase().includes(q)) ?? null;
  }

  function setSort(next: Sort) {
    if (sort === next) desc = !desc;
    else {
      sort = next;
      desc = next === "players" || next === "mods";
      if (next === "ping") desc = false;
    }
  }

  // ------------------------------------------------- virtual scrolling ---
  const ROW = 64;
  let viewport = $state<HTMLDivElement>();
  let scrollTop = $state(0);
  let height = $state(600);
  const first = $derived(Math.max(0, Math.floor(scrollTop / ROW) - 6));
  const last = $derived(Math.min(filtered.length, Math.ceil((scrollTop + height) / ROW) + 6));
  const visible = $derived(filtered.slice(first, last));

  $effect(() => {
    // Back to the top when the filters change what is shown.
    void query, map, continent, hideEmpty, hideFull, noMods, officialOnly, spotlightOnly, hideLocked, favoritesOnly;
    if (viewport) viewport.scrollTop = 0;
  });

  function pingVisible() {
    store.ping(visible.map(serverKey));
  }

  function resetFilters() {
    query = "";
    hideEmpty = hideFull = noMods = officialOnly = spotlightOnly = favoritesOnly = false;
    hideLocked = true;
    map = continent = "";
  }

  const shownPlayers = $derived(filtered.reduce((n, s) => n + s.players, 0));
</script>

<div class="browser">
  <header>
    <div class="title">
      <h1 class="page-title">Servers</h1>
      <span class="muted">{filtered.length.toLocaleString()} servers · {shownPlayers.toLocaleString()} drivers</span>
    </div>
    <div class="actions">
      <button class="btn btn-secondary btn-sm" onclick={() => (store.page = "direct")}><Icon name="link" size={15} /> Direct connect</button>
      <button class="btn btn-secondary btn-sm" onclick={pingVisible} title="Measure ping to the servers on screen"><Icon name="signal" size={15} /> Ping</button>
      <button class="icon-btn" onclick={() => store.loadServers(true)} title="Refresh list">
        <span class:spin={store.loadingServers}><Icon name="refresh" size={18} /></span>
      </button>
    </div>
  </header>

  <div class="filters">
    <label class="search">
      <Icon name="search" size={17} />
      <input placeholder="Search servers, maps, tags — or a player's name" bind:value={query} spellcheck="false" />
      {#if query}<button class="icon-btn clear" onclick={() => (query = "")}><Icon name="close" size={14} /></button>{/if}
    </label>
    <select bind:value={map} title="Map">
      <option value="">All maps</option>
      {#each maps as [m, n]}<option value={m}>{mapName(m)} ({n})</option>{/each}
    </select>
    <select bind:value={continent} title="Region">
      <option value="">All regions</option>
      {#each Object.entries(REGIONS) as [code, label]}<option value={code}>{label}</option>{/each}
    </select>
  </div>
  <div class="toggles">
    <button class="toggle" class:on={hideEmpty} onclick={() => (hideEmpty = !hideEmpty)}>Has players</button>
    <button class="toggle" class:on={hideFull} onclick={() => (hideFull = !hideFull)}>Not full</button>
    <button class="toggle" class:on={noMods} onclick={() => (noMods = !noMods)}>No mods</button>
    <button class="toggle" class:on={officialOnly} onclick={() => (officialOnly = !officialOnly)}>Official</button>
    <button class="toggle" class:on={spotlightOnly} onclick={() => (spotlightOnly = !spotlightOnly)}>Featured & partners</button>
    <button class="toggle" class:on={favoritesOnly} onclick={() => (favoritesOnly = !favoritesOnly)}><Icon name="star" size={13} /> Favorites</button>
    <button class="toggle" class:on={hideLocked} onclick={() => (hideLocked = !hideLocked)}><Icon name="lock" size={12} /> Hide locked</button>
    <button class="btn btn-quiet btn-sm reset" onclick={resetFilters}>Reset</button>
  </div>

  <div class="table panel">
    <div class="thead">
      <span></span>
      <button class:on={sort === "name"} onclick={() => setSort("name")}>Server {sort === "name" ? (desc ? "↓" : "↑") : ""}</button>
      <button class:on={sort === "map"} onclick={() => setSort("map")}>Map {sort === "map" ? (desc ? "↓" : "↑") : ""}</button>
      <button class:on={sort === "players"} onclick={() => setSort("players")}>Players {sort === "players" ? (desc ? "↓" : "↑") : ""}</button>
      <button class:on={sort === "mods"} onclick={() => setSort("mods")}>Mods {sort === "mods" ? (desc ? "↓" : "↑") : ""}</button>
      <button class:on={sort === "ping"} onclick={() => setSort("ping")}>Ping {sort === "ping" ? (desc ? "↓" : "↑") : ""}</button>
      <span></span>
    </div>
    <div
      class="viewport"
      bind:this={viewport}
      bind:clientHeight={height}
      onscroll={(e) => (scrollTop = (e.currentTarget as HTMLDivElement).scrollTop)}
    >
      {#if store.serversError && !store.servers.length}
        <div class="empty">
          <Icon name="alert" size={28} />
          <div>Couldn't load the server list</div>
          <div class="muted small">{store.serversError}</div>
          <button class="btn btn-secondary btn-sm" onclick={() => store.loadServers(true)}>Try again</button>
        </div>
      {:else if !filtered.length && !store.loadingServers}
        <div class="empty">
          <Icon name="search" size={28} />
          <div>No servers match</div>
          <button class="btn btn-secondary btn-sm" onclick={resetFilters}>Reset filters</button>
        </div>
      {/if}
      <div class="spacer" style:height={`${filtered.length * ROW}px`}>
        <div class="rows" style:transform={`translateY(${first * ROW}px)`}>
          {#each visible as s (serverKey(s))}
            {@const key = serverKey(s)}
            {@const p = store.pings[key]}
            {@const hit = playerHit(s)}
            {@const fav = store.favoriteKeys.has(key)}
            <div class="row" class:sel={store.selected === s} role="button" tabindex="0"
              onclick={() => (store.selected = s)}
              ondblclick={() => store.join(s)}
              onkeydown={(e) => e.key === "Enter" && store.join(s)}>
              <button class="fav" class:on={fav} onclick={(e) => { e.stopPropagation(); store.toggleFavorite(s); }} title={fav ? "Unfavorite" : "Favorite"}>
                <Icon name={fav ? "star-fill" : "star"} size={16} />
              </button>
              <div class="cell-name">
                <div class="name"><BeamText text={s.name} /></div>
                <div class="sub">
                  <span class="region">{s.location || "--"}</span>
                  {#if s.partner}<span class="tag partner">Partner</span>{/if}
                  {#if s.featured}<span class="tag featured">Featured</span>{/if}
                  {#if s.official}<span class="tag official">Official</span>{/if}
                  {#if s.password}<Icon name="lock" size={12} />{/if}
                  {#if hit}<span class="hit"><Icon name="users" size={11} /> {hit}</span>
                  {:else}<span class="muted owner">{s.tags.slice(0, 3).join(" · ") || s.owner}</span>{/if}
                </div>
              </div>
              <div class="cell-map">{mapName(s.map)}</div>
              <div class="cell-players">
                <span class:zero={s.players === 0} class:full={s.players >= s.max_players && s.max_players > 0}>{s.players}</span><span class="muted">/{s.max_players}</span>
                <div class="meter pbar"><div class:hot={s.max_players > 0 && s.players >= s.max_players} style:width={`${s.max_players ? Math.min(100, (s.players / s.max_players) * 100) : 0}%`}></div></div>
              </div>
              <div class="cell-mods" class:heavy={s.mods_size > 500 * 1024 * 1024}>
                {s.mods.length ? bytes(s.mods_size) : "—"}
              </div>
              <div class="cell-ping">
                {#if p === undefined}<span class="muted">·</span>
                {:else if p === -1}<span class="muted">…</span>
                {:else if p === null}<span class="bad">n/a</span>
                {:else}<span class:good={p < 80} class:warn={p >= 80 && p < 160} class:bad={p >= 160}>{p} ms</span>{/if}
              </div>
              <div class="cell-join">
                <button class="btn btn-secondary btn-sm join" disabled={!store.canPlay} onclick={(e) => { e.stopPropagation(); store.join(s); }}>Join</button>
              </div>
            </div>
          {/each}
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .browser {
    height: 100%;
    display: flex;
    flex-direction: column;
    padding: 22px 28px 16px;
    min-height: 0;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
  }
  .title {
    display: flex;
    align-items: baseline;
    gap: 12px;
  }
  .title .muted {
    font-variant-numeric: tabular-nums;
  }
  .actions {
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .filters {
    display: flex;
    gap: 8px;
  }
  .search {
    flex: 1;
    position: relative;
    display: flex;
    align-items: center;
  }
  .search > :global(svg) {
    position: absolute;
    left: 11px;
    color: var(--muted);
    pointer-events: none;
  }
  .search input {
    width: 100%;
    height: 36px;
    padding-left: 36px;
  }
  .search .clear {
    position: absolute;
    right: 3px;
  }
  select {
    height: 36px;
    min-width: 168px;
  }
  .toggles {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin: 10px 0 14px;
  }
  .reset {
    margin-left: auto;
  }
  .table {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .thead,
  .row {
    display: grid;
    grid-template-columns: 40px minmax(260px, 1fr) 170px 110px 90px 76px 76px;
    align-items: center;
    padding: 0 12px 0 6px;
  }
  .thead {
    height: 34px;
    border-bottom: 1px solid var(--line);
    font-size: 11.5px;
    font-weight: 600;
    color: var(--muted);
  }
  .thead button {
    text-align: left;
    font-weight: inherit;
    color: inherit;
  }
  .thead button:hover,
  .thead button.on {
    color: var(--text);
  }
  .viewport {
    flex: 1;
    overflow-y: auto;
    position: relative;
    contain: strict;
  }
  .spacer {
    position: relative;
  }
  .rows {
    position: absolute;
    inset: 0 0 auto 0;
    will-change: transform;
  }
  .row {
    height: 64px;
    border-bottom: 1px solid var(--line);
    cursor: pointer;
    outline: none;
  }
  .row:hover,
  .row:focus-visible {
    background: var(--surface-2);
  }
  .row.sel {
    background: var(--surface-3);
    box-shadow: inset 2px 0 0 var(--accent);
  }
  .row:hover .join:not(:disabled) {
    background: var(--accent);
    border-color: var(--accent);
    color: #fff;
  }
  .fav {
    width: 28px;
    height: 28px;
    display: grid;
    place-items: center;
    border-radius: var(--radius);
    color: #3d424c;
  }
  .fav:hover,
  .fav.on {
    color: var(--warn);
  }
  .cell-name {
    min-width: 0;
    padding-right: 16px;
  }
  .name {
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .sub {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 4px;
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    color: var(--muted);
  }
  .owner {
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .hit {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--info);
    font-weight: 600;
  }
  .cell-map {
    color: var(--text-2);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    padding-right: 10px;
  }
  .cell-players {
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .cell-players .zero {
    color: var(--muted);
  }
  .cell-players .full {
    color: var(--warn);
  }
  .pbar {
    width: 70px;
    margin-top: 5px;
  }
  .cell-mods {
    color: var(--text-2);
    font-size: 12.5px;
    font-variant-numeric: tabular-nums;
  }
  .cell-mods.heavy {
    color: var(--warn);
  }
  .cell-ping {
    font-size: 12.5px;
    font-variant-numeric: tabular-nums;
  }
  .good {
    color: var(--good);
  }
  .warn {
    color: var(--warn);
  }
  .bad {
    color: var(--bad);
  }
  .cell-join {
    text-align: right;
  }
  .empty {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    color: var(--text-2);
    z-index: 1;
  }
  .small {
    font-size: 12px;
    max-width: 480px;
    text-align: center;
  }
</style>
