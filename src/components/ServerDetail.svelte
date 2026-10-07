<script lang="ts">
  import BeamText from "./BeamText.svelte";
  import Icon from "./Icon.svelte";
  import { store } from "../lib/store.svelte";
  import { bytes, mapName, serverKey } from "../lib/beam";

  const s = $derived(store.selected);
  const key = $derived(s ? serverKey(s) : "");
  const ping = $derived(s ? store.pings[key] : undefined);
  const fav = $derived(s ? store.favoriteKeys.has(key) : false);
  let showAllMods = $state(false);

  $effect(() => {
    // One quick ping for the server being looked at.
    if (s) {
      showAllMods = false;
      store.ping([serverKey(s)]);
    }
  });

  async function copy() {
    if (!s) return;
    try {
      await navigator.clipboard.writeText(`${s.ip}:${s.port}`);
      store.toast("Address copied", "info");
    } catch {
      store.toast(`${s.ip}:${s.port}`, "info");
    }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") store.selected = null;
  }
</script>

<svelte:window onkeydown={onKey} />

{#if s}
  <div class="scrim" role="presentation" onclick={() => (store.selected = null)}></div>
  <aside class="drawer">
    <header>
      <div class="crumbs">
        <span>{mapName(s.map)}</span>
        <span class="region">{s.location || "--"}</span>
        {#if s.official}<span class="tag official">Official</span>{/if}
        {#if s.featured}<span class="tag featured">Featured</span>{/if}
        {#if s.partner}<span class="tag partner">Partner</span>{/if}
      </div>
      <button class="icon-btn" onclick={() => (store.selected = null)} aria-label="Close"><Icon name="close" size={16} /></button>
    </header>

    <div class="content">
      <h2><BeamText text={s.name} /></h2>
      <div class="owner">Hosted by {s.owner || "unknown"}</div>
      {#if s.password || !s.guests}
        <div class="notes">
          {#if s.password}<span><Icon name="lock" size={12} /> Password required</span>{/if}
          {#if !s.guests}<span>BeamMP account required (no guests)</span>{/if}
        </div>
      {/if}

      <div class="actions">
        <button class="btn btn-primary btn-lg join" disabled={!store.canPlay} onclick={() => store.join(s)}>
          <Icon name="play" size={15} /> Join server
        </button>
        <button class="btn btn-secondary btn-lg square" class:faved={fav} onclick={() => store.toggleFavorite(s)} title={fav ? "Remove from favorites" : "Add to favorites"}>
          <Icon name={fav ? "star-fill" : "star"} size={16} />
        </button>
        <button class="btn btn-secondary btn-lg square" onclick={copy} title="Copy address"><Icon name="copy" size={16} /></button>
      </div>

      <dl class="facts">
        <div><dt>Players</dt><dd>{s.players} <span class="muted">/ {s.max_players}</span></dd></div>
        <div>
          <dt>Ping</dt>
          <dd>{#if ping == null || ping < 0}<span class="muted">{ping === null ? "n/a" : "…"}</span>{:else}{ping} ms{/if}</dd>
        </div>
        <div><dt>Mods</dt><dd class:warn={s.mods_size > 500 * 1024 * 1024}>{s.mods.length ? bytes(s.mods_size) : "None"}</dd></div>
        <div><dt>Version</dt><dd>{s.version || "?"}</dd></div>
      </dl>

      {#if s.description}
        <section>
          <h3 class="label">About</h3>
          <p class="desc"><BeamText text={s.description} multiline /></p>
        </section>
      {/if}

      {#if s.tags.length}
        <section>
          <h3 class="label">Tags</h3>
          <div class="list-inline">{#each s.tags as t}<span class="tag">{t}</span>{/each}</div>
        </section>
      {/if}

      <section>
        <h3 class="label">Online now · {s.player_names.length}</h3>
        {#if s.player_names.length}
          <div class="list-inline">{#each s.player_names as p}<span class="person" class:guest={p.startsWith("guest")}>{p}</span>{/each}</div>
        {:else}
          <p class="muted">Nobody yet.</p>
        {/if}
      </section>

      {#if s.mods.length}
        <section>
          <h3 class="label">Mods to download · {s.mods.length}</h3>
          {#if s.mods_size > 500 * 1024 * 1024}
            <p class="warnline">{bytes(s.mods_size)} downloads on your first join. BeamMP asks before downloading.</p>
          {/if}
          <ul class="mods mono">
            {#each showAllMods ? s.mods : s.mods.slice(0, 8) as m}<li>{m}</li>{/each}
          </ul>
          {#if s.mods.length > 8 && !showAllMods}
            <button class="btn btn-quiet btn-sm" onclick={() => (showAllMods = true)}>Show all {s.mods.length}</button>
          {/if}
        </section>
      {/if}

      <section>
        <h3 class="label">Address</h3>
        <div class="mono addr">{s.ip}:{s.port}</div>
      </section>
    </div>
  </aside>
{/if}

<style>
  .scrim {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    z-index: 20;
  }
  .drawer {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    width: 420px;
    max-width: 92%;
    z-index: 21;
    background: var(--surface);
    border-left: 1px solid var(--line-hi);
    display: flex;
    flex-direction: column;
    animation: slide 0.16s ease-out both;
  }
  @keyframes slide {
    from {
      transform: translateX(16px);
      opacity: 0;
    }
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 48px;
    padding: 0 10px 0 20px;
    border-bottom: 1px solid var(--line);
    flex: none;
  }
  .crumbs {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text-2);
  }
  .content {
    flex: 1;
    overflow-y: auto;
    padding: 18px 20px 28px;
  }
  h2 {
    margin: 0 0 4px;
    font-size: 18px;
    line-height: 1.3;
    font-weight: 650;
    user-select: text;
  }
  .owner {
    font-size: 12.5px;
    color: var(--muted);
  }
  .notes {
    display: flex;
    flex-direction: column;
    gap: 3px;
    margin-top: 8px;
    font-size: 12.5px;
    color: var(--warn);
  }
  .notes span {
    display: inline-flex;
    align-items: center;
    gap: 5px;
  }
  .actions {
    display: flex;
    gap: 6px;
    margin: 16px 0;
  }
  .join {
    flex: 1;
  }
  .square {
    width: 42px;
    padding: 0;
  }
  .faved {
    color: var(--warn);
  }
  .facts {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    margin: 0;
    border: 1px solid var(--line);
    border-radius: var(--radius-lg);
  }
  .facts div {
    padding: 10px 12px;
    border-right: 1px solid var(--line);
  }
  .facts div:last-child {
    border-right: none;
  }
  dt {
    font-size: 11.5px;
    color: var(--muted);
  }
  dd {
    margin: 3px 0 0;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  dd.warn {
    color: var(--warn);
  }
  section {
    margin-top: 20px;
  }
  h3 {
    margin: 0 0 8px;
  }
  p {
    margin: 0;
  }
  .desc {
    line-height: 1.55;
    color: var(--text-2);
    user-select: text;
  }
  .list-inline {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
  }
  .person {
    padding: 2px 7px;
    border-radius: 4px;
    background: var(--surface-3);
    font-size: 12px;
    user-select: text;
  }
  .person.guest {
    color: var(--muted);
  }
  .warnline {
    font-size: 12.5px;
    color: var(--warn);
    margin-bottom: 8px;
  }
  .mods {
    margin: 0 0 8px;
    padding: 0;
    list-style: none;
    color: var(--text-2);
  }
  .mods li {
    padding: 5px 0;
    border-bottom: 1px solid var(--line);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .addr {
    user-select: text;
    color: var(--text-2);
  }
</style>
