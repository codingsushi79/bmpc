<script lang="ts">
  import BeamText from "./BeamText.svelte";
  import Icon from "./Icon.svelte";
  import ServerRow from "./ServerRow.svelte";
  import { api } from "../lib/api";
  import { store } from "../lib/store.svelte";
  import { bytes, mapName, parseAddress, plain, serverKey } from "../lib/beam";
  import type { DirectInfo, SavedServer } from "../lib/types";

  let address = $state("");
  let saveName = $state("");
  let looking = $state(false);
  let result = $state<DirectInfo | null>(null);
  let error = $state<string | null>(null);

  type Live = { info?: DirectInfo; error?: string; loading?: boolean };
  let live = $state<Record<string, Live>>({});

  const saved = $derived(store.view?.settings.direct ?? []);
  const resultSaved = $derived(result ? saved.some((s) => serverKey(s) === serverKey(result!.server)) : false);

  async function lookUp(e?: SubmitEvent) {
    e?.preventDefault();
    looking = true;
    error = null;
    result = null;
    try {
      result = await api.queryServer(address);
      saveName = plain(result.server.name);
    } catch (err) {
      error = String(err);
    } finally {
      looking = false;
    }
  }

  async function joinAddress() {
    // Join even without a lookup: the server may block info requests.
    if (result) return store.join(result.server);
    const target = parseAddress(address);
    if (!target) {
      error = "That isn't a valid address. Try 192.168.1.20 or play.example.com:30814";
      return;
    }
    await store.join({ ...target, name: `${target.ip}:${target.port}`, map: "", at: 0 });
  }

  async function save() {
    if (!result) return;
    try {
      await api.saveDirect(`${result.server.ip}:${result.server.port}`, saveName);
      store.toast("Saved to your servers", "info");
      await store.refresh();
      refreshOne(`${result.server.ip}:${result.server.port}`);
    } catch (err) {
      store.toast(String(err), "err");
    }
  }

  async function remove(s: SavedServer) {
    await api.removeDirect(serverKey(s));
    await store.refresh();
  }

  async function refreshOne(key: string) {
    live[key] = { ...live[key], loading: true };
    try {
      live[key] = { info: await api.queryServer(key) };
    } catch (err) {
      live[key] = { error: String(err) };
    }
  }

  // Poll the saved servers while this page is open: they're few, and each
  // query is one tiny request to that server only.
  $effect(() => {
    const keys = saved.map(serverKey);
    for (const key of keys) if (!live[key]) refreshOne(key);
    const timer = setInterval(() => {
      if (!document.hidden) for (const key of keys) refreshOne(key);
    }, 30_000);
    return () => clearInterval(timer);
  });
</script>

<div class="page">
  <header>
    <h1 class="page-title">Direct connect</h1>
    <p class="muted">Join any BeamMP server by address, including private, LAN and unlisted servers.</p>
  </header>

  <form class="lookup panel" onsubmit={lookUp}>
    <label class="field">
      <span class="label">Server address</span>
      <input bind:value={address} placeholder="192.168.1.20, play.example.com or host:30814" spellcheck="false" oninput={() => (error = null)} />
    </label>
    <button class="btn btn-secondary" disabled={looking || !address.trim()}>
      {#if looking}<span class="spin"><Icon name="refresh" size={14} /></span>{/if} Look up
    </button>
    <button type="button" class="btn btn-primary" disabled={!address.trim() || !store.canPlay} onclick={joinAddress}>Join</button>
  </form>

  {#if error}
    <p class="error"><Icon name="alert" size={14} /> {error}</p>
  {/if}

  {#if result}
    {@const s = result.server}
    <section class="result panel">
      <div class="r-head">
        <div class="r-title">
          <div class="r-name"><BeamText text={s.name} /></div>
          <div class="muted r-addr mono">{s.ip}:{s.port}</div>
        </div>
        <button class="btn btn-primary" disabled={!store.canPlay} onclick={() => store.join(s)}><Icon name="play" size={13} /> Join</button>
      </div>
      {#if result.details}
        <dl class="facts">
          <div><dt>Map</dt><dd>{mapName(s.map)}</dd></div>
          <div><dt>Players</dt><dd>{s.players} / {s.max_players}</dd></div>
          <div><dt>Ping</dt><dd>{result.ping_ms != null ? `${result.ping_ms} ms` : "—"}</dd></div>
          <div><dt>Mods</dt><dd>{s.mods.length ? `${s.mods.length} · ${bytes(s.mods_size)}` : "None"}</dd></div>
          <div><dt>Version</dt><dd>{s.version || "—"}</dd></div>
          <div><dt>Guests</dt><dd>{s.guests ? "Allowed" : "Account required"}</dd></div>
        </dl>
        {#if s.description}<p class="desc"><BeamText text={s.description} multiline /></p>{/if}
        {#if s.player_names.length}<p class="people muted">Online: {s.player_names.join(", ")}</p>{/if}
      {:else}
        <p class="muted">The server is up ({result.ping_ms ?? "?"} ms) but doesn't share its details. You can still join.</p>
      {/if}
      <div class="save">
        <input bind:value={saveName} placeholder="Name for your list" spellcheck="false" />
        <button class="btn btn-secondary" onclick={save}>{resultSaved ? "Rename" : "Save server"}</button>
      </div>
    </section>
  {/if}

  <section class="saved">
    <div class="s-head">
      <h2 class="section-title">Saved servers</h2>
      {#if saved.length}<button class="btn btn-quiet btn-sm" onclick={() => saved.forEach((s) => refreshOne(serverKey(s)))}><Icon name="refresh" size={13} /> Refresh</button>{/if}
    </div>
    {#if saved.length}
      <div class="panel list">
        {#each saved as s (serverKey(s))}
          {@const l = live[serverKey(s)]}
          <ServerRow
            name={s.name}
            ownName
            server={l?.info?.details ? l.info.server : null}
            detail={`${s.ip}:${s.port}`}
            ping={l?.loading && !l.info ? -1 : (l?.info?.ping_ms ?? null)}
            offline={!!l?.error}
          >
            {#snippet actions()}
              <button class="icon-btn" title="Remove" aria-label="Remove" onclick={() => remove(s)}><Icon name="trash" size={14} /></button>
              <button class="btn btn-secondary btn-sm" disabled={!store.canPlay} onclick={() => store.join(l?.info?.server ?? s)}>Join</button>
            {/snippet}
          </ServerRow>
        {/each}
      </div>
    {:else}
      <p class="muted empty">Servers you save show up here with their live status. Look one up above and choose Save server.</p>
    {/if}
  </section>

  <section class="help">
    <h2 class="section-title">Hosting your own?</h2>
    <p class="muted">
      Friends on your network use your LAN address (for example 192.168.1.20). Players outside it need the server's port (30814 by default, TCP and UDP) forwarded on your router, then your public IP or a domain.
    </p>
  </section>
</div>

<style>
  .page {
    height: 100%;
    overflow-y: auto;
    padding: 24px 28px 40px;
    max-width: 920px;
  }
  header p {
    margin: 4px 0 18px;
  }
  .lookup {
    display: flex;
    align-items: flex-end;
    gap: 8px;
    padding: 14px;
  }
  .field {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .error {
    display: flex;
    align-items: center;
    gap: 7px;
    color: var(--bad);
    margin: 10px 2px 0;
    font-size: 13px;
  }
  .result {
    margin-top: 12px;
    padding: 16px;
  }
  .r-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }
  .r-title {
    min-width: 0;
  }
  .r-name {
    font-size: 15px;
    font-weight: 600;
  }
  .r-addr {
    margin-top: 2px;
  }
  .facts {
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    margin: 14px 0 0;
    border: 1px solid var(--line);
    border-radius: var(--radius);
  }
  .facts div {
    padding: 8px 10px;
    border-right: 1px solid var(--line);
    min-width: 0;
  }
  .facts div:last-child {
    border-right: none;
  }
  dt {
    font-size: 11.5px;
    color: var(--muted);
  }
  dd {
    margin: 2px 0 0;
    font-weight: 550;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .desc {
    margin: 12px 0 0;
    color: var(--text-2);
    line-height: 1.5;
  }
  .people {
    margin: 8px 0 0;
    font-size: 12.5px;
  }
  .save {
    display: flex;
    gap: 8px;
    margin-top: 14px;
    padding-top: 14px;
    border-top: 1px solid var(--line);
  }
  .save input {
    flex: 1;
    max-width: 320px;
  }
  .saved,
  .help {
    margin-top: 28px;
  }
  .s-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 10px;
  }
  .empty {
    margin: 0;
  }
  .help p {
    margin: 6px 0 0;
    line-height: 1.55;
    max-width: 680px;
  }
</style>
