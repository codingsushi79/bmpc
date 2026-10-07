<script lang="ts">
  import BeamText from "./BeamText.svelte";
  import Icon from "./Icon.svelte";
  import { store } from "../lib/store.svelte";
  import { bytes, flag, mapHue, mapName, serverKey } from "../lib/beam";

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
    <div class="banner" style:--h={mapHue(s.map)}>
      <button class="icon-btn close" onclick={() => (store.selected = null)} aria-label="Close"><Icon name="close" /></button>
      <div class="badges">
        {#if s.partner}<span class="badge partner">Partner</span>{/if}
        {#if s.featured}<span class="badge featured">Featured</span>{/if}
        {#if s.official}<span class="badge official">Official</span>{/if}
        {#if s.password}<span class="badge"><Icon name="lock" size={11} /> Password</span>{/if}
        {#if !s.guests}<span class="badge">Accounts only</span>{/if}
      </div>
      <div class="map display">{mapName(s.map)}</div>
    </div>

    <div class="content">
      <h2><BeamText text={s.name} /></h2>
      <div class="owner muted">hosted by {s.owner || "unknown"} · {flag(s.location)} {s.location || "—"}</div>

      <div class="actions">
        <button class="btn btn-primary join" disabled={!store.canPlay} onclick={() => store.join(s)}>
          <Icon name="play" size={18} /> Join server
        </button>
        <button class="btn btn-ghost" class:faved={fav} onclick={() => store.toggleFavorite(s)} title="Favorite">
          <Icon name={fav ? "star-fill" : "star"} size={17} />
        </button>
        <button class="btn btn-ghost" onclick={copy} title="Copy address"><Icon name="copy" size={17} /></button>
      </div>

      <div class="facts">
        <div class="fact">
          <div class="eyebrow">Players</div>
          <div class="v"><b>{s.players}</b><span class="muted"> / {s.max_players}</span></div>
        </div>
        <div class="fact">
          <div class="eyebrow">Ping</div>
          <div class="v">
            {#if ping == null || ping < 0}<span class="muted">{ping === null ? "n/a" : "…"}</span>{:else}{ping} ms{/if}
          </div>
        </div>
        <div class="fact">
          <div class="eyebrow">Mods</div>
          <div class="v" class:warn={s.mods_size > 500 * 1024 * 1024}>{s.mods.length ? bytes(s.mods_size) : "None"}</div>
        </div>
        <div class="fact">
          <div class="eyebrow">Version</div>
          <div class="v">{s.version || "?"}</div>
        </div>
      </div>

      {#if s.description}
        <section>
          <div class="eyebrow">About</div>
          <p class="desc"><BeamText text={s.description} multiline /></p>
        </section>
      {/if}

      {#if s.tags.length}
        <section>
          <div class="eyebrow">Tags</div>
          <div class="tags">{#each s.tags as t}<span class="chip">{t}</span>{/each}</div>
        </section>
      {/if}

      <section>
        <div class="eyebrow">Online now ({s.player_names.length})</div>
        {#if s.player_names.length}
          <div class="people">{#each s.player_names as p}<span class="person" class:guest={p.startsWith("guest")}>{p}</span>{/each}</div>
        {:else}
          <div class="muted">Nobody yet — be the first.</div>
        {/if}
      </section>

      {#if s.mods.length}
        <section>
          <div class="eyebrow">Mods you'll download ({s.mods.length})</div>
          {#if s.mods_size > 500 * 1024 * 1024}
            <div class="warnbox"><Icon name="alert" size={15} /> {bytes(s.mods_size)} to download on first join. BeamMP asks before it downloads.</div>
          {/if}
          <ul class="mods mono">
            {#each showAllMods ? s.mods : s.mods.slice(0, 8) as m}<li>{m}</li>{/each}
          </ul>
          {#if s.mods.length > 8 && !showAllMods}
            <button class="btn btn-ghost btn-sm" onclick={() => (showAllMods = true)}>Show all {s.mods.length}</button>
          {/if}
        </section>
      {/if}

      <section>
        <div class="eyebrow">Address</div>
        <div class="mono addr">{s.ip}:{s.port}</div>
      </section>
    </div>
  </aside>
{/if}

<style>
  .scrim {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.35);
    z-index: 20;
    animation: fade 0.2s ease both;
  }
  @keyframes fade {
    from {
      opacity: 0;
    }
  }
  .drawer {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    width: 440px;
    max-width: 92%;
    z-index: 21;
    background: var(--panel-solid);
    border-left: 1px solid var(--line-hi);
    box-shadow: -30px 0 60px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    animation: slide 0.22s cubic-bezier(0.2, 0.8, 0.2, 1) both;
  }
  @keyframes slide {
    from {
      transform: translateX(40px);
      opacity: 0;
    }
  }
  .banner {
    position: relative;
    height: 150px;
    flex: none;
    padding: 16px 20px;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    background:
      radial-gradient(120% 140% at 100% 0%, hsla(var(--h), 90%, 55%, 0.6), transparent 60%),
      linear-gradient(120deg, hsla(var(--h), 60%, 18%, 1), hsla(calc(var(--h) + 40), 70%, 9%, 1));
  }
  .banner::after {
    content: "";
    position: absolute;
    inset: 0;
    background: linear-gradient(0deg, var(--panel-solid), transparent 70%),
      repeating-linear-gradient(115deg, rgba(255, 255, 255, 0.04) 0 2px, transparent 2px 14px);
  }
  .close {
    position: absolute;
    top: 12px;
    right: 12px;
    z-index: 2;
    background: rgba(0, 0, 0, 0.35);
  }
  .badges {
    position: absolute;
    top: 16px;
    left: 20px;
    display: flex;
    gap: 6px;
    z-index: 2;
  }
  .map {
    position: relative;
    z-index: 2;
    font-size: 34px;
    line-height: 1;
  }
  .content {
    flex: 1;
    overflow-y: auto;
    padding: 6px 22px 26px;
  }
  h2 {
    margin: 4px 0 4px;
    font-size: 20px;
    line-height: 1.25;
    user-select: text;
  }
  .owner {
    font-size: 12.5px;
  }
  .actions {
    display: flex;
    gap: 8px;
    margin: 16px 0 18px;
  }
  .join {
    flex: 1;
    height: 46px;
    font-size: 18px;
  }
  .actions .btn-ghost {
    height: 46px;
    width: 46px;
    padding: 0;
  }
  .faved {
    color: var(--warn);
  }
  .facts {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
    margin-bottom: 6px;
  }
  .fact {
    padding: 10px 12px;
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--line);
  }
  .fact .eyebrow {
    font-size: 10.5px;
  }
  .v {
    font-weight: 700;
    margin-top: 4px;
    font-size: 15px;
  }
  .v b {
    color: var(--good);
  }
  .warn {
    color: var(--warn);
  }
  section {
    margin-top: 18px;
  }
  section > .eyebrow {
    margin-bottom: 8px;
  }
  .desc {
    margin: 0;
    line-height: 1.55;
    color: var(--text-2);
    user-select: text;
  }
  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .people {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .person {
    padding: 4px 9px;
    border-radius: 7px;
    background: rgba(61, 220, 132, 0.1);
    color: #a6f4c5;
    font-size: 12.5px;
    font-weight: 600;
    user-select: text;
  }
  .person.guest {
    background: rgba(255, 255, 255, 0.05);
    color: var(--muted);
  }
  .warnbox {
    display: flex;
    gap: 8px;
    align-items: center;
    padding: 9px 12px;
    border-radius: 9px;
    background: rgba(255, 176, 32, 0.08);
    border: 1px solid rgba(255, 176, 32, 0.25);
    color: #ffd38a;
    font-size: 12.5px;
    margin-bottom: 8px;
  }
  .mods {
    margin: 0 0 10px;
    padding: 0;
    list-style: none;
    color: var(--text-2);
  }
  .mods li {
    padding: 4px 0;
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
