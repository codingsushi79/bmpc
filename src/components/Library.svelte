<script lang="ts">
  import Icon from "./Icon.svelte";
  import ServerRow from "./ServerRow.svelte";
  import { store } from "../lib/store.svelte";
  import { ago, serverKey } from "../lib/beam";
  import type { SavedServer, Server } from "../lib/types";

  const live = $derived(new Map(store.servers.map((s) => [serverKey(s), s] as [string, Server])));
  const favorites = $derived(store.view?.settings.favorites ?? []);
  const recents = $derived(store.view?.settings.recents ?? []);

  function open(saved: SavedServer) {
    const s = live.get(serverKey(saved));
    if (s) store.selected = s;
  }
</script>

{#snippet list(items: SavedServer[], empty: string, kind: "fav" | "recent")}
  {#if items.length}
    <div class="panel">
      {#each items as item (serverKey(item))}
        {@const s = live.get(serverKey(item)) ?? null}
        <ServerRow
          name={item.name}
          server={s}
          detail={kind === "fav" ? `Added ${ago(item.at)}` : `Played ${ago(item.at)}`}
          offline={!s && store.servers.length > 0}
          onopen={s ? () => open(item) : undefined}
        >
          {#snippet actions()}
            {#if kind === "fav"}
              <button class="icon-btn star" title="Remove from favorites" aria-label="Remove from favorites" onclick={(e) => { e.stopPropagation(); store.toggleFavorite(item); }}>
                <Icon name="star-fill" size={14} />
              </button>
            {/if}
            <button class="btn btn-secondary btn-sm" disabled={!store.canPlay} onclick={(e) => { e.stopPropagation(); store.join(s ?? item); }}>Join</button>
          {/snippet}
        </ServerRow>
      {/each}
    </div>
  {:else}
    <p class="muted empty">{empty}</p>
  {/if}
{/snippet}

<div class="page">
  <h1 class="page-title">Library</h1>
  <section>
    <h2 class="section-title">Favorites <span class="muted">{favorites.length}</span></h2>
    {@render list(favorites, "Star a server in the browser to add it here. Favorites are also copied to BeamMP's in-game list.", "fav")}
  </section>
  <section>
    <h2 class="section-title">Recently played <span class="muted">{recents.length}</span></h2>
    {@render list(recents, "Servers you join from BeamLink appear here.", "recent")}
  </section>
  <p class="muted note">Private and LAN servers live under <button class="link" onclick={() => (store.page = "direct")}>Direct connect</button>.</p>
</div>

<style>
  .page {
    height: 100%;
    overflow-y: auto;
    padding: 24px 28px 40px;
    max-width: 920px;
  }
  section {
    margin-top: 20px;
  }
  h2 {
    margin-bottom: 10px;
  }
  h2 .muted {
    font-weight: 500;
    margin-left: 4px;
  }
  .star {
    color: var(--warn);
  }
  .empty {
    margin: 0;
  }
  .note {
    margin-top: 24px;
    font-size: 12.5px;
  }
  .link {
    color: var(--text-2);
    text-decoration: underline;
  }
</style>
