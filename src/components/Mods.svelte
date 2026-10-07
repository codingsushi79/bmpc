<script lang="ts">
  import Icon from "./Icon.svelte";
  import { api } from "../lib/api";
  import { store } from "../lib/store.svelte";
  import { bytes } from "../lib/beam";
  import type { ModsReport } from "../lib/types";

  let report = $state<ModsReport | null>(null);
  let busy = $state(false);

  async function load() {
    try {
      report = await api.mods();
    } catch (e) {
      store.toast(String(e), "err");
    }
  }

  $effect(() => {
    load();
  });

  async function toggle(name: string, active: boolean) {
    try {
      await api.setModActive(name, active);
      await load();
    } catch (e) {
      store.toast(String(e), "err");
    }
  }

  async function clear() {
    busy = true;
    try {
      const freed = await api.clearCache();
      store.toast(`Freed ${bytes(freed)} of downloaded server mods`);
      await load();
    } catch (e) {
      store.toast(String(e), "err");
    } finally {
      busy = false;
    }
  }

  const own = $derived(report?.mods.filter((m) => !m.managed) ?? []);
  const managed = $derived(report?.mods.filter((m) => m.managed) ?? []);
  const activeSize = $derived(own.filter((m) => m.active !== false).reduce((n, m) => n + m.bytes, 0));
</script>

<div class="mods fade-in">
  <header>
    <h1 class="display">Mods</h1>
    <div class="actions">
      <button class="btn btn-ghost btn-sm" onclick={() => api.openFolder("mods").catch((e) => store.toast(String(e), "err"))}><Icon name="folder" size={15} /> Open mods folder</button>
      <button class="icon-btn" onclick={load} title="Rescan"><Icon name="refresh" /></button>
    </div>
  </header>

  <div class="top">
    <div class="panel tile">
      <div class="eyebrow">Your mods</div>
      <div class="big display">{own.length}</div>
      <div class="muted">{bytes(activeSize)} enabled</div>
    </div>
    <div class="panel tile">
      <div class="eyebrow">Server mod cache</div>
      <div class="big display">{bytes(report?.cache_bytes ?? 0)}</div>
      <div class="muted">{report?.cache_files ?? 0} files from servers you've joined</div>
      <div class="tile-actions">
        <button class="btn btn-ghost btn-sm" onclick={() => api.openFolder("cache").catch((e) => store.toast(String(e), "err"))}>Open</button>
        <button class="btn btn-danger btn-sm" disabled={busy || !report?.cache_bytes} onclick={clear}><Icon name="trash" size={14} /> Clear cache</button>
      </div>
    </div>
    <div class="panel tile note">
      <Icon name="alert" size={18} />
      <p>
        BeamMP turns off every mod a server didn't send you while you're on that server, so your own mods won't clash with theirs. They come back when you leave.
      </p>
    </div>
  </div>

  <section>
    <div class="eyebrow">Installed in BeamNG.drive</div>
    {#if !report}
      <div class="muted">Scanning…</div>
    {:else if !report.mods_dir}
      <div class="muted">BeamNG's user folder wasn't found. Set it in Settings.</div>
    {:else if !own.length}
      <div class="empty muted">No mods of your own yet. Drop .zip mods into the mods folder, or install them from the in-game repository.</div>
    {:else}
      <div class="list panel">
        {#each own as m (m.location + m.file)}
          <div class="row">
            <Icon name="box" size={17} />
            <div class="main">
              <div class="file">{m.file}</div>
              <div class="muted sub">{m.location === "repository" ? "from the in-game repository" : "mods folder"}</div>
            </div>
            <div class="size">{bytes(m.bytes)}</div>
            <label class="switch" title={m.active === null ? "BeamNG hasn't seen this mod yet" : m.active ? "Enabled" : "Disabled"}>
              <input type="checkbox" checked={m.active !== false} disabled={m.active === null} onchange={(e) => toggle(m.name, (e.currentTarget as HTMLInputElement).checked)} />
              <span></span>
            </label>
          </div>
        {/each}
      </div>
    {/if}
  </section>

  <section>
    <div class="eyebrow">Managed by BeamLink</div>
    <div class="list panel">
      {#each managed as m (m.location + m.file)}
        <div class="row">
          <Icon name="check" size={17} />
          <div class="main">
            <div class="file">{m.file}</div>
            <div class="muted sub">{m.name === "beamlink" ? "BeamLink companion — one-click join" : "BeamMP client mod — multiplayer"}</div>
          </div>
          <div class="size">{bytes(m.bytes)}</div>
          <span class="badge official">{m.active === false ? "Off — fixed on next launch" : "Active"}</span>
        </div>
      {:else}
        <div class="row muted">Not installed yet — run setup from Settings.</div>
      {/each}
    </div>
  </section>
</div>

<style>
  .mods {
    height: 100%;
    overflow-y: auto;
    padding: 26px 32px 30px;
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    margin-bottom: 18px;
  }
  h1 {
    margin: 0;
    font-size: 40px;
    line-height: 1;
  }
  .actions {
    display: flex;
    gap: 8px;
  }
  .top {
    display: grid;
    grid-template-columns: 1fr 1.3fr 1.4fr;
    gap: 14px;
    margin-bottom: 24px;
  }
  .tile {
    padding: 16px 18px;
  }
  .big {
    font-size: 40px;
    line-height: 1.1;
    margin: 4px 0;
  }
  .tile-actions {
    display: flex;
    gap: 8px;
    margin-top: 12px;
  }
  .note {
    display: flex;
    gap: 12px;
    color: var(--text-2);
    line-height: 1.5;
  }
  .note p {
    margin: 0;
    font-size: 13px;
  }
  .note :global(svg) {
    flex: none;
    color: var(--warn);
    margin-top: 2px;
  }
  section {
    margin-bottom: 24px;
  }
  section > .eyebrow {
    margin-bottom: 10px;
  }
  .list {
    overflow: hidden;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--line);
  }
  .row:last-child {
    border-bottom: none;
  }
  .main {
    flex: 1;
    min-width: 0;
  }
  .file {
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sub {
    font-size: 12px;
    margin-top: 2px;
  }
  .size {
    color: var(--text-2);
    min-width: 80px;
    text-align: right;
  }
  .empty {
    padding: 20px;
    border: 1px dashed var(--line-hi);
    border-radius: var(--radius);
  }
  .switch {
    position: relative;
    width: 42px;
    height: 24px;
    flex: none;
  }
  .switch input {
    opacity: 0;
    width: 0;
    height: 0;
    position: absolute;
  }
  .switch span {
    position: absolute;
    inset: 0;
    border-radius: 24px;
    background: rgba(255, 255, 255, 0.12);
    transition: background 0.2s;
    cursor: pointer;
  }
  .switch span::after {
    content: "";
    position: absolute;
    top: 3px;
    left: 3px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #fff;
    transition: transform 0.2s;
  }
  .switch input:checked + span {
    background: var(--grad);
  }
  .switch input:checked + span::after {
    transform: translateX(18px);
  }
  .switch input:disabled + span {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
