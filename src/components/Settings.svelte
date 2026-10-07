<script lang="ts">
  import Icon from "./Icon.svelte";
  import { api } from "../lib/api";
  import { store } from "../lib/store.svelte";
  import type { Settings } from "../lib/types";

  let draft = $state<Settings | null>(null);
  let saving = $state(false);

  $effect(() => {
    if (store.view && !draft) draft = structuredClone($state.snapshot(store.view.settings));
  });

  const view = $derived(store.view);
  const dirty = $derived(!!draft && !!view && JSON.stringify(draft) !== JSON.stringify(view.settings));

  async function save() {
    if (!draft) return;
    saving = true;
    try {
      store.view = await api.saveSettings($state.snapshot(draft));
      draft = structuredClone($state.snapshot(store.view.settings));
      store.toast("Settings saved");
    } catch (e) {
      store.toast(String(e), "err");
    } finally {
      saving = false;
    }
  }

  async function uninstallMods() {
    try {
      const removed = await api.removeMods();
      store.toast(removed.length ? `Removed ${removed.length} item(s) from BeamNG` : "Nothing to remove", "info");
      await store.refresh();
    } catch (e) {
      store.toast(String(e), "err");
    }
  }

  function folder(which: string) {
    api.openFolder(which).catch((e) => store.toast(String(e), "err"));
  }
</script>

<div class="settings fade-in">
  <header>
    <h1 class="display">Settings</h1>
    <button class="btn btn-primary" disabled={!dirty || saving} onclick={save}>{saving ? "Saving…" : "Save changes"}</button>
  </header>

  {#if draft && view}
    <section class="panel">
      <h2 class="display">Installation</h2>
      <div class="status">
        {#each [
          ["BeamNG.drive", view.install.game_found],
          ["BeamMP launcher", view.install.launcher],
          ["BeamMP client mod", view.install.client_mod],
          ["BeamLink companion", view.install.companion],
        ] as [label, ok]}
          <div class="st"><span class="dot" class:good={ok} class:bad={!ok}></span>{label}</div>
        {/each}
      </div>
      <div class="row-actions">
        <button class="btn btn-ghost btn-sm" onclick={() => (store.setupOpen = true)}><Icon name="wrench" size={15} /> Run setup / repair</button>
        <button class="btn btn-danger btn-sm" onclick={uninstallMods}><Icon name="trash" size={14} /> Remove BeamLink mods from the game</button>
      </div>
      {#if !view.install.launcher_supported}
        <p class="hint">
          BeamMP publishes its launcher for Windows only. On Linux, build <span class="mono">BeamMP-Launcher</span> from source and put it in the launcher folder below.
        </p>
      {/if}
    </section>

    <section class="panel">
      <h2 class="display">Folders</h2>
      <p class="hint">Leave blank to detect automatically. Detected: {view.paths.notes.join(" · ") || "nothing yet"}.</p>
      <div class="field">
        <span class="eyebrow">BeamNG.drive install</span>
        <div class="with-btn">
          <input bind:value={draft.game_dir} placeholder={view.paths.game_dir ?? "not found"} spellcheck="false" />
          <button class="btn btn-ghost btn-sm" onclick={() => folder("game")} disabled={!view.paths.game_dir}><Icon name="folder" size={14} /></button>
        </div>
      </div>
      <div class="field">
        <span class="eyebrow">BeamNG user folder</span>
        <div class="with-btn">
          <input bind:value={draft.user_dir} placeholder={view.paths.user_root ?? "not found"} spellcheck="false" />
          <button class="btn btn-ghost btn-sm" onclick={() => folder("user")} disabled={!view.paths.user_dir}><Icon name="folder" size={14} /></button>
        </div>
      </div>
      <div class="field">
        <span class="eyebrow">BeamMP launcher folder</span>
        <div class="with-btn">
          <input bind:value={draft.launcher_dir} placeholder={view.paths.launcher_dir} spellcheck="false" />
          <button class="btn btn-ghost btn-sm" onclick={() => folder("launcher")}><Icon name="folder" size={14} /></button>
        </div>
      </div>
      {#if view.paths.game_version}<p class="hint">Game version {view.paths.game_version}</p>{/if}
    </section>

    <section class="panel">
      <h2 class="display">Multiplayer</h2>
      <div class="two">
        <label class="field">
          <span class="eyebrow">Launcher port</span>
          <input type="number" min="1024" max="65535" bind:value={draft.launcher_port} />
          <span class="hint">Must match BeamMP's in-game setting. Default 4444.</span>
        </label>
        <label class="field">
          <span class="eyebrow">BeamMP branch</span>
          <select bind:value={draft.branch}>
            <option value="Default">Default (stable)</option>
            <option value="Public">Public (testing)</option>
          </select>
        </label>
      </div>
      <label class="field">
        <span class="eyebrow">Extra game arguments</span>
        <input bind:value={draft.game_args} placeholder="e.g. -gfx vk" spellcheck="false" />
      </label>
      <label class="check">
        <input type="checkbox" bind:checked={draft.sync_favorites} />
        <span>Copy favorites into BeamMP's in-game server list</span>
      </label>
    </section>

    <section class="panel about">
      <h2 class="display">About</h2>
      <p class="hint">
        BeamLink {view.version}. Multiplayer is powered by the official
        <button class="link" onclick={() => api.openUrl("https://beammp.com")}>BeamMP</button> launcher and client mod, which BeamLink downloads from BeamMP and checks before installing. BeamLink isn't affiliated with BeamMP or BeamNG.
      </p>
    </section>
  {/if}
</div>

<style>
  .settings {
    height: 100%;
    overflow-y: auto;
    padding: 26px 32px 30px;
    max-width: 980px;
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
  section {
    padding: 20px 22px;
    margin-bottom: 14px;
  }
  h2 {
    margin: 0 0 14px;
    font-size: 22px;
  }
  .status {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 10px;
    margin-bottom: 14px;
  }
  .st {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 10px 12px;
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--line);
    font-weight: 600;
  }
  .row-actions {
    display: flex;
    gap: 8px;
  }
  .hint {
    color: var(--muted);
    font-size: 12.5px;
    line-height: 1.5;
    margin: 10px 0 0;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 12px;
  }
  .field .hint {
    margin: 0;
  }
  .with-btn {
    display: flex;
    gap: 8px;
  }
  .with-btn input {
    flex: 1;
  }
  .with-btn .btn {
    height: 42px;
  }
  .two {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px;
  }
  .two .field {
    margin-top: 0;
  }
  .check {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 16px;
    cursor: pointer;
  }
  .check input {
    width: 18px;
    height: 18px;
    accent-color: var(--accent);
  }
  .link {
    color: var(--text);
    text-decoration: underline;
    font-size: inherit;
    padding: 0;
  }
</style>
